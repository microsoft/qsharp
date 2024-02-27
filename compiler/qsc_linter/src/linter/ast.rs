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
/// as input and outputs a [`Vec<Lint>`](Lint).
#[must_use]
pub fn run_ast_lints(package: &qsc_ast::ast::Package) -> Vec<Lint> {
    let mut lints = CombinedAstLints::new();

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

    lints.buffer
}

/// Combined AST lints for speed.
pub(crate) struct CombinedAstLints {
    buffer: Vec<Lint>,
}

#[allow(unused_variables, clippy::unused_self)]
impl CombinedAstLints {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    fn check_package(&self, package: &Package) {}

    fn check_namespace(&self, namespace: &Namespace) {}

    fn check_item(&self, item: &Item) {}

    fn check_attr(&self, attr: &Attr) {}

    fn check_visibility(&self, visibility: &Visibility) {}

    fn check_ty_def(&self, def: &TyDef) {}

    fn check_callable_decl(&self, decl: &CallableDecl) {}

    fn check_spec_decl(&self, decl: &SpecDecl) {}

    fn check_functor_expr(&self, expr: &FunctorExpr) {}

    fn check_ty(&self, ty: &Ty) {}

    fn check_block(&self, block: &Block) {}

    fn check_stmt(&self, stmt: &Stmt) {}

    fn check_expr(&mut self, expr: &Expr) {
        DoubleParens::check_expr(expr, &mut self.buffer);
        DivisionByZero::check_expr(expr, &mut self.buffer);
    }

    fn check_pat(&self, pat: &Pat) {}

    fn check_qubit_init(&self, init: &QubitInit) {}

    fn check_path(&self, path: &Path) {}

    fn check_ident(&self, ident: &Ident) {}
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

/// Represents a lint pass in the AST.
/// You only need to implement the `check_*` function relevant to your lint.
/// The trait provides default empty implementations for the rest of the methods,
/// which will be optimized to a no-op by the rust compiler.
#[allow(unused_variables)]
pub(crate) trait AstLintPass {
    fn check_attr(attr: &Attr, buffer: &mut Vec<Lint>) {}
    fn check_block(block: &Block, buffer: &mut Vec<Lint>) {}
    fn check_callable_decl(callable_decl: &CallableDecl, buffer: &mut Vec<Lint>) {}
    fn check_expr(expr: &Expr, buffer: &mut Vec<Lint>) {}
    fn check_functor_expr(functor_expr: &FunctorExpr, buffer: &mut Vec<Lint>) {}
    fn check_ident(_: &Ident, buffer: &mut Vec<Lint>) {}
    fn check_item(item: &Item, buffer: &mut Vec<Lint>) {}
    fn check_namespace(namespace: &Namespace, buffer: &mut Vec<Lint>) {}
    fn check_package(package: &Package, buffer: &mut Vec<Lint>) {}
    fn check_pat(pat: &Pat, buffer: &mut Vec<Lint>) {}
    fn check_path(path: &Path, buffer: &mut Vec<Lint>) {}
    fn check_qubit_init(qubit_init: &QubitInit, buffer: &mut Vec<Lint>) {}
    fn check_spec_decl(spec_decl: &SpecDecl, buffer: &mut Vec<Lint>) {}
    fn check_stmt(stmt: &Stmt, buffer: &mut Vec<Lint>) {}
    fn check_ty(ty: &Ty, buffer: &mut Vec<Lint>) {}
    fn check_ty_def(ty_def: &TyDef, buffer: &mut Vec<Lint>) {}
    fn check_visibility(visibility: &Visibility, buffer: &mut Vec<Lint>) {}
}
