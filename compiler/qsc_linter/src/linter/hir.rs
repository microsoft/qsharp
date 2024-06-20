// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::rc::Rc;

use super::LintKind;
use crate::{
    lints::hir::{CombinedHirLints, HirLint},
    Lint, LintConfig, LintLevel,
};
use qsc_data_structures::span::Span;
use qsc_frontend::compile::{CompileUnit, PackageStore};
use qsc_hir::{
    hir::{
        Block, CallableDecl, Expr, ExprKind, Field, Ident, Item, ItemId, ItemKind, Package,
        PackageId, Pat, QubitInit, Res, SpecDecl, Stmt,
    },
    ty::Ty,
    visit::walk_expr,
    visit::Visitor,
};

/// The entry point to the HIR linter. It takes a [`qsc_hir::hir::Package`]
/// as input and outputs a [`Vec<Lint>`](Lint).
#[must_use]
pub fn run_hir_lints(
    package_store: &PackageStore,
    user_package_id: PackageId,
    compile_unit: &CompileUnit,
    config: Option<&[LintConfig]>,
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

    let mut lints =
        CombinedHirLints::from_config(config, package_store, user_package_id, compile_unit);

    for (_, item) in &compile_unit.package.items {
        lints.visit_item(item);
    }

    for stmt in &compile_unit.package.stmts {
        lints.visit_stmt(stmt);
    }

    let mut lint = DeprecatedWithOperator2::new(user_package_id, package_store);
    lint.visit_package(&compile_unit.package);

    lints.buffer.extend(lint.buffer);
    lints.buffer
}

