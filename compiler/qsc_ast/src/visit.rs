// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::ast::{
    Attr, Block, CallableBody, CallableDecl, Expr, ExprKind, FunctorExpr, FunctorExprKind, Ident,
    Item, ItemKind, Namespace, Package, Pat, PatKind, Path, QubitInit, QubitInitKind, SpecBody,
    SpecDecl, Stmt, StmtKind, Ty, TyDef, TyKind,
};

pub trait Visitor: Sized {
    fn visit_package(&mut self, package: &Package) {
        walk_package(self, package);
    }

    fn visit_namespace(&mut self, namespace: &Namespace) {
        walk_namespace(self, namespace);
    }

    fn visit_item(&mut self, item: &Item) {
        walk_item(self, item);
    }

    fn visit_attr(&mut self, attr: &Attr) {
        walk_attr(self, attr);
    }

    fn visit_ty_def(&mut self, def: &TyDef) {
        walk_ty_def(self, def);
    }

    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &SpecDecl) {
        walk_spec_decl(self, decl);
    }

    fn visit_functor_expr(&mut self, expr: &FunctorExpr) {
        walk_functor_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &Ty) {
        walk_ty(self, ty);
    }

    fn visit_block(&mut self, block: &Block) {
        walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &Pat) {
        walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &QubitInit) {
        walk_qubit_init(self, init);
    }

    fn visit_path(&mut self, _: &Path) {}

    fn visit_ident(&mut self, _: &Ident) {}
}

pub fn walk_package(vis: &mut impl Visitor, package: &Package) {
    package
        .namespaces
        .iter()
        .for_each(|n| vis.visit_namespace(n));
}

pub fn walk_namespace(vis: &mut impl Visitor, namespace: &Namespace) {
    vis.visit_ident(&namespace.name);
    namespace.items.iter().for_each(|i| vis.visit_item(i));
}

pub fn walk_item(vis: &mut impl Visitor, item: &Item) {
    match &item.kind {
        ItemKind::Open(ns, alias) => {
            vis.visit_ident(ns);
            alias.iter().for_each(|a| vis.visit_ident(a));
        }
        ItemKind::Type(meta, ident, def) => {
            meta.attrs.iter().for_each(|a| vis.visit_attr(a));
            vis.visit_ident(ident);
            vis.visit_ty_def(def);
        }
        ItemKind::Callable(meta, decl) => {
            meta.attrs.iter().for_each(|a| vis.visit_attr(a));
            vis.visit_callable_decl(decl);
        }
    }
}

pub fn walk_attr(vis: &mut impl Visitor, attr: &Attr) {
    vis.visit_path(&attr.name);
    vis.visit_expr(&attr.arg);
}

pub fn walk_ty_def(vis: &mut impl Visitor, def: &TyDef) {
    match def {
        TyDef::Field(name, ty) => {
            name.iter().for_each(|n| vis.visit_ident(n));
            vis.visit_ty(ty);
        }
        TyDef::Tuple(defs) => defs.iter().for_each(|d| vis.visit_ty_def(d)),
    }
}

pub fn walk_callable_decl(vis: &mut impl Visitor, decl: &CallableDecl) {
    vis.visit_ident(&decl.name);
    decl.ty_params.iter().for_each(|p| vis.visit_ident(p));
    vis.visit_pat(&decl.input);
    vis.visit_ty(&decl.output);
    decl.functors.iter().for_each(|f| vis.visit_functor_expr(f));
    match &decl.body {
        CallableBody::Block(block) => vis.visit_block(block),
        CallableBody::Specs(specs) => specs.iter().for_each(|s| vis.visit_spec_decl(s)),
    }
}

pub fn walk_spec_decl(vis: &mut impl Visitor, decl: &SpecDecl) {
    match &decl.body {
        SpecBody::Gen(_) => {}
        SpecBody::Impl(pat, block) => {
            vis.visit_pat(pat);
            vis.visit_block(block);
        }
    }
}

pub fn walk_functor_expr(vis: &mut impl Visitor, expr: &FunctorExpr) {
    match &expr.kind {
        FunctorExprKind::BinOp(_, lhs, rhs) => {
            vis.visit_functor_expr(lhs);
            vis.visit_functor_expr(rhs);
        }
        FunctorExprKind::Lit(_) => {}
        FunctorExprKind::Paren(expr) => vis.visit_functor_expr(expr),
    }
}

pub fn walk_ty(vis: &mut impl Visitor, ty: &Ty) {
    match &ty.kind {
        TyKind::App(ty, tys) => {
            vis.visit_ty(ty);
            tys.iter().for_each(|t| vis.visit_ty(t));
        }
        TyKind::Arrow(_, lhs, rhs, functors) => {
            vis.visit_ty(lhs);
            vis.visit_ty(rhs);
            functors.iter().for_each(|f| vis.visit_functor_expr(f));
        }
        TyKind::Paren(ty) => vis.visit_ty(ty),
        TyKind::Path(path) => vis.visit_path(path),
        TyKind::Tuple(tys) => tys.iter().for_each(|t| vis.visit_ty(t)),
        TyKind::Hole | TyKind::Prim(_) | TyKind::Var(_) => {}
    }
}

