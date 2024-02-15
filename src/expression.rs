use crate::ctrlc_handler::CtrlCError;
use crate::environment::EvaluationEnvironemnt;
use crate::math::{self, Value};
use crate::program::{LazyExpression, WeakConstant, WeakFunction};
use malachite::num::basic::traits::One;
use malachite::Rational;
use std::mem::{self, ManuallyDrop};
use std::ptr;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Expression {
    Value(Value),
    Argument(Rc<LazyExpression>),
    Constant(WeakConstant),
    Neg(Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Pow(Box<Expression>, Box<Expression>),
    Call(WeakFunction, Vec<Expression>),
    ArgumentIndex(usize),
}

enum SimplifyStepResult<'a> {
    AlreadySimplified,
    ReplaceWith(Expression),
    SimplifyPart(&'a mut Expression),
    SimplifyConstant(WeakConstant),
}

pub enum SimplifyResult {
    Done,
    SimplifyConstant(WeakConstant),
}

impl Drop for Expression {
    fn drop(&mut self) {
        use Expression::*;
        if !self.has_child_expressions() {
            return;
        }
        let mut to_drop = vec![mem::take(self)];
        while let Some(expr) = to_drop.pop() {
            let mut expr = ManuallyDrop::new(expr);
            match &mut *expr {
                Value(val) => unsafe {
                    ptr::drop_in_place(val as *mut math::Value);
                }
                Argument(arg) => unsafe {
                    let arg = ptr::read(arg as *mut Rc<LazyExpression>);
                    if let Some(arg) = Rc::into_inner(arg) {
                        to_drop.push(arg.destruct_not_recursively());
                    }
                },
                Constant(con) => unsafe {
                    ptr::drop_in_place(con as *mut WeakConstant);
                },
                Neg(operand) => unsafe {
                    let operand = ptr::read(operand as *mut Box<Expression>);
                    to_drop.push(*operand);
                },
                Add(left, right)
                | Sub(left, right)
                | Mul(left, right)
                | Div(left, right)
                | Pow(left, right) => unsafe {
                    let left = ptr::read(left as *mut Box<Expression>);
                    to_drop.push(*left);
                    let right = ptr::read(right as *mut Box<Expression>);
                    to_drop.push(*right);
                },
                Call(func, args) => {
                    unsafe {
                        ptr::drop_in_place(func as *mut WeakFunction);
                    }
                    let args = mem::take(args);
                    for arg in args {
                        to_drop.push(arg);
                    }
                }
                ArgumentIndex(_) => {}
            }
        }
    }
}

impl Expression {
    fn has_child_expressions(&self) -> bool {
        use Expression::*;
        match self {
            Value(_) | Argument(_) | Constant(_) | ArgumentIndex(_) => false,
            Neg(_) | Add(_, _) | Sub(_, _) | Mul(_, _) | Div(_, _) | Pow(_, _) => true,
            Call(_, exprs) => !exprs.is_empty(),
        }
    }

    pub fn substitute_args(&mut self, args: &[Rc<LazyExpression>]) {
        use Expression::*;
        match self {
            Value(_) | Argument(_) | Constant(_) => {}
            Neg(expr) => expr.substitute_args(args),
            Add(left, right)
            | Sub(left, right)
            | Mul(left, right)
            | Div(left, right)
            | Pow(left, right) => {
                left.substitute_args(args);
                right.substitute_args(args);
            }
            Call(_, inner_args) => {
                for arg in inner_args {
                    arg.substitute_args(args);
                }
            }
            &mut ArgumentIndex(i) => *self = Argument(Rc::clone(&args[i])),
        }
    }

    pub fn value_if_found(&self) -> Option<&Value> {
        if let Self::Value(v) = self {
            Some(v)
        } else {
            None
        }
    }

    fn value_if_found_mut(&mut self) -> Option<&mut Value> {
        if let Self::Value(v) = self {
            Some(v)
        } else {
            None
        }
    }

    // If simplify_step returns ReplaceWith, you must replace the expression immediately!
    // Otherwise, the expression may be in an invalid state.
    fn simplify_step(&mut self, env: &mut EvaluationEnvironemnt) -> SimplifyStepResult<'_> {
        use Expression::*;
        use SimplifyStepResult::*;
        match self {
            Value(_) => AlreadySimplified,
            Argument(arg) => {
                if let Some(n) = arg.value_if_found() {
                    ReplaceWith(Value(n.clone()))
                } else {
                    SimplifyConstant(WeakConstant::from(&*arg))
                }
            }
            Constant(con) => {
                if let Some(n) = con.value_if_found() {
                    ReplaceWith(Value(n))
                } else {
                    SimplifyConstant(con.clone())
                }
            }
            Neg(e) => {
                if let Some(n) = e.value_if_found_mut() {
                    ReplaceWith(Value(-mem::take(n)))
                } else {
                    SimplifyPart(e)
                }
            }
            Add(left, right) => match (left.value_if_found_mut(), right.value_if_found_mut()) {
                (Some(u), None) | (None, Some(u)) if u.is_undefined() => {
                    ReplaceWith(Value(mem::take(u)))
                }
                (Some(left), Some(right)) => ReplaceWith(Value(mem::take(left) + right)),
                (Some(_), None) => SimplifyPart(right),
                (None, Some(_)) => SimplifyPart(left),
                (None, None) if env.gen_bool() => SimplifyPart(left),
                (None, None) => SimplifyPart(right),
            },
            Sub(left, right) => match (left.value_if_found_mut(), right.value_if_found_mut()) {
                (Some(u), None) | (None, Some(u)) if u.is_undefined() => {
                    ReplaceWith(Value(mem::take(u)))
                }
                (Some(left), Some(right)) => ReplaceWith(Value(mem::take(left) - right)),
                (Some(_), None) => SimplifyPart(right),
                (None, Some(_)) => SimplifyPart(left),
                (None, None) if env.gen_bool() => SimplifyPart(left),
                (None, None) => SimplifyPart(right),
            },
            Mul(left, right) => match (left.value_if_found_mut(), right.value_if_found_mut()) {
                (Some(x), _) | (_, Some(x)) if x.is_zero() => ReplaceWith(Value(mem::take(x))),
                (Some(n), Some(m)) => ReplaceWith(Value(mem::take(n) * m)),
                (Some(_), None) => SimplifyPart(right),
                (None, Some(_)) => SimplifyPart(left),
                (None, None) if env.gen_bool() => SimplifyPart(left),
                (None, None) => SimplifyPart(right),
            },

            Div(left, right) => match (left.value_if_found_mut(), right.value_if_found_mut()) {
                (Some(u), None) | (None, Some(u)) if u.is_undefined() => {
                    ReplaceWith(Value(mem::take(u)))
                }
                (Some(left), Some(right)) => ReplaceWith(Value(mem::take(left) / right)),
                (Some(_), None) => SimplifyPart(right),
                (None, Some(_)) => SimplifyPart(left),
                (None, None) if env.gen_bool() => SimplifyPart(left),
                (None, None) => SimplifyPart(right),
            },

            Pow(left, right) => match (left.value_if_found_mut(), right.value_if_found_mut()) {
                (Some(n), Some(m)) => ReplaceWith(Value(mem::take(n).pow(m))),
                (Some(one), _) if one.is_one() => ReplaceWith(Value(mem::take(one))),
                (_, Some(z)) if z.is_zero() => {
                    ReplaceWith(Value(math::Value::Number(Rational::ONE)))
                }
                (Some(_), None) => SimplifyPart(right),
                (None, Some(_)) => SimplifyPart(left),
                (None, None) if env.gen_bool() => SimplifyPart(left),
                (None, None) => SimplifyPart(right),
            },
            Call(func, ref mut args) => {
                let args = mem::take(args)
                    .into_iter()
                    .map(|arg| Rc::new(LazyExpression::new(arg)))
                    .collect::<Vec<_>>();
                ReplaceWith(func.call(&args))
            }
            ArgumentIndex(_) => panic!("argument was not substituted"),
        }
    }

    pub fn simplify(
        &mut self,
        env: &mut EvaluationEnvironemnt,
    ) -> Result<SimplifyResult, CtrlCError> {
        use SimplifyStepResult::*;
        // to_simplify should be &mut Expression, but the borrow checker doesn't like that.
        // I think that's a bug in the borrow checker ¯\_(ツ)_/¯.
        let mut to_simplify = self as *mut Expression;
        loop {
            env.tick()?;
            unsafe {
                match (*to_simplify).simplify_step(env) {
                    AlreadySimplified => return Ok(SimplifyResult::Done),
                    ReplaceWith(result) => {
                        *to_simplify = result;
                        return Ok(SimplifyResult::Done);
                    }
                    SimplifyPart(new_to_simplify) => to_simplify = new_to_simplify,
                    SimplifyConstant(con) => return Ok(SimplifyResult::SimplifyConstant(con)),
                }
            }
        }
    }

    pub fn is_default(&self) -> bool {
        matches!(self, Expression::ArgumentIndex(usize::MAX))
    }
}

impl Default for Expression {
    fn default() -> Self {
        Self::ArgumentIndex(usize::MAX)
    }
}
