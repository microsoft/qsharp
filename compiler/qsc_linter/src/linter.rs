// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(crate) mod ast;
pub(crate) mod hir;

use self::{ast::run_ast_lints, hir::run_hir_lints};
use crate::lints::{ast::AstLint, hir::HirLint};
use miette::{Diagnostic, LabeledSpan};
use qsc_data_structures::span::Span;
use qsc_frontend::compile::{CompileUnit, PackageStore};
use qsc_hir::hir::{Item, ItemId};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// The entry point to the linter. It takes a [`qsc_frontend::compile::CompileUnit`]
/// as input and outputs a [`Vec<Lint>`](Lint).
#[must_use]
pub fn run_lints(
    package_store: &PackageStore,
    compile_unit: &CompileUnit,
    config: Option<&[LintConfig]>,
) -> Vec<Lint> {
    let compilation = Compilation {
        package_store,
        compile_unit,
    };

    let mut ast_lints = run_ast_lints(&compile_unit.ast.package, config, compilation);
    let mut hir_lints = run_hir_lints(&compile_unit.package, config, compilation);

    let mut lints = Vec::new();
    lints.append(&mut ast_lints);
    lints.append(&mut hir_lints);
    lints
}

#[derive(Clone, Copy)]
pub(crate) struct Compilation<'a> {
    pub package_store: &'a PackageStore,
    pub compile_unit: &'a CompileUnit,
}

impl Compilation<'_> {
    /// Resolves an item id to an item.
    pub fn resolve_item_id(&self, item_id: &ItemId) -> &Item {
        let package = match item_id.package {
            Some(package_id) => {
                &self
                    .package_store
                    .get(package_id)
                    .expect("package should exist in store")
                    .package
            }
            None => &self.compile_unit.package,
        };
        package
            .items
            .get(item_id.item)
            .expect("item id should exist")
    }

    /// Returns a substring of the user code's `SourceMap` in the range `lo..hi`.
    pub fn get_source_code(&self, span: Span) -> String {
        let source = self
            .compile_unit
            .sources
            .find_by_offset(span.lo)
            .expect("source should exist");

        let lo = (span.lo - source.offset) as usize;
        let hi = (span.hi - source.offset) as usize;
        source.contents[lo..hi].to_string()
    }

    /// Returns the indentation at the given offset.
    pub fn indentation_at_offset(&self, offset: u32) -> u32 {
        let source = self
            .compile_unit
            .sources
            .find_by_offset(offset)
            .expect("source should exist");

        let mut indentation = 0;
        for c in source.contents[..(offset - source.offset) as usize]
            .chars()
            .rev()
        {
            if c == '\n' {
                break;
            } else if c == ' ' {
                indentation += 1;
            } else if c == '\t' {
                indentation += 4;
            } else {
                indentation = 0;
            }
        }
        indentation
    }
}

/// A lint emitted by the linter.
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
    /// The suggested edits to fix the lint.
    pub code_action_edits: Vec<(String, Span)>,
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
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LintLevel {
    /// The lint is effectively disabled.
    #[default]
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
