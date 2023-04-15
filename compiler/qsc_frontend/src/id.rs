// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::{
    ast::{
        Attr, Block, CallableDecl, Expr, FunctorExpr, Ident, Item, Namespace, NodeId, Package, Pat,
        Path, QubitInit, SpecDecl, Stmt, Ty, TyDef,
    },
    mut_visit::{self, MutVisitor},
};
use qsc_hir::{
    hir,
    mut_visit::{self as hir_mut_visit, MutVisitor as HirMutVisitor},
};

#[derive(Debug)]
pub struct AstAssigner {
    next_id: NodeId,
}

impl AstAssigner {
    pub(super) fn new() -> Self {
        Self {
            next_id: NodeId::zero(),
        }
    }

    pub(super) fn next_id(&mut self) -> NodeId {
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

impl MutVisitor for AstAssigner {
    fn visit_package(&mut self, package: &mut Package) {
        self.assign(&mut package.id);
        mut_visit::walk_package(self, package);
    }

    fn visit_namespace(&mut self, namespace: &mut Namespace) {
        self.assign(&mut namespace.id);
        mut_visit::walk_namespace(self, namespace);
    }

    fn visit_item(&mut self, item: &mut Item) {
        self.assign(&mut item.id);
        if let Some(visibility) = &mut item.meta.visibility {
            self.assign(&mut visibility.id);
        }
        mut_visit::walk_item(self, item);
    }

    fn visit_attr(&mut self, attr: &mut Attr) {
        self.assign(&mut attr.id);
        mut_visit::walk_attr(self, attr);
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

    fn visit_path(&mut self, path: &mut Path) {
        self.assign(&mut path.id);
        mut_visit::walk_path(self, path);
    }

    fn visit_ident(&mut self, ident: &mut Ident) {
        self.assign(&mut ident.id);
    }
}

#[derive(Debug)]
pub struct HirAssigner {
    next_id: hir::NodeId,
}

impl HirAssigner {
    pub(super) fn new() -> Self {
        Self {
            next_id: hir::NodeId::zero(),
        }
    }

    pub(super) fn next_id(&mut self) -> hir::NodeId {
        let id = self.next_id;
        self.next_id = self.next_id.successor();
        id
    }

    fn assign(&mut self, id: &mut hir::NodeId) {
        if id.is_placeholder() {
            *id = self.next_id();
        }
    }
}

impl HirMutVisitor for HirAssigner {
    fn visit_package(&mut self, package: &mut hir::Package) {
        self.assign(&mut package.id);
        hir_mut_visit::walk_package(self, package);
    }

    fn visit_namespace(&mut self, namespace: &mut hir::Namespace) {
        self.assign(&mut namespace.id);
        hir_mut_visit::walk_namespace(self, namespace);
    }

    fn visit_item(&mut self, item: &mut hir::Item) {
        self.assign(&mut item.id);
        if let Some(visibility) = &mut item.meta.visibility {
            self.assign(&mut visibility.id);
        }
        hir_mut_visit::walk_item(self, item);
    }

    fn visit_attr(&mut self, attr: &mut hir::Attr) {
        self.assign(&mut attr.id);
        hir_mut_visit::walk_attr(self, attr);
    }

    fn visit_ty_def(&mut self, def: &mut hir::TyDef) {
        self.assign(&mut def.id);
        hir_mut_visit::walk_ty_def(self, def);
    }

    fn visit_callable_decl(&mut self, decl: &mut hir::CallableDecl) {
        self.assign(&mut decl.id);
        hir_mut_visit::walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &mut hir::SpecDecl) {
        self.assign(&mut decl.id);
        hir_mut_visit::walk_spec_decl(self, decl);
    }

    fn visit_functor_expr(&mut self, expr: &mut hir::FunctorExpr) {
        self.assign(&mut expr.id);
        hir_mut_visit::walk_functor_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &mut hir::Ty) {
        self.assign(&mut ty.id);
        hir_mut_visit::walk_ty(self, ty);
    }

    fn visit_block(&mut self, block: &mut hir::Block) {
        self.assign(&mut block.id);
        hir_mut_visit::walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &mut hir::Stmt) {
        self.assign(&mut stmt.id);
        hir_mut_visit::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &mut hir::Expr) {
        self.assign(&mut expr.id);
        hir_mut_visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &mut hir::Pat) {
        self.assign(&mut pat.id);
        hir_mut_visit::walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &mut hir::QubitInit) {
        self.assign(&mut init.id);
        hir_mut_visit::walk_qubit_init(self, init);
    }

    fn visit_path(&mut self, path: &mut hir::Path) {
        self.assign(&mut path.id);
        hir_mut_visit::walk_path(self, path);
    }

    fn visit_ident(&mut self, ident: &mut hir::Ident) {
        self.assign(&mut ident.id);
    }
}
