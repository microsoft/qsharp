use super::{declare_lint, push_lint};
use crate::linter::{ast::AstLintPass, Lint, LintLevel};
use qsc_ast::ast::{BinOp, ExprKind, Lit};

declare_lint!(DoubleParens, LintLevel::Warning, "unnecessary parentheses");
declare_lint!(
    DivisionByZero,
    LintLevel::Allow,
    "attempt to divide by zero"
);

impl AstLintPass for DoubleParens {
    fn check_expr(expr: &qsc_ast::ast::Expr, buffer: &mut Vec<Lint>) {
        if let ExprKind::Paren(ref inner_expr) = *expr.kind {
            if matches!(*inner_expr.kind, ExprKind::Paren(_)) {
                push_lint!(Self, expr, buffer);
            }
        }
    }
}

impl AstLintPass for DivisionByZero {
    fn check_expr(expr: &qsc_ast::ast::Expr, buffer: &mut Vec<Lint>) {
        if let ExprKind::BinOp(BinOp::Div, _, ref rhs) = *expr.kind {
            if let ExprKind::Lit(ref lit) = *rhs.kind {
                if let Lit::Int(0) = **lit {
                    push_lint!(Self, expr, buffer);
                }
            }
        }
    }
}