pub fn walk_block(vis: &mut impl Visitor, block: &Block) {
    block.stmts.iter().for_each(|s| vis.visit_stmt(s));
}

pub fn walk_stmt(vis: &mut impl Visitor, stmt: &Stmt) {
    match &stmt.kind {
        StmtKind::Borrow(pat, init, block) | StmtKind::Use(pat, init, block) => {
            vis.visit_pat(pat);
            vis.visit_qubit_init(init);
            block.iter().for_each(|b| vis.visit_block(b));
        }
        StmtKind::Expr(expr) | StmtKind::Semi(expr) => vis.visit_expr(expr),
        StmtKind::Let(pat, value) | StmtKind::Mutable(pat, value) => {
            vis.visit_pat(pat);
            vis.visit_expr(value);
        }
    }
}

pub fn walk_expr(vis: &mut impl Visitor, expr: &Expr) {
    match &expr.kind {
        ExprKind::Array(exprs) => exprs.iter().for_each(|e| vis.visit_expr(e)),
        ExprKind::ArrayRepeat(item, size) => {
            vis.visit_expr(item);
            vis.visit_expr(size);
        }
        ExprKind::Assign(lhs, rhs)
        | ExprKind::AssignOp(_, lhs, rhs)
        | ExprKind::BinOp(_, lhs, rhs) => {
            vis.visit_expr(lhs);
            vis.visit_expr(rhs);
        }
        ExprKind::AssignUpdate(record, index, value) => {
            vis.visit_expr(record);
            vis.visit_expr(index);
            vis.visit_expr(value);
        }
        ExprKind::Block(block) => vis.visit_block(block),
        ExprKind::Call(callee, arg) => {
            vis.visit_expr(callee);
            vis.visit_expr(arg);
        }
        ExprKind::Conjugate(within, apply) => {
            vis.visit_block(within);
            vis.visit_block(apply);
        }
        ExprKind::Fail(msg) => vis.visit_expr(msg),
        ExprKind::Field(record, name) => {
            vis.visit_expr(record);
            vis.visit_ident(name);
        }
        ExprKind::For(pat, iter, block) => {
            vis.visit_pat(pat);
            vis.visit_expr(iter);
            vis.visit_block(block);
        }
        ExprKind::If(cond, body, otherwise) => {
            vis.visit_expr(cond);
            vis.visit_block(body);
            otherwise.iter().for_each(|e| vis.visit_expr(e));
        }
        ExprKind::Index(array, index) => {
            vis.visit_expr(array);
            vis.visit_expr(index);
        }
        ExprKind::Lambda(_, pat, expr) => {
            vis.visit_pat(pat);
            vis.visit_expr(expr);
        }
        ExprKind::Paren(expr) | ExprKind::Return(expr) | ExprKind::UnOp(_, expr) => {
            vis.visit_expr(expr);
        }
        ExprKind::Path(path) => vis.visit_path(path),
        ExprKind::Range(start, step, stop) => {
            start.iter().for_each(|s| vis.visit_expr(s));
            step.iter().for_each(|s| vis.visit_expr(s));
            stop.iter().for_each(|s| vis.visit_expr(s));
        }
        ExprKind::Repeat(body, until, fixup) => {
            vis.visit_block(body);
            vis.visit_expr(until);
            fixup.iter().for_each(|f| vis.visit_block(f));
        }
        ExprKind::TernOp(_, e1, e2, e3) => {
            vis.visit_expr(e1);
            vis.visit_expr(e2);
            vis.visit_expr(e3);
        }
        ExprKind::Tuple(exprs) => exprs.iter().for_each(|e| vis.visit_expr(e)),
        ExprKind::While(cond, block) => {
            vis.visit_expr(cond);
            vis.visit_block(block);
        }
        ExprKind::Hole | ExprKind::Lit(_) => {}
    }
}

pub fn walk_pat(vis: &mut impl Visitor, pat: &Pat) {
    match &pat.kind {
        PatKind::Bind(name, ty) => {
            vis.visit_ident(name);
            ty.iter().for_each(|t| vis.visit_ty(t));
        }
        PatKind::Discard(ty) => ty.iter().for_each(|t| vis.visit_ty(t)),
        PatKind::Elided => {}
        PatKind::Paren(pat) => vis.visit_pat(pat),
        PatKind::Tuple(pats) => pats.iter().for_each(|p| vis.visit_pat(p)),
    }
}

pub fn walk_qubit_init(vis: &mut impl Visitor, init: &QubitInit) {
    match &init.kind {
        QubitInitKind::Array(len) => vis.visit_expr(len),
        QubitInitKind::Paren(init) => vis.visit_qubit_init(init),
        QubitInitKind::Single => {}
        QubitInitKind::Tuple(inits) => inits.iter().for_each(|i| vis.visit_qubit_init(i)),
    }
}
