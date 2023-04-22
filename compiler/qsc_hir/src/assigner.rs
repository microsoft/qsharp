// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    hir::{
        Attr, Block, CallableDecl, Expr, FunctorExpr, Ident, Item, NodeId, Package, Pat, QubitInit,
        SpecDecl, Stmt, Ty, TyDef, Visibility,
    },
    mut_visit::{self, MutVisitor},
};

#[derive(Debug)]
pub struct Assigner {
    next_id: NodeId,
}

impl Assigner {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: NodeId::zero(),
        }
    }

    pub fn next_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id = self.next_id.successor();
        id
    }

    fn assign(&mut self, id: &mut NodeId) {
        if id.is_placeholder() {
            *id = self.next_id();
        }
    }
}

impl Default for Assigner {
    fn default() -> Self {
        Self::new()
    }
}

impl MutVisitor for Assigner {
    fn visit_package(&mut self, package: &mut Package) {
        self.assign(&mut package.id);
        mut_visit::walk_package(self, package);
    }

    fn visit_item(&mut self, item: &mut Item) {
        self.assign(&mut item.id);
        mut_visit::walk_item(self, item);
    }

    fn visit_attr(&mut self, attr: &mut Attr) {
        self.assign(&mut attr.id);
        mut_visit::walk_attr(self, attr);
    }

    fn visit_visibility(&mut self, visibility: &mut Visibility) {
        self.assign(&mut visibility.id);
    }

    fn visit_ty_def(&mut self, def: &mut TyDef) {
        self.assign(&mut def.id);
        mut_visit::walk_ty_def(self, def);
    }

    fn visit_callable_decl(&mut self, decl: &mut CallableDecl) {
        self.assign(&mut decl.id);
        mut_visit::walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &mut SpecDecl) {
        self.assign(&mut decl.id);
        mut_visit::walk_spec_decl(self, decl);
    }

    fn visit_functor_expr(&mut self, expr: &mut FunctorExpr) {
        self.assign(&mut expr.id);
        mut_visit::walk_functor_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &mut Ty) {
        self.assign(&mut ty.id);
        mut_visit::walk_ty(self, ty);
    }

    fn visit_block(&mut self, block: &mut Block) {
        self.assign(&mut block.id);
        mut_visit::walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        self.assign(&mut stmt.id);
        mut_visit::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        self.assign(&mut expr.id);
        mut_visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &mut Pat) {
        self.assign(&mut pat.id);
        mut_visit::walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &mut QubitInit) {
        self.assign(&mut init.id);
        mut_visit::walk_qubit_init(self, init);
    }

    fn visit_ident(&mut self, ident: &mut Ident) {
        self.assign(&mut ident.id);
    }
}
