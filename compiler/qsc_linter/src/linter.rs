// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(crate) mod ast;
pub(crate) mod hir;

use miette::{Diagnostic, LabeledSpan};
use qsc_data_structures::span::Span;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::lints::{ast::AstLint, hir::HirLint};

/// A lint emited by the linter.
#[derive(Debug, Clone, thiserror::Error)]
pub struct Lint {
    /// A span indicating where the diagnostic is in the source code.
    pub span: Span,
    /// This is the message the user will see in the code editor.
    pub message: &'static str,
    /// The lint level: allow, warning, error.
    pub level: LintLevel,
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
            LintLevel::Warning | LintLevel::ForceWarning => Some(miette::Severity::Warning),
            LintLevel::Error | LintLevel::ForceError => Some(miette::Severity::Error),
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        let source_span = miette::SourceSpan::from(self.span);
        let labeled_span = LabeledSpan::new_with_span(Some(self.to_string()), source_span);
        Some(Box::new(vec![labeled_span].into_iter()))
    }
}

/// A lint level. This defines if a lint will be treated as a warning or an error,
/// and if the lint level can be overriden by the user.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LintLevel {
    /// The lint is effectively disabled.
    Allow,
    /// The lint will be treated as a warning.
    Warning,
    /// The lint will be treated as a warning and cannot be overriden by the user.
    ForceWarning,
    /// The lint will be treated as an error.
    Error,
    /// The lint will be treated as an error and cannot be overriden by the user.
    ForceError,
}

impl Display for LintLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self {
            LintLevel::Allow => "",
            LintLevel::Warning | LintLevel::ForceWarning => "warning",
            LintLevel::Error | LintLevel::ForceError => "error",
        };

        write!(f, "{level}")
    }
}

/// End-user configuration for each lint level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintConfig {
    #[serde(rename = "lint")]
    pub(crate) kind: LintKind,
    pub(crate) level: LintLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LintKind {
    Ast(AstLint),
    Hir(HirLint),
}
