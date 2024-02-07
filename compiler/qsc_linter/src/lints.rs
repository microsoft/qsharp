pub(crate) mod ast;
#[cfg(test)]
mod tests;

use crate::Lint;
use qsc::SourceMap;

/// The entry point to the AST linter. It takes a [`qsc_ast::ast::Package`]
/// as input and outputs a Vec<[`Lint`]>.
#[must_use]
pub fn run_ast_lints(sources: &SourceMap, package: &qsc_ast::ast::Package) -> Vec<Lint> {
    todo!()
}

/// The entry point to the HIR linter. It takes a [`qsc_ast::ast::Package`]
/// as input and outputs a Vec<[`Lint`]>.
#[must_use]
pub fn run_hir_lints(sources: &SourceMap, package: &qsc_hir::hir::Package) -> Vec<Lint> {
    todo!()
}

macro_rules! declare_lint {
    ($lint_name:ident, $level:expr, $msg:expr) => {
        pub(crate) struct $lint_name;

        impl $lint_name {
            const LEVEL: LintLevel = $level;
            const MESSAGE: &'static str = $msg;
        }
    };
}

macro_rules! push_lint {
    ($lint_ty:ty, $node:expr) => {
        crate::linter::push(Lint {
            node_id: $node.id,
            span: $node.span,
            message: <$lint_ty>::MESSAGE,
            level: <$lint_ty>::LEVEL,
        })
    };
}

pub(crate) use declare_lint;
pub(crate) use push_lint;
