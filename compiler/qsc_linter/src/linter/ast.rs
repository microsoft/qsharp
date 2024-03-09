// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::{
    ast::{
        Attr, Block, CallableDecl, Expr, FunctorExpr, Ident, Item, Namespace, Package, Pat, Path,
        QubitInit, SpecDecl, Stmt, TopLevelNode, Ty, TyDef, Visibility,
    },
    visit::Visitor,
};

use crate::{lints::ast::CombinedAstLints, Lint};

/// The entry point to the AST linter. It takes a [`qsc_ast::ast::Package`]
/// as input and outputs a [`Vec<Lint>`](Lint).
#[must_use]
pub fn run_ast_lints(package: &qsc_ast::ast::Package) -> Vec<Lint> {
    let mut lints = CombinedAstLints::default();

    for node in package.nodes.iter() {
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
    fn check_attr(_attr: &Attr, _buffer: &mut Vec<Lint>) {}
    fn check_block(_block: &Block, _buffer: &mut Vec<Lint>) {}
    fn check_callable_decl(_callable_decl: &CallableDecl, _buffer: &mut Vec<Lint>) {}
    fn check_expr(_expr: &Expr, _buffer: &mut Vec<Lint>) {}
    fn check_functor_expr(_functor_expr: &FunctorExpr, _buffer: &mut Vec<Lint>) {}
    fn check_ident(_ident: &Ident, _buffer: &mut Vec<Lint>) {}
    fn check_item(_item: &Item, _buffer: &mut Vec<Lint>) {}
    fn check_namespace(_namespace: &Namespace, _buffer: &mut Vec<Lint>) {}
    fn check_package(_package: &Package, _buffer: &mut Vec<Lint>) {}
    fn check_pat(_pat: &Pat, _buffer: &mut Vec<Lint>) {}
    fn check_path(_path: &Path, _buffer: &mut Vec<Lint>) {}
    fn check_qubit_init(_qubit_init: &QubitInit, _buffer: &mut Vec<Lint>) {}
    fn check_spec_decl(_spec_decl: &SpecDecl, _buffer: &mut Vec<Lint>) {}
    fn check_stmt(_stmt: &Stmt, _buffer: &mut Vec<Lint>) {}
    fn check_ty(_ty: &Ty, _buffer: &mut Vec<Lint>) {}
    fn check_ty_def(_ty_def: &TyDef, _buffer: &mut Vec<Lint>) {}
    fn check_visibility(_visibility: &Visibility, _buffer: &mut Vec<Lint>) {}
}

/// This macro allow us to declare lints while avoiding boilerplate. It does two things:
///  1. Declares the lint structs with their default [`LintLevel`] and message.
///  2. Implements the [`CombinedAstLints`] struct.
macro_rules! declare_ast_lints {
    ($( ($lint_name:ident, $level:expr, $msg:expr) ),* $(,)?) => {
        $(declare_ast_lints!{@ $lint_name, $level, $msg})*
        declare_ast_lints! {@IMPL_COMBINED $($lint_name),* }
    };

    (@ $lint_name:ident, $level:expr, $msg:expr) => {
        pub(crate) struct $lint_name;

        impl $lint_name {
            const LEVEL: LintLevel = $level;
            const MESSAGE: &'static str = $msg;
        }
    };

    (@IMPL_COMBINED $($lint_name:ty),*) => {
        /// Combined AST lints for speed. This combined lint allow us to
        /// evaluate all the lints in a single AST pass, instead of doing
        /// an individual pass for each lint in the linter.
        #[derive(Default)]
        pub(crate) struct CombinedAstLints {
            pub buffer: Vec<Lint>,
        }

        use qsc_ast::{
            ast::{
                Attr, Block, CallableDecl, Expr, FunctorExpr, Ident, Item, Namespace, Package, Pat, Path,
                QubitInit, SpecDecl, Stmt, Ty, TyDef, Visibility,
            },
            visit::{self, Visitor},
        };

        impl CombinedAstLints {
            fn check_package(&mut self, package: &Package) { $(<$lint_name>::check_package(package, &mut self.buffer));*; }
            fn check_namespace(&mut self, namespace: &Namespace) { $(<$lint_name>::check_namespace(namespace, &mut self.buffer));*; }
            fn check_item(&mut self, item: &Item) { $(<$lint_name>::check_item(item, &mut self.buffer));*; }
            fn check_attr(&mut self, attr: &Attr) { $(<$lint_name>::check_attr(attr, &mut self.buffer));*; }
            fn check_visibility(&mut self, visibility: &Visibility) { $(<$lint_name>::check_visibility(visibility, &mut self.buffer));*; }
            fn check_ty_def(&mut self, def: &TyDef) { $(<$lint_name>::check_ty_def(def, &mut self.buffer));*; }
            fn check_callable_decl(&mut self, decl: &CallableDecl) { $(<$lint_name>::check_callable_decl(decl, &mut self.buffer));*; }
            fn check_spec_decl(&mut self, decl: &SpecDecl) { $(<$lint_name>::check_spec_decl(decl, &mut self.buffer));*; }
            fn check_functor_expr(&mut self, expr: &FunctorExpr) { $(<$lint_name>::check_functor_expr(expr, &mut self.buffer));*; }
            fn check_ty(&mut self, ty: &Ty) { $(<$lint_name>::check_ty(ty, &mut self.buffer));*; }
            fn check_block(&mut self, block: &Block) { $(<$lint_name>::check_block(block, &mut self.buffer));*; }
            fn check_stmt(&mut self, stmt: &Stmt) { $(<$lint_name>::check_stmt(stmt, &mut self.buffer));*; }
            fn check_expr(&mut self, expr: &Expr) { $(<$lint_name>::check_expr(expr, &mut self.buffer));*; }
            fn check_pat(&mut self, pat: &Pat) { $(<$lint_name>::check_pat(pat, &mut self.buffer));*; }
            fn check_qubit_init(&mut self, init: &QubitInit) { $(<$lint_name>::check_qubit_init(init, &mut self.buffer));*; }
            fn check_path(&mut self, path: &Path) { $(<$lint_name>::check_path(path, &mut self.buffer));*; }
            fn check_ident(&mut self, ident: &Ident) { $(<$lint_name>::check_ident(ident, &mut self.buffer));*; }
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

            fn visit_visibility(&mut self, visibility: &'a Visibility) {
                self.check_visibility(visibility);
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

            fn visit_ident(&mut self, ident: &'a Ident) {
                self.check_ident(ident);
            }
        }
    }
}

pub(crate) use declare_ast_lints;
