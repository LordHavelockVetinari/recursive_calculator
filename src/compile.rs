use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;

use crate::expression::Expression;
use crate::math::Value;
use crate::parse as p;
use crate::program::{Function, Program, WeakConstant, WeakFunction};

#[derive(Debug, thiserror::Error)]
pub enum CompilationError {
    #[error("constant not found: {0}")]
    ConstantNotFound(String),
    #[error("function not found: {0}")]
    FunctionNotFound(String),
    #[error("{0} is a constant, not a function")]
    ConstantNotFunction(String),
    #[error("{0} is a function, not a constant")]
    FunctionNotConstant(String),
    #[error("constant or function {0:?} declared more than once")]
    DuplicateDeclaration(String),
    #[error("bad equation")]
    BadEquation,
    #[error("function parameter must be an identifier")]
    BadParameter,
    #[error("parameter {0:?} shadows a global constant or function")]
    ParamShadowsGlobal(String),
    #[error("parameter {0:?} declared more than once")]
    DuplicateParameter(String),
    #[error("function {0:?} got {1} argument(s) instead of {2}")]
    WrongNArgs(String, usize, usize),
}

struct LocalContext {
    param_indices: HashMap<String, usize>,
}

static GLOBAL_CONTEXT: Lazy<LocalContext> = Lazy::new(|| LocalContext {
    param_indices: HashMap::new(),
});

fn compile_expression(
    expr: &p::Expression,
    program: &Program,
    context: &LocalContext,
) -> Result<Expression, CompilationError> {
    Ok(match expr {
        p::Expression::Number(n) => Expression::Value(Value::Number(n.clone())),
        p::Expression::Identifier(name) => {
            if let Some(&index) = context.param_indices.get(name) {
                Expression::ArgumentIndex(index)
            } else if let Some(constant) = program.get_constant(name) {
                Expression::Constant(WeakConstant::from(constant))
            } else if program.get_function(name).is_some() {
                return Err(CompilationError::FunctionNotConstant(name.clone()));
            } else {
                return Err(CompilationError::ConstantNotFound(name.clone()));
            }
        }
        p::Expression::Pos(expr) => compile_expression(expr, program, context)?,
        p::Expression::Neg(expr) => {
            Expression::Neg(Box::new(compile_expression(expr, program, context)?))
        }
        p::Expression::Add(left, right) => Expression::Add(
            Box::new(compile_expression(left, program, context)?),
            Box::new(compile_expression(right, program, context)?),
        ),
        p::Expression::Sub(left, right) => Expression::Sub(
            Box::new(compile_expression(left, program, context)?),
            Box::new(compile_expression(right, program, context)?),
        ),
        p::Expression::Mul(left, right) => Expression::Mul(
            Box::new(compile_expression(left, program, context)?),
            Box::new(compile_expression(right, program, context)?),
        ),
        p::Expression::Div(left, right) => Expression::Div(
            Box::new(compile_expression(left, program, context)?),
            Box::new(compile_expression(right, program, context)?),
        ),
        p::Expression::Pow(left, right) => Expression::Pow(
            Box::new(compile_expression(left, program, context)?),
            Box::new(compile_expression(right, program, context)?),
        ),
        p::Expression::Call(name, args) => {
            let Some(function) = program.get_function(name) else {
                return Err(if program.get_constant(name).is_some() {
                    CompilationError::ConstantNotFunction(name.clone())
                } else {
                    CompilationError::FunctionNotFound(name.clone())
                });
            };
            let args = args
                .iter()
                .map(|arg| compile_expression(arg, program, context))
                .collect::<Result<Vec<Expression>, CompilationError>>()?;
            let n_params = program.get_n_params(name).unwrap();
            if args.len() != n_params {
                return Err(CompilationError::WrongNArgs(
                    name.clone(),
                    args.len(),
                    n_params,
                ));
            }
            Expression::Call(WeakFunction::from(function), args)
        }
    })
}

