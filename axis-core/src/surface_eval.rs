use std::collections::HashMap;

use crate::ast::*;
use crate::value::{Env, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceEvalError {
    UnboundVariable(String),
    NotAFunction,
    InvalidProjection,
    NonBoolCondition,
    NonGroundResult, // if result is a closure (we avoid in tests for now)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SValue {
    Int(i64),
    Bool(bool),
    Unit,
    Tuple(Vec<SValue>),
    Closure {
        param: Ident,
        body: Expr,     // SURFACE body
        env: SEnv,      // captured env
    },
}

pub type SEnv = HashMap<String, SValue>;

pub fn env_from_core(env: &Env) -> Result<SEnv, SurfaceEvalError> {
    let mut out = SEnv::new();
    for (k, v) in env.iter() {
        out.insert(k.clone(), svalue_from_value(v)?);
    }
    Ok(out)
}

pub fn value_from_svalue(v: &SValue) -> Result<Value, SurfaceEvalError> {
    match v {
        SValue::Int(n) => Ok(Value::Int(*n)),
        SValue::Bool(b) => Ok(Value::Bool(*b)),
        SValue::Unit => Ok(Value::Unit),
        SValue::Tuple(xs) => Ok(Value::Tuple(
            xs.iter().map(value_from_svalue).collect::<Result<Vec<_>, _>>()?
        )),
        SValue::Closure { .. } => Err(SurfaceEvalError::NonGroundResult),
    }
}

fn svalue_from_value(v: &Value) -> Result<SValue, SurfaceEvalError> {
    match v {
        Value::Int(n) => Ok(SValue::Int(*n)),
        Value::Bool(b) => Ok(SValue::Bool(*b)),
        Value::Unit => Ok(SValue::Unit),
        Value::Tuple(xs) => Ok(SValue::Tuple(
            xs.iter().map(svalue_from_value).collect::<Result<Vec<_>, _>>()?
        )),
        Value::Closure { .. } => Err(SurfaceEvalError::NonGroundResult),
    }
}

// ------------------------------------------------------------
// Public entry points
// ------------------------------------------------------------

pub fn eval_block(block: &Block, env: &SEnv) -> Result<SValue, SurfaceEvalError> {
    // Execute statements sequentially (shadowing/rebinding = env update)
    let mut cur = env.clone();

    for s in &block.stmts {
        match s {
            Stmt::Let { name, value, .. } => {
                let v = eval_expr(value, &cur)?;
                cur.insert(name.0.clone(), v);
            }
            Stmt::Rebind { name, value } => {
                let v = eval_expr(value, &cur)?;
                cur.insert(name.0.clone(), v);
            }
        }
    }

    eval_expr(&block.expr, &cur)
}

pub fn eval_expr(expr: &Expr, env: &SEnv) -> Result<SValue, SurfaceEvalError> {
    match expr {
        Expr::LetIn { name, value, body, .. } => {
            let v = eval_expr(value, env)?;
            let mut new_env = env.clone();
            new_env.insert(name.0.clone(), v);
            eval_expr(body, &new_env)
        }

        Expr::If { cond, then_br, else_br } => {
            match eval_expr(cond, env)? {
                SValue::Bool(true) => eval_expr(then_br, env),
                SValue::Bool(false) => eval_expr(else_br, env),
                _ => Err(SurfaceEvalError::NonBoolCondition),
            }
        }

        Expr::Lambda { param, body } => Ok(SValue::Closure {
            param: param.name.clone(),
            body: (*body.clone()),
            env: env.clone(),
        }),

        Expr::App { head, args } => {
            let mut f = eval_expr(head, env)?;
            for a in args {
                let av = eval_expr(a, env)?;
                f = apply_one(f, av)?;
            }
            Ok(f)
        }

        Expr::Atom(a) => eval_atom(a, env),
    }
}

fn apply_one(func: SValue, arg: SValue) -> Result<SValue, SurfaceEvalError> {
    match func {
        SValue::Closure { param, body, env } => {
            let mut new_env = env.clone();
            new_env.insert(param.0.clone(), arg);
            eval_expr(&body, &new_env)
        }
        _ => Err(SurfaceEvalError::NotAFunction),
    }
}

fn eval_atom(a: &Atom, env: &SEnv) -> Result<SValue, SurfaceEvalError> {
    let mut base = match &a.base {
        AtomBase::Lit(l) => match l {
            Lit::Int(n) => SValue::Int(*n),
            Lit::Bool(b) => SValue::Bool(*b),
            Lit::Unit => SValue::Unit,
        },

        AtomBase::Var(id) => env
            .get(&id.0)
            .cloned()
            .ok_or_else(|| SurfaceEvalError::UnboundVariable(id.0.clone()))?,

        AtomBase::Tuple(xs) => {
            let mut vs = Vec::new();
            for x in xs {
                vs.push(eval_expr(x, env)?);
            }
            SValue::Tuple(vs)
        }

        AtomBase::Paren(e) => eval_expr(e, env)?,

        AtomBase::Block(b) => eval_block(b, env)?,
    };

    // projections
    for idx in &a.projs {
        match base {
            SValue::Tuple(ref elems) => {
                let i = idx - 1;
                if i >= elems.len() {
                    return Err(SurfaceEvalError::InvalidProjection);
                }
                base = elems[i].clone();
            }
            _ => return Err(SurfaceEvalError::InvalidProjection),
        }
    }

    Ok(base)
}
