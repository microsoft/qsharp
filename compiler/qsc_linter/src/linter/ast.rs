use qsc_ast::{
    ast::{
        Attr, Block, CallableDecl, Expr, FunctorExpr, Ident, Item, Namespace, Package, Pat, Path,
        QubitInit, SpecDecl, Stmt, TopLevelNode, Ty, TyDef, Visibility,
    },
    visit::{self, Visitor},
};

use crate::{
    lints::ast::{DivisionByZero, DoubleParens},
    Lint,
};

/// The entry point to the AST linter. It takes a [`qsc_ast::ast::Package`]
/// as input and outputs a Vec<[`Lint`]>.
#[must_use]
pub fn run_ast_lints(package: &qsc_ast::ast::Package) -> Vec<Lint> {
    let mut lints = CombinedAstLints;

    for node in package.nodes.iter() {
        match node {
            TopLevelNode::Namespace(namespace) => {
                lints.visit_namespace(namespace);
            }
            TopLevelNode::Stmt(stmt) => {
                lints.visit_stmt(stmt);
            }
        }
    }

    super::drain().collect()
}

/// Combined AST lints for speed.
pub(crate) struct CombinedAstLints;

impl AstLintPass for CombinedAstLints {
    fn check_expr(expr: &Expr) {
        DoubleParens::check_expr(expr);
        DivisionByZero::check_expr(expr);
    }
}

#[allow(unused_variables)]
pub(crate) trait AstLintPass {
    fn check_attr(attr: &Attr) {}
    fn check_block(block: &Block) {}
    fn check_callable_decl(callable_decl: &CallableDecl) {}
    fn check_expr(expr: &Expr) {}
    fn check_functor_expr(functor_expr: &FunctorExpr) {}
    fn check_ident(_: &Ident) {}
    fn check_item(item: &Item) {}
    fn check_namespace(namespace: &Namespace) {}
    fn check_package(package: &Package) {}
    fn check_pat(pat: &Pat) {}
    fn check_path(path: &Path) {}
    fn check_qubit_init(qubit_init: &QubitInit) {}
    fn check_spec_decl(spec_decl: &SpecDecl) {}
    fn check_stmt(stmt: &Stmt) {}
    fn check_ty(ty: &Ty) {}
    fn check_ty_def(ty_def: &TyDef) {}
    fn check_visibility(visibility: &Visibility) {}
}

impl<'a> Visitor<'a> for CombinedAstLints {
    fn visit_package(&mut self, package: &'a Package) {
        CombinedAstLints::check_package(package);
        visit::walk_package(self, package);
    }

    fn visit_namespace(&mut self, namespace: &'a Namespace) {
        CombinedAstLints::check_namespace(namespace);
        visit::walk_namespace(self, namespace);
    }

    fn visit_item(&mut self, item: &'a Item) {
        CombinedAstLints::check_item(item);
        visit::walk_item(self, item);
    }

    fn visit_attr(&mut self, attr: &'a Attr) {
        CombinedAstLints::check_attr(attr);
        visit::walk_attr(self, attr);
    }

    fn visit_visibility(&mut self, visibility: &'a Visibility) {
        CombinedAstLints::check_visibility(visibility);
    }

    fn visit_ty_def(&mut self, def: &'a TyDef) {
        CombinedAstLints::check_ty_def(def);
        visit::walk_ty_def(self, def);
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        CombinedAstLints::check_callable_decl(decl);
        visit::walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
        CombinedAstLints::check_spec_decl(decl);
        visit::walk_spec_decl(self, decl);
    }

    fn visit_functor_expr(&mut self, expr: &'a FunctorExpr) {
        CombinedAstLints::check_functor_expr(expr);
        visit::walk_functor_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &'a Ty) {
        CombinedAstLints::check_ty(ty);
        visit::walk_ty(self, ty);
    }

    fn visit_block(&mut self, block: &'a Block) {
        CombinedAstLints::check_block(block);
        visit::walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        CombinedAstLints::check_stmt(stmt);
        visit::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        CombinedAstLints::check_expr(expr);
        visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &'a Pat) {
        CombinedAstLints::check_pat(pat);
        visit::walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &'a QubitInit) {
        CombinedAstLints::check_qubit_init(init);
        visit::walk_qubit_init(self, init);
    }

    fn visit_path(&mut self, path: &'a Path) {
        CombinedAstLints::check_path(path);
        visit::walk_path(self, path);
    }

    fn visit_ident(&mut self, ident: &'a Ident) {
        CombinedAstLints::check_ident(ident);
    }
}
