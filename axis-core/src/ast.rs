// axis-core/src/ast.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<FnDef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDef {
    pub name: Ident,
    pub param: Param,
    pub ret_type: Type,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(pub String);

// -----------------------------
// Types
// -----------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    Unit,
    Tuple(Vec<Type>),          // (T1, T2, ...)
    Func(Box<Type>, Box<Type>) // A -> B
}

// -----------------------------
// Blocks & statements
// -----------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub expr: Expr, // final expression
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Let {
        name: Ident,
        ty: Type,
        value: Expr,
    },
    Rebind {
        name: Ident,
        value: Expr,
    },
    Expr {
        expr: Expr,
    },
}

// -----------------------------
// Expressions
// -----------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    LetIn {
        name: Ident,
        ty: Type,
        value: Box<Expr>,
        body: Box<Expr>,
    },

    If {
        cond: Box<Expr>,
        then_br: Box<Expr>,
        else_br: Box<Expr>,
    },

    Lambda {
        param: Param,
        body: Box<Expr>,
    },

    App {
        head: Box<Expr>,    // atom base + projections applied
        args: Vec<Expr>,    // from (call | app_arg)* lowering
    },

    Atom(Atom),
}

// -----------------------------
// Atoms (projection is here)
// -----------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Atom {
    pub base: AtomBase,
    pub projs: Vec<usize>, // e.g. .1.2 => [1,2]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtomBase {
    Lit(Lit),
    Var(Ident),
    Tuple(Vec<Expr>),     // tuple literal
    Paren(Box<Expr>),     // (expr)
    Block(Box<Block>),    // { ... }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lit {
    Int(i64),
    Bool(bool),
    Unit,
}
