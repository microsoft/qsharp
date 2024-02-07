use super::{declare_lint, push_lint};
use crate::linter::{ast::AstLintPass, Lint, LintLevel};
#[allow(unused_imports)]
use qsc_ast::ast::{
    Attr, Block, CallableDecl, Expr, ExprKind, FunctorExpr, Ident, Item, Namespace, Package, Pat,
    Path, QubitInit, SpecDecl, Stmt, Ty, TyDef, Visibility,
};
use qsc_ast::ast::{BinOp, Lit};

declare_lint!(
    DoubleParens,
    LintLevel::Warn,
    "unnecesary double parentheses"
);
declare_lint!(DivisionByZero, LintLevel::Deny, "attempt to divide by zero");

impl AstLintPass for DoubleParens {
    fn check_expr(&self, expr: &qsc_ast::ast::Expr) {
        if let ExprKind::Paren(ref inner_expr) = *expr.kind {
            if matches!(*inner_expr.kind, ExprKind::Paren(_)) {
                push_lint!(Self, expr);
            }
        }
    }
}

impl AstLintPass for DivisionByZero {
    fn check_expr(&self, expr: &qsc_ast::ast::Expr) {
        if let ExprKind::BinOp(BinOp::Div, _, ref rhs) = *expr.kind {
            if let ExprKind::Lit(ref lit) = *rhs.kind {
                if let Lit::Int(ref x) = **lit {
                    if *x == 0 {
                        push_lint!(Self, expr);
                    }
                }
            }
        }
    }
}
