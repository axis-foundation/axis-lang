use pest::iterators::{Pair, Pairs};

use crate::ast::*;
use crate::parse::Rule;

// ============================================================
// Entry point
// ============================================================

pub fn parse_program(pairs: Pairs<Rule>) -> Program {
    let mut functions = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            // The entrypoint parse uses Rule::program, so pest yields a top-level `program` pair.
            // Also note: `item` is a silent rule, so `program` directly contains `fn_def` pairs.
            Rule::program => {
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::fn_def => functions.push(parse_fn_def(inner)),
                        Rule::EOI => {}
                        _ => unreachable!("unexpected program content: {:?}", inner.as_rule()),
                    }
                }
            }

            // Allow callers to pass a stream of fn_def pairs directly (e.g. in tests).
            Rule::fn_def => functions.push(parse_fn_def(pair)),
            Rule::EOI => {}
            _ => unreachable!("unexpected top-level rule: {:?}", pair.as_rule()),
        }
    }

    Program { functions }
}

// ============================================================
// Functions
// ============================================================

fn parse_fn_def(pair: Pair<Rule>) -> FnDef {
    let mut it = pair.into_inner();

    let name = parse_ident(it.next().unwrap());
    let param_name = parse_ident(it.next().unwrap());
    let param_ty = parse_type(it.next().unwrap());
    let ret_ty = parse_type(it.next().unwrap());
    let body = parse_block(it.next().unwrap());

    FnDef {
        name,
        param: Param {
            name: param_name,
            ty: param_ty,
        },
        ret_type: ret_ty,
        body,
    }
}

// ============================================================
// Identifiers & literals
// ============================================================

fn parse_ident(pair: Pair<Rule>) -> Ident {
    Ident(pair.as_str().to_string())
}

fn parse_lit(pair: Pair<Rule>) -> Lit {
    match pair.as_rule() {
        Rule::int_lit => Lit::Int(pair.as_str().parse().unwrap()),
        Rule::bool_lit => Lit::Bool(pair.as_str() == "true"),
        Rule::unit_lit => Lit::Unit,
        _ => unreachable!("unexpected literal"),
    }
}

// ============================================================
// Types
// ============================================================

fn parse_type(pair: Pair<Rule>) -> Type {
    match pair.as_rule() {
        Rule::type_ => {
            // `type_` is a non-silent wrapper in the grammar.
            let inner = pair.into_inner().next().unwrap();
            parse_type(inner)
        }

        Rule::func_lhs => {
            // `func_lhs` is a non-silent wrapper in the grammar.
            let inner = pair.into_inner().next().unwrap();
            parse_type(inner)
        }

        Rule::paren_type => {
            // Parenthesized type used for grouping, e.g. (Int -> Int)
            let inner = pair.into_inner().next().unwrap();
            parse_type(inner)
        }

        Rule::base_type => match pair.as_str() {
            "Int" => Type::Int,
            "Bool" => Type::Bool,
            "Unit" => Type::Unit,
            _ => unreachable!("unknown base type"),
        },

        Rule::tuple_type => {
            let elems = pair.into_inner().map(parse_type).collect();
            Type::Tuple(elems)
        }

        Rule::func_type => {
            let mut it = pair.into_inner();
            let lhs = parse_type(it.next().unwrap());
            let rhs = parse_type(it.next().unwrap());
            Type::Func(Box::new(lhs), Box::new(rhs))
        }

        _ => unreachable!("unexpected type rule"),
    }
}

// ============================================================
// Blocks & statements
// ============================================================

fn parse_block(pair: Pair<Rule>) -> Block {
    let mut stmts = Vec::new();
    let mut final_expr = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::stmt => {
                let stmt_inner = inner.into_inner().next().unwrap();
                match stmt_inner.as_rule() {
                    Rule::let_stmt => stmts.push(parse_let_stmt(stmt_inner)),
                    Rule::rebind_stmt => stmts.push(parse_rebind_stmt(stmt_inner)),
                    Rule::expr_stmt => stmts.push(parse_expr_stmt(stmt_inner)),
                    _ => unreachable!("unexpected stmt kind: {:?}", stmt_inner.as_rule()),
                }
            }
            // `expr` is a silent rule in the grammar, so we receive its concrete variants here.
            Rule::let_expr | Rule::if_expr | Rule::lambda_expr | Rule::app_expr => {
                final_expr = Some(parse_expr(inner))
            }
            _ => unreachable!("unexpected block content: {:?}", inner.as_rule()),
        }
    }

    let expr = final_expr.unwrap_or_else(|| {
        Expr::Atom(Atom {
            base: AtomBase::Lit(Lit::Unit),
            projs: vec![],
        })
    });

    Block {
        stmts,
        expr,
    }
}

