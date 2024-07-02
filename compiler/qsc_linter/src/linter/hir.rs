// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    lints::hir::{CombinedHirLints, HirLint},
    Lint, LintConfig, LintLevel,
};
use qsc_hir::{
    hir::{Block, CallableDecl, Expr, Ident, Item, Package, Pat, QubitInit, SpecDecl, Stmt},
    visit::Visitor,
};

/// The entry point to the HIR linter. It takes a [`qsc_hir::hir::Package`]
/// as input and outputs a [`Vec<Lint>`](Lint).
#[must_use]
pub fn run_hir_lints(
    package: &Package,
    config: Option<&[LintConfig]>,
    compilation: Compilation,
) -> Vec<Lint> {
    let config: Vec<(HirLint, LintLevel)> = config
        .unwrap_or(&[])
        .iter()
        .filter_map(|lint_config| {
            if let LintKind::Hir(kind) = lint_config.kind {
                Some((kind, lint_config.level))
            } else {
                None
            }
        })
        .collect();

    let mut lints = CombinedHirLints::from_config(config, compilation);

    for (_, item) in &package.items {
        lints.visit_item(item);
    }

    for stmt in &package.stmts {
        lints.visit_stmt(stmt);
    }

    lints.buffer
}

/// Represents a lint pass in the HIR.
/// You only need to implement the `check_*` function relevant to your lint.
/// The trait provides default empty implementations for the rest of the methods,
/// which will be optimized to a no-op by the rust compiler.
pub(crate) trait HirLintPass {
    fn check_block(&mut self, _block: &Block, _buffer: &mut Vec<Lint>, _compilation: Compilation) {}
    fn check_callable_decl(
        &mut self,
        _callable_decl: &CallableDecl,
        _buffer: &mut Vec<Lint>,
        _compilation: Compilation,
    ) {
    }
    fn check_expr(&mut self, _expr: &Expr, _buffer: &mut Vec<Lint>, _compilation: Compilation) {}
    fn check_ident(&mut self, _ident: &Ident, _buffer: &mut Vec<Lint>, _compilation: Compilation) {}
    fn check_item(&mut self, _item: &Item, _buffer: &mut Vec<Lint>, _compilation: Compilation) {}
    fn check_package(
        &mut self,
        _package: &Package,
        _buffer: &mut Vec<Lint>,
        _compilation: Compilation,
    ) {
    }
    fn check_pat(&mut self, _pat: &Pat, _buffer: &mut Vec<Lint>, _compilation: Compilation) {}
    fn check_qubit_init(
        &mut self,
        _qubit_init: &QubitInit,
        _buffer: &mut Vec<Lint>,
        _compilation: Compilation,
    ) {
    }
    fn check_spec_decl(
        &mut self,
        _spec_decl: &SpecDecl,
        _buffer: &mut Vec<Lint>,
        _compilation: Compilation,
    ) {
    }
    fn check_stmt(&mut self, _stmt: &Stmt, _buffer: &mut Vec<Lint>, _compilation: Compilation) {}
}