fn compile_constant(
    program: &mut Program,
    constant: &str,
    value: &p::Expression,
) -> Result<(), CompilationError> {
    let value = compile_expression(value, program, &GLOBAL_CONTEXT)?;
    program.get_weak_constant(constant).unwrap().init(value);
    Ok(())
}

fn compile_function(
    program: &mut Program,
    function: &str,
    params: &[String],
    code: &p::Expression,
) -> Result<(), CompilationError> {
    for (i, p) in params.into_iter().enumerate() {
        if program.get_constant_or_function(p).is_some() {
            return Err(CompilationError::ParamShadowsGlobal(p.clone()));
        }
        if params[..i].contains(p) {
            return Err(CompilationError::DuplicateParameter(p.to_string()))
        }
    }
    let context = LocalContext {
        param_indices: params.iter().cloned().zip(0..).collect(),
    };
    let result = compile_expression(code, program, &context)?;
    program
        .get_weak_function(function)
        .unwrap()
        .init(Function::new(params.len(), result));
    Ok(())
}

fn compile_assignment(
    program: &mut Program,
    assigned: &p::Expression,
    value: &p::Expression,
) -> Result<(), CompilationError> {
    match assigned {
        p::Expression::Identifier(constant) => compile_constant(program, constant, value),
        p::Expression::Call(function, params) => {
            let params = params
                .iter()
                .map(|param| match param {
                    p::Expression::Identifier(name) => Ok(name.clone()),
                    _ => Err(CompilationError::BadParameter),
                })
                .collect::<Result<Vec<String>, CompilationError>>()?;
            compile_function(program, function, &params, value)
        }
        _ => Err(CompilationError::BadEquation),
    }
}

fn compile_multi_assignment(
    program: &mut Program,
    assigned: &[p::Expression],
    value: &p::Expression,
) -> Result<(), CompilationError> {
    for a in assigned {
        compile_assignment(program, a, value)?;
    }
    Ok(())
}

fn assert_no_duplicate_assignments(code: &p::Code) -> Result<(), CompilationError> {
    let mut assigned = HashSet::new();
    for stmt in &code.statements {
        let p::Statement::Assign(exprs) = stmt else {
            continue;
        };
        for expr in exprs.split_last().unwrap().1 {
            let (p::Expression::Identifier(name) | p::Expression::Call(name, _)) = expr else {
                return Err(CompilationError::BadEquation);
            };
            if !assigned.insert(name.clone()) {
                return Err(CompilationError::DuplicateDeclaration(name.clone()));
            }
        }
    }
    Ok(())
}

fn insert_uninit_global(
    global: &p::Expression,
    program: &mut Program,
) -> Result<(), CompilationError> {
    match global {
        p::Expression::Identifier(name) => {
            program.define_constant(name.clone());
        }
        p::Expression::Call(name, params) => {
            program.define_function(name.clone(), params.len());
        }
        _ => return Err(CompilationError::BadEquation),
    }
    Ok(())
}

fn insert_uninit_globals(code: &p::Code, program: &mut Program) -> Result<(), CompilationError> {
    for stmt in &code.statements {
        let p::Statement::Assign(exprs) = stmt else {
            continue;
        };
        for global in exprs.split_last().unwrap().1 {
            insert_uninit_global(global, program)?;
        }
    }
    Ok(())
}

pub fn compile_into(code: p::Code, program: &mut Program) -> Result<(), CompilationError> {
    assert_no_duplicate_assignments(&code)?;
    insert_uninit_globals(&code, program)?;
    for stmt in code.statements {
        match stmt {
            p::Statement::Assign(mut exprs) => {
                let value = exprs.pop().unwrap();
                compile_multi_assignment(program, &exprs, &value)?;
            }
            p::Statement::Evaluate(expr) => {
                let expr = compile_expression(&expr, program, &GLOBAL_CONTEXT)?;
                program.evaluate_later(expr);
            }
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn compile(code: p::Code) -> Result<Program, CompilationError> {
    let mut program = Program::new();
    compile_into(code, &mut program)?;
    Ok(program)
}
