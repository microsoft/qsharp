#[cfg(test)]
mod tests;

use std::{cell::RefCell, rc::Rc};

use crate::linter::{ast::LintPass, Lint, LintBuffer, LintLevel};

#[allow(unused_imports)]
use qsc_ast::ast::{
    Attr, Block, CallableDecl, Expr, ExprKind, FunctorExpr, Ident, Item, Namespace, Package, Pat,
    Path, QubitInit, SpecDecl, Stmt, Ty, TyDef, Visibility,
};

struct DoubleParens {
    buffer: Rc<RefCell<LintBuffer>>,
}

impl<'a> LintPass<'a> for DoubleParens {
    fn new(buffer: Rc<RefCell<LintBuffer>>) -> Self {
        Self { buffer }
    }

    fn check_expr(&mut self, expr: &'a qsc_ast::ast::Expr) {
        if let ExprKind::Paren(ref inner_expr) = *expr.kind {
            if matches!(*inner_expr.kind, ExprKind::Paren(_)) {
                self.buffer.borrow_mut().push(Lint {
                    node_id: inner_expr.id,
                    span: expr.span,
                    message: "unnecesary double parentheses".to_owned(),
                    level: LintLevel::Warn,
                });
            }
        }
    }
}
