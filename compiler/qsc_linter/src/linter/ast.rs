use qsc_ast::{
    ast::{
        Attr, Block, CallableDecl, Expr, FunctorExpr, Ident, Item, Namespace, Package, Pat, Path,
        QubitInit, SpecDecl, Stmt, Ty, TyDef, Visibility,
    },
    visit::{self, Visitor},
};

#[allow(unused_variables)]
pub trait LintPass<'a> {
    fn check_attr(&mut self, attr: &'a Attr) {}
    fn check_block(&mut self, block: &'a Block) {}
    fn check_callable_decl(&mut self, callable_decl: &'a CallableDecl) {}
    fn check_expr(&mut self, expr: &'a Expr) {}
    fn check_functor_expr(&mut self, functor_expr: &'a FunctorExpr) {}
    fn check_ident(&mut self, _: &'a Ident) {}
    fn check_item(&mut self, item: &'a Item) {}
    fn check_namespace(&mut self, namespace: &'a Namespace) {}
    fn check_package(&mut self, package: &'a Package) {}
    fn check_pat(&mut self, pat: &'a Pat) {}
    fn check_path(&mut self, path: &'a Path) {}
    fn check_qubit_init(&mut self, qubit_init: &'a QubitInit) {}
    fn check_spec_decl(&mut self, spec_decl: &'a SpecDecl) {}
    fn check_stmt(&mut self, stmt: &'a Stmt) {}
    fn check_ty(&mut self, ty: &'a Ty) {}
    fn check_ty_def(&mut self, ty_def: &'a TyDef) {}
    fn check_visibility(&mut self, visibility: &'a Visibility) {}
}

pub struct DummyWrapper<'a>(pub &'a mut dyn LintPass<'a>);

// impl<'a> DummyWrapper<'a> {
//     pub fn new(x: &'a mut dyn LintPass<'a>) -> Self {
//         Self(x)
//     }
// }

impl<'a> Visitor<'a> for DummyWrapper<'a> {
    fn visit_package(&mut self, package: &'a Package) {
        self.0.check_package(package);
        visit::walk_package(self, package);
    }

    fn visit_namespace(&mut self, namespace: &'a Namespace) {
        self.0.check_namespace(namespace);
        visit::walk_namespace(self, namespace);
    }

    fn visit_item(&mut self, item: &'a Item) {
        self.0.check_item(item);
        visit::walk_item(self, item);
    }

    fn visit_attr(&mut self, attr: &'a Attr) {
        self.0.check_attr(attr);
        visit::walk_attr(self, attr);
    }

    fn visit_visibility(&mut self, visibility: &'a Visibility) {
        self.0.check_visibility(visibility);
    }

    fn visit_ty_def(&mut self, def: &'a TyDef) {
        self.0.check_ty_def(def);
        visit::walk_ty_def(self, def);
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        self.0.check_callable_decl(decl);
        visit::walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
        self.0.check_spec_decl(decl);
        visit::walk_spec_decl(self, decl);
    }

    fn visit_functor_expr(&mut self, expr: &'a FunctorExpr) {
        self.0.check_functor_expr(expr);
        visit::walk_functor_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &'a Ty) {
        self.0.check_ty(ty);
        visit::walk_ty(self, ty);
    }

    fn visit_block(&mut self, block: &'a Block) {
        self.0.check_block(block);
        visit::walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        self.0.check_stmt(stmt);
        visit::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        self.0.check_expr(expr);
        visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &'a Pat) {
        self.0.check_pat(pat);
        visit::walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &'a QubitInit) {
        self.0.check_qubit_init(init);
        visit::walk_qubit_init(self, init);
    }

    fn visit_path(&mut self, path: &'a Path) {
        self.0.check_path(path);
        visit::walk_path(self, path);
    }

    fn visit_ident(&mut self, ident: &'a Ident) {
        self.0.check_ident(ident);
    }
}