/// Represents a lint pass in the HIR.
/// You only need to implement the `check_*` function relevant to your lint.
/// The trait provides default empty implementations for the rest of the methods,
/// which will be optimized to a no-op by the rust compiler.
pub(crate) trait HirLintPass<'compilation> {
    fn check_block(&self, _block: &Block, _buffer: &mut Vec<Lint>) {}
    fn check_callable_decl(&self, _callable_decl: &CallableDecl, _buffer: &mut Vec<Lint>) {}
    fn check_expr(&self, _expr: &Expr, _buffer: &mut Vec<Lint>) {}
    fn check_ident(&self, _ident: &Ident, _buffer: &mut Vec<Lint>) {}
    fn check_item(&self, _item: &Item, _buffer: &mut Vec<Lint>) {}
    fn check_package(&self, _package: &Package, _buffer: &mut Vec<Lint>) {}
    fn check_pat(&self, _pat: &Pat, _buffer: &mut Vec<Lint>) {}
    fn check_qubit_init(&self, _qubit_init: &QubitInit, _buffer: &mut Vec<Lint>) {}
    fn check_spec_decl(&self, _spec_decl: &SpecDecl, _buffer: &mut Vec<Lint>) {}
    fn check_stmt(&self, _stmt: &Stmt, _buffer: &mut Vec<Lint>) {}
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
        use qsc_frontend::compile::{CompileUnit, PackageStore};
        use qsc_hir::hir::PackageId;
        $(declare_hir_lints!{ @LINT_STRUCT $lint_name, $default_level, $msg, $help })*

        // This is a silly wrapper module to avoid contaminating the environment
        // calling the macro with unwanted imports.
        mod _hir_macro_expansion {
            use crate::{linter::hir::{declare_hir_lints, HirLintPass}, Lint, LintLevel};
            use qsc_hir::{
                hir::{Block, CallableDecl, Expr, Ident, Item, Package, Pat, QubitInit, SpecDecl, Stmt},
                visit::{self, Visitor},
            };
            use qsc_frontend::compile::{CompileUnit, PackageStore};
            use qsc_hir::hir::PackageId;
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
        #[allow(dead_code)]
        pub(crate) struct $lint_name<'compilation> {
            level: LintLevel,
            message: &'static str,
            help: &'static str,
            kind: LintKind,
            package_store: &'compilation PackageStore,
            user_package_id: PackageId,
            compile_unit: &'compilation CompileUnit,
        }

        impl<'compilation> $lint_name<'compilation> {
            const DEFAULT_LEVEL: LintLevel = $default_level;
            fn new(
                package_store: &'compilation PackageStore,
                user_package_id: PackageId,
                compile_unit: &'compilation CompileUnit
            ) -> Self {
                Self { level: Self::DEFAULT_LEVEL, message: $msg, help: $help, kind: LintKind::Hir(HirLint::$lint_name), package_store, user_package_id, compile_unit }
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
            $($lint_name: $lint_name<'compilation>),*
        }

        // Most of the calls here are empty methods and they get optimized at compile time to a no-op.
        impl<'compilation> CombinedHirLints<'compilation> {
            pub fn new(
                package_store: &'compilation PackageStore,
                user_package_id: PackageId,
                compile_unit: &'compilation CompileUnit
            ) -> Self {
                Self {
                    buffer: Vec::default(),
                    $($lint_name: <$lint_name>::new(package_store, user_package_id, compile_unit)),*
                }
            }

            pub fn from_config(
                config: Vec<(HirLint, LintLevel)>,
                package_store: &'compilation PackageStore,
                user_package_id: PackageId,
                compile_unit: &'compilation CompileUnit,
            ) -> Self {
                let mut combined_hir_lints = Self::new(package_store, user_package_id, compile_unit);
                for (lint, level) in config {
                    match lint {
                        $(HirLint::$lint_name => combined_hir_lints.$lint_name.level = level),*
                    }
                }
                combined_hir_lints
            }

            fn check_block(&mut self, block: &Block) { $(self.$lint_name.check_block(block, &mut self.buffer));* }
            fn check_callable_decl(&mut self, decl: &CallableDecl) { $(self.$lint_name.check_callable_decl(decl, &mut self.buffer));* }
            fn check_expr(&mut self, expr: &Expr) { $(self.$lint_name.check_expr(expr, &mut self.buffer));* }
            fn check_ident(&mut self, ident: &Ident) { $(self.$lint_name.check_ident(ident, &mut self.buffer));* }
            fn check_item(&mut self, item: &Item) { $(self.$lint_name.check_item(item, &mut self.buffer));* }
            fn check_package(&mut self, package: &Package) { $(self.$lint_name.check_package(package, &mut self.buffer));* }
            fn check_pat(&mut self, pat: &Pat) { $(self.$lint_name.check_pat(pat, &mut self.buffer));* }
            fn check_qubit_init(&mut self, init: &QubitInit) { $(self.$lint_name.check_qubit_init(init, &mut self.buffer));* }
            fn check_spec_decl(&mut self, decl: &SpecDecl) { $(self.$lint_name.check_spec_decl(decl, &mut self.buffer));* }
            fn check_stmt(&mut self, stmt: &Stmt) { $(self.$lint_name.check_stmt(stmt, &mut self.buffer));* }
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

struct WithOperatorLint {
    span: Span,
    ty_name: Rc<str>,
    is_w_eq: bool,
    field_assigns: Vec<(Rc<str>, Rc<str>)>,
}

struct DeprecatedWithOperator2<'a> {
    user_package_id: PackageId,
    package_store: &'a PackageStore,
    buffer: Vec<Lint>,
    lint_info: Option<WithOperatorLint>,
}

impl<'a> DeprecatedWithOperator2<'a> {
    fn new(user_package_id: PackageId, package_store: &'a PackageStore) -> Self {
        Self {
            user_package_id,
            package_store,
            buffer: Vec::new(),
            lint_info: None,
        }
    }

    /// Returns a substring of the user code's `SourceMap` in the range `lo..hi`.
    fn get_source_code(&self, span: Span) -> String {
        let unit = self
            .package_store
            .get(self.user_package_id)
            .expect("user package should exist");

        let source = unit
            .sources
            .find_by_offset(span.lo)
            .expect("source should exist");

        let lo = (span.lo - source.offset) as usize;
        let hi = (span.hi - source.offset) as usize;
        source.contents[lo..hi].to_string()
    }

    fn indentation_at_offset(&self, offset: u32) -> u32 {
        let unit = self
            .package_store
            .get(self.user_package_id)
            .expect("user package should exist");

        let source = unit
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

impl Visitor<'_> for DeprecatedWithOperator2<'_> {
    fn visit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::UpdateField(container, field, value)
            | ExprKind::AssignField(container, field, value) => {
                if let Ty::Udt(ty_name, Res::Item(item_id)) = &container.ty {
                    let (item, _, _) = resolve_item_relative_to_user_package(
                        *item_id,
                        self.user_package_id,
                        self.package_store,
                    );
                    if let ItemKind::Ty(_, udt) = &item.kind {
                        if udt.is_struct() {
                            let field_name = if let Field::Path(path) = field {
                                udt.find_field(path)
                                    .expect("field should exist in struct")
                                    .name
                                    .as_ref()
                                    .expect("struct fields always have names")
                                    .clone()
                            } else {
                                panic!("field should be a path");
                            };
                            let field_value = Rc::from(self.get_source_code(value.span));
                            let field_info = (field_name, field_value);

                            match &mut self.lint_info {
                                Some(existing_info) => {
                                    existing_info.field_assigns.push(field_info);
                                }
                                None => {
                                    self.lint_info = Some(WithOperatorLint {
                                        span: expr.span,
                                        ty_name: ty_name.clone(),
                                        is_w_eq: matches!(expr.kind, ExprKind::AssignField(..)),
                                        field_assigns: vec![field_info],
                                    });
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                if let Some(info) = &self.lint_info {
                    // Construct a Struct constructor expr and print it back into Q# code
                    let indentation = (self.indentation_at_offset(info.span.lo) + 4) as usize;
                    let innermost_expr = self.get_source_code(expr.span);
                    let mut new_expr = if info.is_w_eq {
                        format!("set {} = new {} {{\n", innermost_expr, info.ty_name)
                    } else {
                        format!("new {} {{\n", info.ty_name)
                    };
                    new_expr.push_str(&format!(
                        "{:indent$}...{},\n",
                        "",
                        innermost_expr,
                        indent = indentation
                    ));
                    for (field, value) in info.field_assigns.iter().rev() {
                        new_expr.push_str(&format!(
                            "{:indent$}{} = {},\n",
                            "",
                            field,
                            value,
                            indent = indentation
                        ));
                    }
                    new_expr.push_str(&format!("{:indent$}}}", "", indent = indentation - 4));

                    let lint = Lint {
                        span: info.span,
                        level: LintLevel::Warn,
                        message: "deprecated `w/` and `w/=` operators for structs",
                        help:
                            "`w/` and `w/=` operators for structs are deprecated, use `new` instead",
                        kind: LintKind::Hir(HirLint::DeprecatedWithOperator),
                        code_action_edits: vec![(new_expr, info.span)],
                    };
                    self.buffer.push(lint);
                    self.lint_info = None;
                }
            }
        }
        walk_expr(self, expr);
    }
}

fn resolve_item_relative_to_user_package(
    item_id: ItemId,
    user_package_id: PackageId,
    package_store: &PackageStore,
) -> (&Item, &Package, ItemId) {
    resolve_item(package_store, user_package_id, item_id)
}

fn resolve_item(
    package_store: &PackageStore,
    local_package_id: PackageId,
    item_id: ItemId,
) -> (&Item, &Package, ItemId) {
    // If the `ItemId` contains a package id, use that.
    // Lack of a package id means the item is in the
    // same package as the one this `ItemId` reference
    // came from. So use the local package id passed in.
    let package_id = item_id.package.unwrap_or(local_package_id);
    let package = &package_store
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

pub(crate) use declare_hir_lints;
