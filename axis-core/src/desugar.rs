use std::collections::HashMap;

use crate::ast::*;
use crate::core_ast::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesugarError {
    RebindWithoutPriorLet { name: String },
}

pub fn desugar_program(p: &Program) -> Result<Vec<(Ident, CoreExpr)>, DesugarError> {
    // For now: just desugar each function body to a CoreExpr.
    // Later we can build a proper CoreProgram struct.
    let mut out = Vec::new();
    for f in &p.functions {
        let body = desugar_block(&f.body)?;
        out.push((f.name.clone(), body));
    }
    Ok(out)
}

pub fn desugar_block(b: &Block) -> Result<CoreExpr, DesugarError> {
    // We build nested `let` bindings by folding statements in reverse order.
    // However, `rebind` needs the type from a *prior* `let` in source order.
    // So we do a forward pass first to validate and record the type for each rebind.
    #[derive(Clone)]
    enum StmtOut<'a> {
        Bind { name: Ident, ty: Type, value: &'a Expr },
        Expr { expr: &'a Expr },
    }

    let mut types: HashMap<String, Type> = HashMap::new();
    let mut stmts_out: Vec<StmtOut<'_>> = Vec::with_capacity(b.stmts.len());

    for stmt in &b.stmts {
        match stmt {
            Stmt::Let { name, ty, value } => {
                types.insert(name.0.clone(), ty.clone());
                stmts_out.push(StmtOut::Bind {
                    name: name.clone(),
                    ty: ty.clone(),
                    value,
                });
            }
            Stmt::Rebind { name, value } => {
                let ty = types.get(&name.0).cloned().ok_or_else(|| {
                    DesugarError::RebindWithoutPriorLet {
                        name: name.0.clone(),
                    }
                })?;
                stmts_out.push(StmtOut::Bind {
                    name: name.clone(),
                    ty,
                    value,
                });
            }
            Stmt::Expr { expr } => {
                stmts_out.push(StmtOut::Expr { expr });
            }
        }
    }

    // Start from the final expression and wrap statements outward (reverse fold).
    let mut acc = desugar_expr(&b.expr)?;
    for stmt in stmts_out.into_iter().rev() {
        match stmt {
            StmtOut::Bind { name, ty, value } => {
                let v = desugar_expr(value)?;
                acc = CoreExpr::LetIn {
                    name,
                    ty,
                    value: Box::new(v),
                    body: Box::new(acc),
                };
            }
            StmtOut::Expr { expr } => {
                let first = desugar_expr(expr)?;
                acc = CoreExpr::Seq {
                    first: Box::new(first),
                    then: Box::new(acc),
                };
            }
        }
    }

    Ok(acc)
}

pub fn desugar_expr(e: &Expr) -> Result<CoreExpr, DesugarError> {
    match e {
        Expr::LetIn { name, ty, value, body } => Ok(CoreExpr::LetIn {
            name: name.clone(),
            ty: ty.clone(),
            value: Box::new(desugar_expr(value)?),
            body: Box::new(desugar_expr(body)?),
        }),

        Expr::If { cond, then_br, else_br } => Ok(CoreExpr::If {
            cond: Box::new(desugar_expr(cond)?),
            then_br: Box::new(desugar_expr(then_br)?),
            else_br: Box::new(desugar_expr(else_br)?),
        }),

        Expr::Lambda { param, body } => Ok(CoreExpr::Lambda {
            param: param.clone(),
            body: Box::new(desugar_expr(body)?),
        }),

        Expr::App { head, args } => Ok(CoreExpr::App {
            head: Box::new(desugar_expr(head)?),
            args: args.iter().map(desugar_expr).collect::<Result<Vec<_>, _>>()?,
        }),

        Expr::Atom(a) => Ok(CoreExpr::Atom(desugar_atom(a)?)),
    }
}

pub fn desugar_atom(a: &Atom) -> Result<CoreAtom, DesugarError> {
    let base = match &a.base {
        AtomBase::Lit(l) => CoreAtomBase::Lit(l.clone()),
        AtomBase::Var(id) => CoreAtomBase::Var(id.clone()),
        AtomBase::Tuple(xs) => CoreAtomBase::Tuple(
            xs.iter().map(desugar_expr).collect::<Result<Vec<_>, _>>()?
        ),
        AtomBase::Paren(e) => CoreAtomBase::Paren(Box::new(desugar_expr(e)?)),
        AtomBase::Block(b) => {
            // A block used as an expression: desugar it to a CoreExpr, then
            // wrap it as a Paren(expr) atom so it can still live in Atom position.
            // (We can erase Paren later if we want.)
            let ce = desugar_block(b.as_ref())?;
            return Ok(CoreAtom {
                base: CoreAtomBase::Paren(Box::new(ce)),
                projs: a.projs.clone(),
            });
        }
    };

    Ok(CoreAtom { base, projs: a.projs.clone() })
}
