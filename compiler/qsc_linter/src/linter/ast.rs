// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    lints::ast::{AstLint, CombinedAstLints},
    Lint, LintConfig, LintLevel,
};
use qsc_ast::{
    ast::{
        Attr, Block, CallableDecl, Expr, FunctorExpr, Ident, Item, Namespace, Package, Pat, Path,
        PathKind, QubitInit, SpecDecl, Stmt, TopLevelNode, Ty, TyDef,
    },
    visit::Visitor,
};

/// The entry point to the AST linter. It takes a [`qsc_ast::ast::Package`]
/// as input and outputs a [`Vec<Lint>`](Lint).
#[must_use]
pub fn run_ast_lints(package: &qsc_ast::ast::Package, config: Option<&[LintConfig]>) -> Vec<Lint> {
    let config: Vec<(AstLint, LintLevel)> = config
        .unwrap_or(&[])
        .iter()
        .filter_map(|lint_config| {
            if let LintKind::Ast(kind) = lint_config.kind {
                Some((kind, lint_config.level))
            } else {
                None
            }
        })
        .collect();

    let mut lints = CombinedAstLints::from_config(config);

    for node in &package.nodes {
        match node {
            TopLevelNode::Namespace(namespace) => lints.visit_namespace(namespace),
            TopLevelNode::Stmt(stmt) => lints.visit_stmt(stmt),
        }
    }

    lints.buffer
}

/// Represents a lint pass in the AST.
/// You only need to implement the `check_*` function relevant to your lint.
/// The trait provides default empty implementations for the rest of the methods,
/// which will be optimized to a no-op by the rust compiler.
pub(crate) trait AstLintPass {
    fn check_attr(&self, _attr: &Attr, _buffer: &mut Vec<Lint>) {}
    fn check_block(&self, _block: &Block, _buffer: &mut Vec<Lint>) {}
    fn check_callable_decl(&self, _callable_decl: &CallableDecl, _buffer: &mut Vec<Lint>) {}
    fn check_expr(&self, _expr: &Expr, _buffer: &mut Vec<Lint>) {}
    fn check_functor_expr(&self, _functor_expr: &FunctorExpr, _buffer: &mut Vec<Lint>) {}
    fn check_ident(&self, _ident: &Ident, _buffer: &mut Vec<Lint>) {}
    fn check_item(&self, _item: &Item, _buffer: &mut Vec<Lint>) {}
    fn check_namespace(&self, _namespace: &Namespace, _buffer: &mut Vec<Lint>) {}
    fn check_package(&self, _package: &Package, _buffer: &mut Vec<Lint>) {}
    fn check_pat(&self, _pat: &Pat, _buffer: &mut Vec<Lint>) {}
    fn check_path(&self, _path: &Path, _buffer: &mut Vec<Lint>) {}
    fn check_path_kind(&self, _path: &PathKind, _buffer: &mut Vec<Lint>) {}
    fn check_qubit_init(&self, _qubit_init: &QubitInit, _buffer: &mut Vec<Lint>) {}
    fn check_spec_decl(&self, _spec_decl: &SpecDecl, _buffer: &mut Vec<Lint>) {}
    fn check_stmt(&self, _stmt: &Stmt, _buffer: &mut Vec<Lint>) {}
    fn check_ty(&self, _ty: &Ty, _buffer: &mut Vec<Lint>) {}
    fn check_ty_def(&self, _ty_def: &TyDef, _buffer: &mut Vec<Lint>) {}
}

