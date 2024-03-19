use qsc_hir::hir::Lit;

use crate::linter::hir::declare_hir_lints;

use super::lint;

declare_hir_lints! {
    (Placeholder, LintLevel::Allow, "this a placeholder", "remove after addding the first HIR lint"),
}

impl HirLintPass for Placeholder {
    fn check_expr(&self, expr: &qsc_hir::hir::Expr, buffer: &mut Vec<Lint>) {
        if let qsc_hir::hir::ExprKind::Lit(Lit::Int(42)) = &expr.kind {
            buffer.push(lint!(self, expr.span));
        }
    }
}
