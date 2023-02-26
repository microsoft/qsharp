// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::{
    ast::{
        Attr, Block, CallableDecl, Expr, FunctorExpr, Ident, Item, Namespace, NodeId, Package, Pat,
        Path, QubitInit, SpecDecl, Stmt, Ty, TyDef,
    },
    mut_visit::{self, MutVisitor},
};

pub(super) struct Assigner {
    next_id: NodeId,
}

impl Assigner {
    pub(super) fn new() -> Self {
        Self {
            next_id: NodeId::default(),
        }
    }

    fn next_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id = self.next_id.successor();
        id
    }
}

impl MutVisitor for Assigner {
    fn visit_package(&mut self, package: &mut Package) {
        package.id = self.next_id();
        mut_visit::walk_package(self, package);
    }

    fn visit_namespace(&mut self, namespace: &mut Namespace) {
        namespace.id = self.next_id();
        mut_visit::walk_namespace(self, namespace);
    }

    fn visit_item(&mut self, item: &mut Item) {
        item.id = self.next_id();
        mut_visit::walk_item(self, item);
    }

    fn visit_attr(&mut self, attr: &mut Attr) {
        attr.id = self.next_id();
        mut_visit::walk_attr(self, attr);
    }

    fn visit_ty_def(&mut self, def: &mut TyDef) {
        def.id = self.next_id();
        mut_visit::walk_ty_def(self, def);
    }

    fn visit_callable_decl(&mut self, decl: &mut CallableDecl) {
        decl.id = self.next_id();
        mut_visit::walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &mut SpecDecl) {
        decl.id = self.next_id();
        mut_visit::walk_spec_decl(self, decl);
    }

    fn visit_functor_expr(&mut self, expr: &mut FunctorExpr) {
        expr.id = self.next_id();
        mut_visit::walk_functor_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &mut Ty) {
        ty.id = self.next_id();
        mut_visit::walk_ty(self, ty);
    }

    fn visit_block(&mut self, block: &mut Block) {
        block.id = self.next_id();
        mut_visit::walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        stmt.id = self.next_id();
        mut_visit::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        expr.id = self.next_id();
        mut_visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &mut Pat) {
        pat.id = self.next_id();
        mut_visit::walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &mut QubitInit) {
        init.id = self.next_id();
        mut_visit::walk_qubit_init(self, init);
    }

    fn visit_path(&mut self, path: &mut Path) {
        path.id = self.next_id();
        mut_visit::walk_path(self, path);
    }

    fn visit_ident(&mut self, ident: &mut Ident) {
        ident.id = self.next_id();
    }
}
