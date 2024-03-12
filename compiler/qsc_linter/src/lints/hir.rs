use crate::linter::hir::declare_hir_lints;

use super::push_lint;

declare_hir_lints! {
    (Stump, LintLevel::Allow, "remove this stump after addding the first HIR lint"),
}

impl HirLintPass for Stump {
    fn check_ident(&self, ident: &qsc_hir::hir::Ident, buffer: &mut Vec<crate::Lint>) {
        if &*ident.name == "stump" {
            push_lint!(self, ident.span, buffer);
        }
    }
}
