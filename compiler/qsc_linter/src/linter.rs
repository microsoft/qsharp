// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(crate) mod ast;
pub(crate) mod hir;

use self::{ast::run_ast_lints, hir::run_hir_lints};
use crate::lints::{ast::AstLint, hir::HirLint};
use miette::{Diagnostic, LabeledSpan};
use qsc_ast::ast::NodeId;
use qsc_data_structures::span::Span;
use qsc_doc_gen::display::Lookup;
use qsc_frontend::{
    compile::{CompileUnit, PackageStore},
    resolve,
};
use qsc_hir::{
    hir::{Item, ItemId, Package, PackageId},
    ty,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// The entry point to the linter. It takes a [`qsc_frontend::compile::CompileUnit`]
/// as input and outputs a [`Vec<Lint>`](Lint).
#[must_use]
pub fn run_lints(
    package_store: &PackageStore,
    user_package_id: PackageId,
    compile_unit: &CompileUnit,
    config: Option<&[LintConfig]>,
) -> Vec<Lint> {
    let compilation = Compilation {
        package_store,
        user_package_id,
    };

    let mut ast_lints = run_ast_lints(&compile_unit.ast.package, config);
    let mut hir_lints = run_hir_lints(&compile_unit.package, config, compilation);

    let mut lints = Vec::new();
    lints.append(&mut ast_lints);
    lints.append(&mut hir_lints);
    lints
}

#[derive(Clone, Copy)]
pub(crate) struct Compilation<'a> {
    pub package_store: &'a PackageStore,
    pub user_package_id: PackageId,
}

impl<'a> Lookup for Compilation<'a> {
    fn get_ty(&self, _: NodeId) -> Option<&ty::Ty> {
        unimplemented!("Not needed for linter")
    }

    fn get_res(&self, _: NodeId) -> Option<&resolve::Res> {
        unimplemented!("Not needed for linter")
    }

    /// Returns the hir `Item` node referred to by `item_id`,
    /// along with the `Package` and `PackageId` for the package
    /// that it was found in.
    fn resolve_item_relative_to_user_package(&self, item_id: &ItemId) -> (&Item, &Package, ItemId) {
        self.resolve_item(self.user_package_id, item_id)
    }

    /// Returns the hir `Item` node referred to by `res`.
    /// `Res`s can resolve to external packages, and the references
    /// are relative, so here we also need the
    /// local `PackageId` that the `res` itself came from.
    fn resolve_item_res(
        &self,
        local_package_id: PackageId,
        res: &qsc_hir::hir::Res,
    ) -> (&Item, ItemId) {
        match res {
            qsc_hir::hir::Res::Item(item_id) => {
                let (item, _, resolved_item_id) = self.resolve_item(local_package_id, item_id);
                (item, resolved_item_id)
            }
            _ => panic!("expected to find item"),
        }
    }

    /// Returns the hir `Item` node referred to by `item_id`.
    /// `ItemId`s can refer to external packages, and the references
    /// are relative, so here we also need the local `PackageId`
    /// that the `ItemId` originates from.
    fn resolve_item(
        &self,
        local_package_id: PackageId,
        item_id: &ItemId,
    ) -> (&Item, &Package, ItemId) {
        // If the `ItemId` contains a package id, use that.
        // Lack of a package id means the item is in the
        // same package as the one this `ItemId` reference
        // came from. So use the local package id passed in.
        let package_id = item_id.package.unwrap_or(local_package_id);
        let package = &self
            .package_store
            .get(package_id)
            .expect("package should exist in store")
            .package;
        (
            package
                .items
                .get(item_id.item)
                .expect("item id should exist"),
            package,
            ItemId {
                package: Some(package_id),
                item: item_id.item,
            },
        )
    }
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
