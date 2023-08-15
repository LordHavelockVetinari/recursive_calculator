use crate::bool_gen::BoolGen;
use crate::expression::{Expression, SimplifyResult};
use crate::math::format::{Format, FormattedValue};
use crate::math::Value;
use either::Either;
use std::cell::OnceCell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, BufRead, Write};
use std::rc::{Rc, Weak};

pub struct LazyExpression {
    expression: RefCell<Expression>,
    value: OnceCell<Value>,
}

impl LazyExpression {
    pub fn new_uninit() -> RcConstant {
        Rc::new(Self {
            expression: RefCell::new(Expression::default()),
            value: OnceCell::new(),
        })
    }

    pub fn new(expr: Expression) -> Self {
        Self {
            expression: RefCell::new(expr),
            value: OnceCell::new(),
        }
    }

    pub fn value_if_found(&self) -> Option<&Value> {
        self.value.get()
    }

    fn simplify(&self, gen: &mut BoolGen) -> Option<&Value> {
        if let Some(n) = self.value.get() {
            return Some(n);
        }
        let mut expr = self.expression.try_borrow_mut().unwrap();
        match expr.simplify(gen) {
            SimplifyResult::Done => {
                let n = expr.value_if_found()?;
                self.value
                    .set(n.clone())
                    .unwrap_or_else(|_| panic!("expression was evaluated twice"));
                self.value.get()
            }
            SimplifyResult::SimplifyConstant(to_simplify) => {
                to_simplify.simplify(gen);
                None
            }
        }
    }

    pub fn evaluate(&self, gen: &mut BoolGen) -> &Value {
        loop {
            if let Some(n) = self.simplify(gen) {
                return n;
            }
        }
    }
}

pub struct Function {
    n_params: usize,
    code: Expression,
}

impl Function {
    pub fn new_uninit() -> RcFunction {
        Rc::new(OnceCell::new())
    }

    pub fn new(n_params: usize, code: Expression) -> Self {
        Self { n_params, code }
    }

    pub fn call(&self, args: &[RcConstant]) -> Expression {
        assert_eq!(args.len(), self.n_params);
        let mut code = self.code.clone();
        code.substitute_args(args);
        code
    }
}

pub type RcConstant = Rc<LazyExpression>;
pub type RcFunction = Rc<OnceCell<Function>>;

#[derive(Clone)]
pub struct WeakConstant {
    data: Weak<LazyExpression>,
}

impl WeakConstant {
    pub fn init(&self, expr: Expression) {
        let rc = self
            .data
            .upgrade()
            .expect("constant reference was dropped too early");
        let mut old_expr = rc.expression.borrow_mut();
        assert!(
            old_expr.is_default(),
            "constant reference was initialized twice"
        );
        *old_expr = expr;
    }

    pub fn value_if_found(&self) -> Option<Value> {
        self.data
            .upgrade()
            .expect("constant reference was dropped too early")
            .value
            .get()
            .cloned()
    }

    fn simplify(self, gen: &mut BoolGen) {
        let mut to_simplify = self;
        loop {
            let rc = to_simplify
                .data
                .upgrade()
                .expect("constant reference was dropped too early");
            if rc.value.get().is_some() {
                return;
            }
            let mut expr = rc.expression.try_borrow_mut().unwrap();
            match expr.simplify(gen) {
                SimplifyResult::Done => {
                    if let Some(n) = expr.value_if_found() {
                        rc.value
                            .set(n.clone())
                            .unwrap_or_else(|_| panic!("expression was evaluated twice"));
                    }
                    return;
                }
                SimplifyResult::SimplifyConstant(new_to_simplify) => to_simplify = new_to_simplify,
            }
        }
    }
}

impl From<&RcConstant> for WeakConstant {
    fn from(value: &RcConstant) -> Self {
        Self {
            data: Rc::downgrade(value),
        }
    }
}

#[derive(Clone)]
pub struct WeakFunction {
    data: Weak<OnceCell<Function>>,
}

impl WeakFunction {
    pub fn init(&self, func: Function) {
        self.data
            .upgrade()
            .expect("function reference was dropped too early")
            .set(func)
            .unwrap_or_else(|_| panic!("function reference was initialized twice"));
    }

    pub fn call(&self, args: &[RcConstant]) -> Expression {
        self.data
            .upgrade()
            .expect("function reference was dropped too early")
            .get()
            .expect("uninitialized function reference")
            .call(args)
    }
}

