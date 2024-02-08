use qsc_hir::{
    hir::{Block, CallableDecl, Expr, Ident, Item, Package, Pat, QubitInit, SpecDecl, Stmt},
    visit::{self, Visitor},
};

use crate::Lint;

/// The entry point to the HIR linter. It takes a [`qsc_hir::hir::Package`]
/// as input and outputs a [`Vec<Lint>`](Lint).
#[must_use]
pub fn run_hir_lints(package: &Package) -> Vec<Lint> {
    let mut lints = CombinedHirLints;

    for stmt in &package.stmts {
        lints.visit_stmt(stmt);
    }

    super::drain().collect()
}

/// Combined HIR lints for speed.
pub(crate) struct CombinedHirLints;

impl HirLintPass for CombinedHirLints {}

impl<'a> Visitor<'a> for CombinedHirLints {
    fn visit_package(&mut self, package: &'a Package) {
        CombinedHirLints::check_package(package);
        visit::walk_package(self, package);
    }

    fn visit_item(&mut self, item: &'a Item) {
        CombinedHirLints::check_item(item);
        visit::walk_item(self, item);
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        CombinedHirLints::check_callable_decl(decl);
        visit::walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
        CombinedHirLints::check_spec_decl(decl);
        visit::walk_spec_decl(self, decl);
    }

    fn visit_block(&mut self, block: &'a Block) {
        CombinedHirLints::check_block(block);
        visit::walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        CombinedHirLints::check_stmt(stmt);
        visit::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        CombinedHirLints::check_expr(expr);
        visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &'a Pat) {
        CombinedHirLints::check_pat(pat);
        visit::walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &'a QubitInit) {
        CombinedHirLints::check_qubit_init(init);
        visit::walk_qubit_init(self, init);
    }

    fn visit_ident(&mut self, ident: &'a Ident) {
        CombinedHirLints::check_ident(ident);
    }
}

#[allow(unused_variables)]
pub(crate) trait HirLintPass {
    fn check_block(block: &Block) {}
    fn check_callable_decl(callable_decl: &CallableDecl) {}
    fn check_expr(expr: &Expr) {}
    fn check_ident(_: &Ident) {}
    fn check_item(item: &Item) {}
    fn check_package(package: &Package) {}
    fn check_pat(pat: &Pat) {}
    fn check_qubit_init(qubit_init: &QubitInit) {}
    fn check_spec_decl(spec_decl: &SpecDecl) {}
    fn check_stmt(stmt: &Stmt) {}
}
