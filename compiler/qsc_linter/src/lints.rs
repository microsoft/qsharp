#[cfg(test)]
mod tests;

use std::{cell::RefCell, rc::Rc};

use crate::linter::{ast::LintPass, Lint, LintBuffer, LintLevel};

#[allow(unused_imports)]
use qsc_ast::ast::{
    Attr, Block, CallableDecl, Expr, ExprKind, FunctorExpr, Ident, Item, Namespace, Package, Pat,
    Path, QubitInit, SpecDecl, Stmt, Ty, TyDef, Visibility,
};
use qsc_ast::ast::{BinOp, Lit};

struct DoubleParens {
    buffer: Rc<RefCell<LintBuffer>>,
}

impl DoubleParens {
    pub fn new(buffer: Rc<RefCell<LintBuffer>>) -> Self {
        Self { buffer }
    }
}

impl<'a> LintPass<'a> for DoubleParens {
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

struct DivisionByZero {
    buffer: Rc<RefCell<LintBuffer>>,
}

impl DivisionByZero {
    pub fn new(buffer: Rc<RefCell<LintBuffer>>) -> Self {
        Self { buffer }
    }
}

impl<'a> LintPass<'a> for DivisionByZero {
    fn check_expr(&mut self, expr: &'a qsc_ast::ast::Expr) {
        if let ExprKind::BinOp(BinOp::Div, _, ref rhs) = *expr.kind {
            if let ExprKind::Lit(ref lit) = *rhs.kind {
                if let Lit::Int(ref x) = **lit {
                    if *x == 0 {
                        self.buffer.borrow_mut().push(Lint {
                            node_id: expr.id,
                            span: expr.span,
                            message: "attempt to divide by zero".to_owned(),
                            level: LintLevel::Deny,
                        });
                    }
                }
            }
        }
    }
}
