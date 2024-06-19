// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(crate) mod ast;
pub(crate) mod hir;

use self::{ast::run_ast_lints, hir::run_hir_lints};
use crate::lints::{ast::AstLint, hir::HirLint};
use miette::{Diagnostic, LabeledSpan};
use qsc_data_structures::span::Span;
use qsc_frontend::compile::CompileUnit;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

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

/// A lint emited by the linter.
#[derive(Debug, Clone, thiserror::Error)]
pub struct Lint {
    /// A span indicating where the diagnostic is in the source code.
    pub span: Span,
    /// The lint level: allow, warning, error.
    pub level: LintLevel,
    /// The message the user will see in the code editor.
    pub message: &'static str,
    /// The help text the user will see in the code editor.
    pub help: &'static str,
    /// An enum identifying this lint.
    pub kind: LintKind,
}

impl std::fmt::Display for Lint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Diagnostic for Lint {
    fn severity(&self) -> Option<miette::Severity> {
        match self.level {
            LintLevel::Allow => None,
            LintLevel::Warn | LintLevel::ForceWarn => Some(miette::Severity::Warning),
            LintLevel::Error | LintLevel::ForceError => Some(miette::Severity::Error),
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        let source_span = miette::SourceSpan::from(self.span);
        let labeled_span = LabeledSpan::new_with_span(None, source_span);
        Some(Box::new(vec![labeled_span].into_iter()))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        if self.help.is_empty() {
            None
        } else {
            Some(Box::new(self.help))
        }
    }
}

/// A lint level. This defines if a lint will be treated as a warning or an error,
/// and if the lint level can be overriden by the user.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LintLevel {
    /// The lint is effectively disabled.
    Allow,
    /// The lint will be treated as a warning.
    Warn,
    /// The lint will be treated as a warning and cannot be overriden by the user.
    ForceWarn,
    /// The lint will be treated as an error.
    Error,
    /// The lint will be treated as an error and cannot be overriden by the user.
    ForceError,
}

impl Display for LintLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self {
            LintLevel::Allow => "",
            LintLevel::Warn | LintLevel::ForceWarn => "warning",
            LintLevel::Error | LintLevel::ForceError => "error",
        };

        write!(f, "{level}")
    }
}

/// End-user configuration for each lint level.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LintConfig {
    #[serde(rename = "lint")]
    /// Represents the lint name.
    pub kind: LintKind,
    /// The lint level.
    pub level: LintLevel,
}

/// Represents a lint name.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum LintKind {
    /// AST lint name.
    Ast(AstLint),
    /// HIR lint name.
    Hir(HirLint),
}