fn parse_let_stmt(pair: Pair<Rule>) -> Stmt {
    let mut it = pair.into_inner();
    Stmt::Let {
        name: parse_ident(it.next().unwrap()),
        ty: parse_type(it.next().unwrap()),
        value: parse_expr(it.next().unwrap()),
    }
}

fn parse_rebind_stmt(pair: Pair<Rule>) -> Stmt {
    let mut it = pair.into_inner();
    Stmt::Rebind {
        name: parse_ident(it.next().unwrap()),
        value: parse_expr(it.next().unwrap()),
    }
}

fn parse_expr_stmt(pair: Pair<Rule>) -> Stmt {
    // `expr` is a silent rule, so expr_stmt directly contains the concrete expr variant.
    let inner = pair.into_inner().next().unwrap();
    Stmt::Expr {
        expr: parse_expr(inner),
    }
}

// ============================================================
// Expressions
// ============================================================

fn parse_expr(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::block => Expr::Atom(Atom {
            base: AtomBase::Block(Box::new(parse_block(pair))),
            projs: vec![],
        }),

        Rule::let_expr => {
            let mut it = pair.into_inner();
            Expr::LetIn {
                name: parse_ident(it.next().unwrap()),
                ty: parse_type(it.next().unwrap()),
                value: Box::new(parse_expr(it.next().unwrap())),
                body: Box::new(parse_expr(it.next().unwrap())),
            }
        }

        Rule::if_expr => {
            let mut it = pair.into_inner();
            Expr::If {
                cond: Box::new(parse_expr(it.next().unwrap())),
                then_br: Box::new(parse_expr(it.next().unwrap())),
                else_br: Box::new(parse_expr(it.next().unwrap())),
            }
        }

        Rule::lambda_expr => {
            let mut it = pair.into_inner();
            let param_name = parse_ident(it.next().unwrap());
            let param_ty = parse_type(it.next().unwrap());
            let body = parse_expr(it.next().unwrap());

            Expr::Lambda {
                param: Param {
                    name: param_name,
                    ty: param_ty,
                },
                body: Box::new(body),
            }
        }

        Rule::app_expr => parse_app_expr(pair),

        _ => unreachable!("unexpected expr rule"),
    }
}

// ============================================================
// Application & atoms
// ============================================================

fn parse_app_expr(pair: Pair<Rule>) -> Expr {
    let mut it = pair.into_inner();

    let head_atom = parse_atom(it.next().unwrap());
    let mut args = Vec::new();

    for part in it {
        match part.as_rule() {
            Rule::call => {
                for arg in part.into_inner() {
                    args.push(parse_expr(arg));
                }
            }
            Rule::app_arg => {
                args.push(parse_app_arg(part));
            }
            _ => unreachable!("unexpected application part"),
        }
    }

    Expr::App {
        head: Box::new(Expr::Atom(head_atom)),
        args,
    }
}

fn parse_app_arg(pair: Pair<Rule>) -> Expr {
    let mut it = pair.into_inner();

    // `app_arg_base` is a silent rule, so we receive its concrete variants here
    // (literal | ident | tuple_lit | paren_expr).
    let base = parse_atom_base(it.next().unwrap());
    let mut projs = Vec::new();

    for proj in it {
        let idx = proj
            .into_inner()
            .next()
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap();
        projs.push(idx);
    }

    Expr::Atom(Atom { base, projs })
}

fn parse_atom(pair: Pair<Rule>) -> Atom {
    let mut it = pair.into_inner();

    let base = parse_atom_base(it.next().unwrap());
    let mut projs = Vec::new();

    for proj in it {
        let idx = proj
            .into_inner()
            .next()
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap();
        projs.push(idx);
    }

    Atom { base, projs }
}

fn parse_atom_base(pair: Pair<Rule>) -> AtomBase {
    match pair.as_rule() {
        Rule::literal => {
            let lit = parse_lit(pair.into_inner().next().unwrap());
            AtomBase::Lit(lit)
        }

        Rule::ident => AtomBase::Var(parse_ident(pair)),

        Rule::tuple_lit => {
            let elems = pair.into_inner().map(parse_expr).collect();
            AtomBase::Tuple(elems)
        }

        Rule::paren_expr => {
            let inner = parse_expr(pair.into_inner().next().unwrap());
            AtomBase::Paren(Box::new(inner))
        }

        Rule::block => AtomBase::Block(Box::new(parse_block(pair))),

        _ => unreachable!("unexpected atom base"),
    }
}