impl From<&RcFunction> for WeakFunction {
    fn from(value: &RcFunction) -> Self {
        Self {
            data: Rc::downgrade(value),
        }
    }
}

#[derive(Clone)]
enum Definition {
    Constant {
        constant: RcConstant,
    },
    Function {
        n_params: usize,
        function: RcFunction,
    },
}

#[derive(Clone)]
pub struct Program {
    old_definitions: Vec<Definition>, // Makes sure old definitions don't get deleted when they are still reachable.
    definitions: HashMap<String, Definition>,
    to_evaluate: Vec<Expression>,
}

#[derive(Debug, thiserror::Error)]
#[error("trying to delete a definition that didn't exist")]
pub struct DefinitionDidntExist;

impl Program {
    pub fn new() -> Self {
        Self {
            old_definitions: vec![],
            definitions: HashMap::new(),
            to_evaluate: vec![],
        }
    }

    pub fn get_constant_or_function(&self, name: &str) -> Option<Either<&RcConstant, &RcFunction>> {
        match self.definitions.get(name)? {
            Definition::Constant { constant } => Some(Either::Left(constant)),
            Definition::Function { function, .. } => Some(Either::Right(function)),
        }
    }

    pub fn get_constant(&self, name: &str) -> Option<&RcConstant> {
        match self.definitions.get(name)? {
            Definition::Constant { constant } => Some(constant),
            _ => None,
        }
    }

    pub fn get_weak_constant(&self, name: &str) -> Option<WeakConstant> {
        Some(WeakConstant::from(self.get_constant(name)?))
    }

    pub fn get_function(&self, name: &str) -> Option<&RcFunction> {
        match self.definitions.get(name)? {
            Definition::Function { function, .. } => Some(function),
            _ => None,
        }
    }

    pub fn get_weak_function(&self, name: &str) -> Option<WeakFunction> {
        Some(WeakFunction::from(self.get_function(name)?))
    }

    pub fn get_n_params(&self, function: &str) -> Option<usize> {
        match self.definitions.get(function)? {
            &Definition::Function { n_params, .. } => Some(n_params),
            _ => None,
        }
    }

    pub fn define_constant(&mut self, name: String) {
        let old_def = self.definitions.insert(
            name,
            Definition::Constant {
                constant: LazyExpression::new_uninit(),
            },
        );
        if let Some(old_def) = old_def {
            self.old_definitions.push(old_def);
        }
    }

    pub fn define_function(&mut self, name: String, n_params: usize) {
        let old_def = self.definitions.insert(
            name,
            Definition::Function {
                function: Function::new_uninit(),
                n_params,
            },
        );
        if let Some(old_def) = old_def {
            self.old_definitions.push(old_def);
        }
    }

    pub fn undefine(&mut self, name: &str) -> Result<(), DefinitionDidntExist> {
        match self.definitions.remove(name) {
            Some(old_def) => {
                self.old_definitions.push(old_def);
                Ok(())
            }
            None => Err(DefinitionDidntExist),
        }
    }

    pub fn evaluate_later(&mut self, expr: Expression) {
        self.to_evaluate.push(expr);
    }

    pub fn run(&mut self, env: &mut Environment<'_>) -> Result<(), Box<dyn Error>> {
        for expr in self.to_evaluate.drain(..) {
            let expr = LazyExpression::new(expr);
            let value = expr.evaluate(&mut env.bool_gen);
            env.output_value(value)?;
        }
        Ok(())
    }
}

pub struct Environment<'a> {
    pub input: Box<dyn BufRead + 'a>,
    pub output: Box<dyn Write + 'a>,
    pub error_output: Box<dyn Write + 'a>,
    pub output_format: Format,
    pub are_errors_fatal: bool,
    pub suggest_help: bool,
    pub show_welcome_message: bool,
    pub bool_gen: BoolGen,
}

impl Default for Environment<'_> {
    fn default() -> Self {
        Self {
            input: Box::new(std::io::stdin().lock()),
            output: Box::new(std::io::stdout()),
            error_output: Box::new(std::io::stderr()),
            output_format: Format::default(),
            are_errors_fatal: false,
            suggest_help: false,
            show_welcome_message: true,
            bool_gen: BoolGen::new(),
        }
    }
}

impl<'a> Environment<'a> {
    pub fn output_value(&mut self, value: &Value) -> Result<(), io::Error> {
        writeln!(self.output, "{}", FormattedValue(self.output_format, value))?;
        Ok(())
    }
}
