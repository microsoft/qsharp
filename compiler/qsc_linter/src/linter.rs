// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(crate) mod ast;
pub(crate) mod hir;

use crate::lints::{ast::AstLint, hir::HirLint};
use qsc_data_structures::linter::{Lint, LintLevel};
use qsc_frontend::compile::CompileUnit;
use serde::{Deserialize, Serialize};

use self::{ast::run_ast_lints, hir::run_hir_lints};

/// The entry point to the linter. It takes a [`qsc_frontend::compile::CompileUnit`]
/// as input and outputs a [`Vec<Lint>`](Lint).
#[must_use]
pub fn run_lints(compile_unit: &CompileUnit, config: Option<&[LintConfig]>) -> Vec<Lint> {
    let mut ast_lints = run_ast_lints(&compile_unit.ast.package, config);
    let mut hir_lints = run_hir_lints(&compile_unit.package, config);

    let mut lints = Vec::new();
    lints.append(&mut ast_lints);
    lints.append(&mut hir_lints);
    lints
        .into_iter()
        .filter(|lint| !matches!(lint.level, LintLevel::Allow))
        .collect()
}

/// End-user configuration for each lint level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintConfig {
    #[serde(rename = "lint")]
    /// Represents the lint name.
    pub kind: LintKind,
    /// The lint level.
    pub level: LintLevel,
}

/// Represents a lint name.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LintKind {
    /// AST lint name.
    Ast(AstLint),
    /// HIR lint name.
    Hir(HirLint),
}
