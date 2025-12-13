use std::collections::HashMap;

use crate::ast::Ident;
use crate::core_ast::CoreExpr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Unit,
    Tuple(Vec<Value>),
    Closure {
        param: Ident,
        body: CoreExpr,
        env: Env,
    },
}

pub type Env = HashMap<String, Value>;
