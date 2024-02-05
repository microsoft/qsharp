#[cfg(test)]
mod tests;

use super::linter;
use crate::linter::{ast::LintPass, Lint, LintLevel};
#[allow(unused_imports)]
use qsc_ast::ast::{
    Attr, Block, CallableDecl, Expr, ExprKind, FunctorExpr, Ident, Item, Namespace, Package, Pat,
    Path, QubitInit, SpecDecl, Stmt, Ty, TyDef, Visibility,
};
use qsc_ast::ast::{BinOp, Lit};

macro_rules! declare_lint {
    ($lint_name:ident, $level:expr, $msg:expr) => {
        struct $lint_name;

        impl $lint_name {
            const LEVEL: LintLevel = $level;
            const MESSAGE: &'static str = $msg;
        }
    };
}

macro_rules! push_lint {
    ($lint_ty:ty, $node:expr) => {
        linter::push(Lint {
            node_id: $node.id,
            span: $node.span,
            message: <$lint_ty>::MESSAGE,
            level: <$lint_ty>::LEVEL,
        })
    };
}

declare_lint!(
    DoubleParens,
    LintLevel::Warn,
    "unnecesary double parentheses"
);
declare_lint!(DivisionByZero, LintLevel::Deny, "attempt to divide by zero");

impl<'a> LintPass<'a> for DoubleParens {
    fn check_expr(&mut self, expr: &'a qsc_ast::ast::Expr) {
        if let ExprKind::Paren(ref inner_expr) = *expr.kind {
            if matches!(*inner_expr.kind, ExprKind::Paren(_)) {
                push_lint!(Self, expr);
            }
        }
    }
}

impl<'a> LintPass<'a> for DivisionByZero {
    fn check_expr(&mut self, expr: &'a qsc_ast::ast::Expr) {
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