/// This macro allow us to declare lints while avoiding boilerplate. It does three things:
///  1. Declares the lint structs with their default [`LintLevel`] and message.
///  2. Declares & Implements the [`AstLintsConfig`] struct.
///  3. Declares & Implements the [`CombinedAstLints`] struct.
///
/// Otherwise, each time a contributor adds a new lint, they would also need to sync the
/// declarations and implementations of [`AstLintsConfig`] and [`CombinedAstLints`] for
/// the lint to be integrated with the our linting infrastructure.
macro_rules! declare_ast_lints {
    ($( ($lint_name:ident, $default_level:expr, $msg:expr, $help:expr) ),* $(,)?) => {
        // Declare the structs representing each lint.
        use crate::{Lint, LintKind, LintLevel, linter::ast::AstLintPass};
        $(declare_ast_lints!{ @LINT_STRUCT $lint_name, $default_level, $msg, $help})*

        // This is a silly wrapper module to avoid contaminating the environment
        // calling the macro with unwanted imports.
        mod _ast_macro_expansion {
            use crate::{linter::ast::{declare_ast_lints, AstLintPass}, Lint, LintLevel};
            use qsc_ast::{
                ast::{
                    Attr, Block, CallableDecl, Expr, FunctorExpr, Ident, Item, Namespace, Package, Pat, Path, PathKind,
                    QubitInit, SpecDecl, Stmt, Ty, TyDef,
                },
                visit::{self, Visitor},
            };
            use super::{$($lint_name),*};

            // Declare & implement the `AstLintsConfig` and CombinedAstLints structs.
            declare_ast_lints!{ @CONFIG_ENUM $($lint_name),* }
            declare_ast_lints!{ @COMBINED_STRUCT $($lint_name),* }
        }

        // This is an internal implementation detail, so we make it public only within the crate.
        pub(crate) use _ast_macro_expansion::CombinedAstLints;

        // This will be used by the language service to configure the linter, so we make it public.
        pub use _ast_macro_expansion::AstLint;
    };

    // Declare & implement a struct representing a lint.
    (@LINT_STRUCT $lint_name:ident, $default_level:expr, $msg:expr, $help:expr) => {
        pub(crate) struct $lint_name {
            level: LintLevel,
        }

        impl Default for $lint_name {
            fn default() -> Self {
                Self { level: Self::DEFAULT_LEVEL }
            }
        }

        impl From<LintLevel> for $lint_name {
            fn from(value: LintLevel) -> Self {
                Self { level: value }
            }
        }

        impl $lint_name {
            const DEFAULT_LEVEL: LintLevel = $default_level;

            const fn lint_kind(&self) -> LintKind {
                LintKind::Ast(AstLint::$lint_name)
            }

            const fn message(&self) -> &'static str {
                $msg
            }

            const fn help(&self) -> &'static str {
                $help
            }
        }
    };

    // Declare the `AstLint` enum.
    (@CONFIG_ENUM $($lint_name:ident),*) => {
        use serde::{Deserialize, Serialize};

        /// An enum listing all existing AST lints.
        #[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
        #[serde(rename_all = "camelCase")]
        pub enum AstLint {
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
        /// Combined AST lints for speed. This combined lint allow us to
        /// evaluate all the lints in a single AST pass, instead of doing
        /// an individual pass for each lint in the linter.
        pub(crate) struct CombinedAstLints {
            pub buffer: Vec<Lint>,
            $($lint_name: $lint_name),*
        }

        impl Default for CombinedAstLints {
            fn default() -> Self {
                Self {
                    buffer: Vec::default(),
                    $($lint_name: <$lint_name>::default()),*
                }
            }
        }

        // Most of the calls here are empty methods and they get optimized at compile time to a no-op.
        impl CombinedAstLints {
            pub fn from_config(config: Vec<(AstLint, LintLevel)>) -> Self {
                let mut combined_ast_lints = Self::default();
                for (lint, level) in config {
                    match lint {
                        $(AstLint::$lint_name => combined_ast_lints.$lint_name.level = level),*
                    }
                }
                combined_ast_lints
            }

            fn check_package(&mut self, package: &Package) { $(self.$lint_name.check_package(package, &mut self.buffer));*; }
            fn check_namespace(&mut self, namespace: &Namespace) { $(self.$lint_name.check_namespace(namespace, &mut self.buffer));*; }
            fn check_item(&mut self, item: &Item) { $(self.$lint_name.check_item(item, &mut self.buffer));*; }
            fn check_attr(&mut self, attr: &Attr) { $(self.$lint_name.check_attr(attr, &mut self.buffer));*; }
            fn check_ty_def(&mut self, def: &TyDef) { $(self.$lint_name.check_ty_def(def, &mut self.buffer));*; }
            fn check_callable_decl(&mut self, decl: &CallableDecl) { $(self.$lint_name.check_callable_decl(decl, &mut self.buffer));*; }
            fn check_spec_decl(&mut self, decl: &SpecDecl) { $(self.$lint_name.check_spec_decl(decl, &mut self.buffer));*; }
            fn check_functor_expr(&mut self, expr: &FunctorExpr) { $(self.$lint_name.check_functor_expr(expr, &mut self.buffer));*; }
            fn check_ty(&mut self, ty: &Ty) { $(self.$lint_name.check_ty(ty, &mut self.buffer));*; }
            fn check_block(&mut self, block: &Block) { $(self.$lint_name.check_block(block, &mut self.buffer));*; }
            fn check_stmt(&mut self, stmt: &Stmt) { $(self.$lint_name.check_stmt(stmt, &mut self.buffer));*; }
            fn check_expr(&mut self, expr: &Expr) { $(self.$lint_name.check_expr(expr, &mut self.buffer));*; }
            fn check_pat(&mut self, pat: &Pat) { $(self.$lint_name.check_pat(pat, &mut self.buffer));*; }
            fn check_qubit_init(&mut self, init: &QubitInit) { $(self.$lint_name.check_qubit_init(init, &mut self.buffer));*; }
            fn check_path(&mut self, path: &Path) { $(self.$lint_name.check_path(path, &mut self.buffer));*; }
            fn check_path_kind(&mut self, path: &PathKind) { $(self.$lint_name.check_path_kind(path, &mut self.buffer));*; }
            fn check_ident(&mut self, ident: &Ident) { $(self.$lint_name.check_ident(ident, &mut self.buffer));*; }
        }

        impl<'a> Visitor<'a> for CombinedAstLints {
            fn visit_package(&mut self, package: &'a Package) {
                self.check_package(package);
                visit::walk_package(self, package);
            }

            fn visit_namespace(&mut self, namespace: &'a Namespace) {
                self.check_namespace(namespace);
                visit::walk_namespace(self, namespace);
            }

            fn visit_item(&mut self, item: &'a Item) {
                self.check_item(item);
                visit::walk_item(self, item);
            }

            fn visit_attr(&mut self, attr: &'a Attr) {
                self.check_attr(attr);
                visit::walk_attr(self, attr);
            }

            fn visit_ty_def(&mut self, def: &'a TyDef) {
                self.check_ty_def(def);
                visit::walk_ty_def(self, def);
            }

            fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
                self.check_callable_decl(decl);
                visit::walk_callable_decl(self, decl);
            }

            fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
                self.check_spec_decl(decl);
                visit::walk_spec_decl(self, decl);
            }

            fn visit_functor_expr(&mut self, expr: &'a FunctorExpr) {
                self.check_functor_expr(expr);
                visit::walk_functor_expr(self, expr);
            }

            fn visit_ty(&mut self, ty: &'a Ty) {
                self.check_ty(ty);
                visit::walk_ty(self, ty);
            }

            fn visit_block(&mut self, block: &'a Block) {
                self.check_block(block);
                visit::walk_block(self, block);
            }

            fn visit_stmt(&mut self, stmt: &'a Stmt) {
                self.check_stmt(stmt);
                visit::walk_stmt(self, stmt);
            }

            fn visit_expr(&mut self, expr: &'a Expr) {
                self.check_expr(expr);
                visit::walk_expr(self, expr);
            }

            fn visit_pat(&mut self, pat: &'a Pat) {
                self.check_pat(pat);
                visit::walk_pat(self, pat);
            }

            fn visit_qubit_init(&mut self, init: &'a QubitInit) {
                self.check_qubit_init(init);
                visit::walk_qubit_init(self, init);
            }

            fn visit_path(&mut self, path: &'a Path) {
                self.check_path(path);
                visit::walk_path(self, path);
            }

            fn visit_path_kind(&mut self, path: &'a PathKind) {
                self.check_path_kind(path);
                visit::walk_path_kind(self, path);
            }

            fn visit_ident(&mut self, ident: &'a Ident) {
                self.check_ident(ident);
            }
        }
    };
}

pub(crate) use declare_ast_lints;

use super::LintKind;
