use crate::Lint;

/// The entry point to the HIR linter. It takes a [`qsc_ast::ast::Package`]
/// as input and outputs a Vec<[`Lint`]>.
#[must_use]
pub fn run_hir_lints(package: &qsc_hir::hir::Package) -> Vec<Lint> {
    todo!()
}