/// This macro allow us to declare lints while avoiding boilerplate. It does three things:
///  1. Declares the lint structs with their default [`LintLevel`] and message.
///  2. Declares & Implements the [`HirLintsConfig`] struct.
///  3. Declares & Implements the [`CombinedHirLints`] struct.
///
/// Otherwise, each time a contributor adds a new lint, they would also need to sync the
/// declarations and implementations of [`HirLintsConfig`] and [`CombinedHirLints`] for
/// the lint to be integrated with the our linting infrastructure.
macro_rules! declare_hir_lints {
    ($( ($lint_name:ident, $default_level:expr, $msg:expr, $help:expr) ),* $(,)?) => {
        // Declare the structs representing each lint.
        use crate::{Lint, LintKind, LintLevel, linter::hir::HirLintPass};
        $(declare_hir_lints!{ @LINT_STRUCT $lint_name, $default_level, $msg, $help })*

        // This is a silly wrapper module to avoid contaminating the environment
        // calling the macro with unwanted imports.
        mod _hir_macro_expansion {
            use crate::{linter::Compilation, linter::hir::{declare_hir_lints, HirLintPass}, Lint, LintLevel};
            use qsc_hir::{
                hir::{Block, CallableDecl, Expr, Ident, Item, Package, Pat, QubitInit, SpecDecl, Stmt},
                visit::{self, Visitor},
            };
            use super::{$($lint_name),*};

            // Declare & implement the `HirLintsConfig` and CombinedHirLints structs.
            declare_hir_lints!{ @CONFIG_ENUM $($lint_name),* }
            declare_hir_lints!{ @COMBINED_STRUCT $($lint_name),* }
        }

        // This is an internal implementation detail, so we make it public only within the crate.
        pub(crate) use _hir_macro_expansion::CombinedHirLints;

        // This will be used by the language service to configure the linter, so we make it public.
        pub use _hir_macro_expansion::HirLint;
    };

    // Declare & implement a struct representing a lint.
    (@LINT_STRUCT $lint_name:ident, $default_level:expr, $msg:expr, $help:expr) => {
        impl $lint_name {
            const DEFAULT_LEVEL: LintLevel = $default_level;

            fn new() -> Self {
                #[allow(clippy::needless_update)]
                Self { level: Self::DEFAULT_LEVEL, ..Default::default() }
            }

            const fn lint_kind(&self) -> LintKind {
                LintKind::Hir(HirLint::$lint_name)
            }

            const fn message(&self) -> &'static str {
                $msg
            }

            const fn help(&self) -> &'static str {
                $help
            }
        }
    };

    // Declare the `HirLint` enum.
    (@CONFIG_ENUM $($lint_name:ident),*) => {
        use serde::{Deserialize, Serialize};

        /// An enum listing all existing HIR lints.
        #[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
        #[serde(rename_all = "camelCase")]
        pub enum HirLint {
            $(
                #[doc = stringify!($lint_name)]
                $lint_name
            ),*
        }
    };

    // Declare & implement the `CombinedAstLints` structure.
    (@COMBINED_STRUCT $($lint_name:ident),*) => {
        // There is no trivial way in rust of converting an identifier from PascalCase
        // to snake_case within `macro_rules`. Since these fields are private and cannot
        // be accessed anywhere outside this macro, I chose to #[allow(non_snake_case)]
        // for field names.
        #[allow(non_snake_case)]
        /// Combined HIR lints for speed. This combined lint allow us to
        /// evaluate all the lints in a single HIR pass, instead of doing
        /// an individual pass for each lint in the linter.
        pub(crate) struct CombinedHirLints<'compilation> {
            pub buffer: Vec<Lint>,
            pub compilation: Compilation<'compilation>,
            $($lint_name: $lint_name),*
        }

        impl<'compilation> CombinedHirLints<'compilation> {
            fn new(compilation: Compilation<'compilation>) -> Self {
                Self {
                    buffer: Vec::new(),
                    compilation,
                    $($lint_name: <$lint_name>::new()),*
                }
            }
        }

        // Most of the calls here are empty methods and they get optimized at compile time to a no-op.
        impl<'compilation> CombinedHirLints<'compilation> {
            pub fn from_config(config: Vec<(HirLint, LintLevel)>, compilation: Compilation<'compilation>) -> Self {
                let mut combined_hir_lints = Self::new(compilation);
                for (lint, level) in config {
                    match lint {
                        $(HirLint::$lint_name => combined_hir_lints.$lint_name.level = level),*
                    }
                }
                combined_hir_lints
            }

            fn check_block(&mut self, block: &Block) { $(self.$lint_name.check_block(block, &mut self.buffer, self.compilation));* }
            fn check_callable_decl(&mut self, decl: &CallableDecl) { $(self.$lint_name.check_callable_decl(decl, &mut self.buffer, self.compilation));* }
            fn check_expr(&mut self, expr: &Expr) { $(self.$lint_name.check_expr(expr, &mut self.buffer, self.compilation));* }
            fn check_ident(&mut self, ident: &Ident) { $(self.$lint_name.check_ident(ident, &mut self.buffer, self.compilation));* }
            fn check_item(&mut self, item: &Item) { $(self.$lint_name.check_item(item, &mut self.buffer, self.compilation));* }
            fn check_package(&mut self, package: &Package) { $(self.$lint_name.check_package(package, &mut self.buffer, self.compilation));* }
            fn check_pat(&mut self, pat: &Pat) { $(self.$lint_name.check_pat(pat, &mut self.buffer, self.compilation));* }
            fn check_qubit_init(&mut self, init: &QubitInit) { $(self.$lint_name.check_qubit_init(init, &mut self.buffer, self.compilation));* }
            fn check_spec_decl(&mut self, decl: &SpecDecl) { $(self.$lint_name.check_spec_decl(decl, &mut self.buffer, self.compilation));* }
            fn check_stmt(&mut self, stmt: &Stmt) { $(self.$lint_name.check_stmt(stmt, &mut self.buffer, self.compilation));* }
        }

        impl<'a> Visitor<'a> for CombinedHirLints<'_> {
            fn visit_block(&mut self, block: &'a Block) {
                self.check_block(block);
                visit::walk_block(self, block);
            }

            fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
                self.check_callable_decl(decl);
                visit::walk_callable_decl(self, decl);
            }

            fn visit_expr(&mut self, expr: &'a Expr) {
                self.check_expr(expr);
                visit::walk_expr(self, expr);
            }

            fn visit_ident(&mut self, ident: &'a Ident) {
                self.check_ident(ident);
            }

            fn visit_item(&mut self, item: &'a Item) {
                self.check_item(item);
                visit::walk_item(self, item);
            }

            fn visit_package(&mut self, package: &'a Package) {
                self.check_package(package);
                visit::walk_package(self, package);
            }

            fn visit_pat(&mut self, pat: &'a Pat) {
                self.check_pat(pat);
                visit::walk_pat(self, pat);
            }

            fn visit_qubit_init(&mut self, init: &'a QubitInit) {
                self.check_qubit_init(init);
                visit::walk_qubit_init(self, init);
            }

            fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
                self.check_spec_decl(decl);
                visit::walk_spec_decl(self, decl);
            }

            fn visit_stmt(&mut self, stmt: &'a Stmt) {
                self.check_stmt(stmt);
                visit::walk_stmt(self, stmt);
            }
        }
    };
}

pub(crate) use declare_hir_lints;

use super::{Compilation, LintKind};
