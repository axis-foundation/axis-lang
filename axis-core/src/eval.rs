use crate::core_ast::*;
use crate::value::*;

// ------------------------------------------------------------
// Errors
// ------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    UnboundVariable(String),
    TypeError(String),
    NotAFunction(Value),
    InvalidProjection,
    NonBoolCondition,
}

pub fn eval(expr: &CoreExpr, env: &Env) -> Result<Value, EvalError> {
    match expr {
        CoreExpr::LetIn { name, value, body, .. } => {
            let v = eval(value, env)?;
            let mut new_env = env.clone();
            new_env.insert(name.0.clone(), v);
            eval(body, &new_env)
        }

        CoreExpr::If { cond, then_br, else_br } => {
            match eval(cond, env)? {
                Value::Bool(true) => eval(then_br, env),
                Value::Bool(false) => eval(else_br, env),
                _ => Err(EvalError::NonBoolCondition),
            }
        }

        CoreExpr::Lambda { param, body } => {
            Ok(Value::Closure {
                param: param.name.clone(),
                body: (*body.clone()),
                env: env.clone(),
            })
        }

        CoreExpr::App { head, args } => {
            let func = eval(head, env)?;
            let mut arg_vals = Vec::new();
            for a in args {
                arg_vals.push(eval(a, env)?);
            }
            apply(func, arg_vals)
        }

        CoreExpr::Atom(a) => eval_atom(a, env),
    }
}

// ------------------------------------------------------------
// Application
// ------------------------------------------------------------

fn apply(func: Value, args: Vec<Value>) -> Result<Value, EvalError> {
    let mut f = func;
    let mut rest = args;

    while let Some(arg) = rest.first().cloned() {
        rest.remove(0);

        match f {
            Value::Closure { param, body, env } => {
                let mut new_env = env.clone();
                new_env.insert(param.0.clone(), arg);
                f = eval(&body, &new_env)?;
            }
            _ => return Err(EvalError::NotAFunction(f)),
        }
    }

    Ok(f)
}

// ------------------------------------------------------------
// Atoms
// ------------------------------------------------------------

fn eval_atom(atom: &CoreAtom, env: &Env) -> Result<Value, EvalError> {
    let mut base = match &atom.base {
        CoreAtomBase::Lit(l) => match l {
            crate::ast::Lit::Int(n) => Value::Int(*n),
            crate::ast::Lit::Bool(b) => Value::Bool(*b),
            crate::ast::Lit::Unit => Value::Unit,
        },

        CoreAtomBase::Var(id) => env
            .get(&id.0)
            .cloned()
            .ok_or_else(|| EvalError::UnboundVariable(id.0.clone()))?,

        CoreAtomBase::Tuple(xs) => {
            let mut vs = Vec::new();
            for x in xs {
                vs.push(eval(x, env)?);
            }
            Value::Tuple(vs)
        }

        CoreAtomBase::Paren(e) => eval(e, env)?,
    };

    // Apply projections
    for idx in &atom.projs {
        match base {
            Value::Tuple(ref elems) => {
                let i = idx - 1;
                if i >= elems.len() {
                    return Err(EvalError::InvalidProjection);
                }
                base = elems[i].clone();
            }
            _ => return Err(EvalError::InvalidProjection),
        }
    }

    Ok(base)
}
