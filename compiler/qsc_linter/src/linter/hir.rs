use qsc_hir::{
    hir::{Block, CallableDecl, Expr, Ident, Item, Package, Pat, QubitInit, SpecDecl, Stmt},
    visit::{self, Visitor},
};

use crate::Lint;

/// The entry point to the HIR linter. It takes a [`qsc_hir::hir::Package`]
/// as input and outputs a [`Vec<Lint>`](Lint).
#[must_use]
pub fn run_hir_lints(package: &Package) -> Vec<Lint> {
    let mut lints = CombinedHirLints::new();

    for stmt in &package.stmts {
        lints.visit_stmt(stmt);
    }

    lints.buffer
}

/// Combined HIR lints for speed.
pub(crate) struct CombinedHirLints {
    buffer: Vec<Lint>,
}

#[allow(unused_variables, clippy::unused_self)]
impl CombinedHirLints {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    fn check_package(&self, package: &Package) {}

    fn check_item(&self, item: &Item) {}

    fn check_callable_decl(&self, decl: &CallableDecl) {}

    fn check_spec_decl(&self, decl: &SpecDecl) {}

    fn check_block(&self, block: &Block) {}

    fn check_stmt(&self, stmt: &Stmt) {}

    fn check_expr(&mut self, expr: &Expr) {}

    fn check_pat(&self, pat: &Pat) {}

    fn check_qubit_init(&self, init: &QubitInit) {}

    fn check_ident(&self, ident: &Ident) {}
}

impl<'a> Visitor<'a> for CombinedHirLints {
    fn visit_package(&mut self, package: &'a Package) {
        self.check_package(package);
        visit::walk_package(self, package);
    }

    fn visit_item(&mut self, item: &'a Item) {
        self.check_item(item);
        visit::walk_item(self, item);
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        self.check_callable_decl(decl);
        visit::walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
        self.check_spec_decl(decl);
        visit::walk_spec_decl(self, decl);
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

    fn visit_ident(&mut self, ident: &'a Ident) {
        self.check_ident(ident);
    }
}

/// Represents a lint pass in the HIR.
/// You only need to implement the `check_*` function relevant to your lint.
/// The trait provides default empty implementations for the rest of the methods,
/// which will be optimized to a no-op by the rust compiler.
#[allow(unused_variables)]
pub(crate) trait HirLintPass {
    fn check_block(block: &Block, buffer: &mut Vec<Lint>) {}
    fn check_callable_decl(callable_decl: &CallableDecl, buffer: &mut Vec<Lint>) {}
    fn check_expr(expr: &Expr, buffer: &mut Vec<Lint>) {}
    fn check_ident(_: &Ident, buffer: &mut Vec<Lint>) {}
    fn check_item(item: &Item, buffer: &mut Vec<Lint>) {}
    fn check_package(package: &Package, buffer: &mut Vec<Lint>) {}
    fn check_pat(pat: &Pat, buffer: &mut Vec<Lint>) {}
    fn check_qubit_init(qubit_init: &QubitInit, buffer: &mut Vec<Lint>) {}
    fn check_spec_decl(spec_decl: &SpecDecl, buffer: &mut Vec<Lint>) {}
    fn check_stmt(stmt: &Stmt, buffer: &mut Vec<Lint>) {}
}
