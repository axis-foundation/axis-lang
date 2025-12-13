use crate::ast::{Ident, Lit, Param, Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreExpr {
    Seq {
        first: Box<CoreExpr>,
        then: Box<CoreExpr>,
    },
    LetIn {
        name: Ident,
        ty: Type,
        value: Box<CoreExpr>,
        body: Box<CoreExpr>,
    },
    If {
        cond: Box<CoreExpr>,
        then_br: Box<CoreExpr>,
        else_br: Box<CoreExpr>,
    },
    Lambda {
        param: Param,
        body: Box<CoreExpr>,
    },
    App {
        head: Box<CoreExpr>,
        args: Vec<CoreExpr>,
    },
    Atom(CoreAtom),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreAtom {
    pub base: CoreAtomBase,
    pub projs: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreAtomBase {
    Lit(Lit),
    Var(Ident),
    Tuple(Vec<CoreExpr>),
    Paren(Box<CoreExpr>), // keep for now (optional to erase later)
}
