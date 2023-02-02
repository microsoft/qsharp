// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    Attribute, Block, CallBody, CallHeader, DeclInfo, Expr, ExprKind, FunctorExpr, Ident, Item,
    ItemKind, Namespace, Pat, PatKind, Path, Project, QubitInit, QubitInitKind, SpecBody, SpecDecl,
    Ty, TyDef, TyKind,
};

pub trait Visitor: Sized {
    fn visit_project(&mut self, project: &Project) {
        walk_project(self, project);
    }

    fn visit_namespace(&mut self, namespace: &Namespace) {
        walk_namespace(self, namespace);
    }

    fn visit_item(&mut self, item: &Item) {
        walk_item(self, item);
    }

    fn visit_decl_info(&mut self, info: &DeclInfo) {
        walk_decl_info(self, info);
    }

    fn visit_attribute(&mut self, attr: &Attribute) {
        walk_attribute(self, attr);
    }

    fn visit_ty_def(&mut self, def: &TyDef) {
        walk_ty_def(self, def);
    }

    fn visit_call_header(&mut self, header: &CallHeader) {
        walk_call_header(self, header);
    }

    fn visit_call_body(&mut self, body: &CallBody) {
        walk_call_body(self, body);
    }

    fn visit_spec_decl(&mut self, decl: &SpecDecl) {
        walk_spec_decl(self, decl);
    }

    fn visit_spec_body(&mut self, body: &SpecBody) {
        walk_spec_body(self, body);
    }

    fn visit_functor_expr(&mut self, expr: &FunctorExpr) {
        walk_functor_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &Ty) {
        walk_ty(self, ty);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }

    fn visit_block(&mut self, block: &Block) {
        walk_block(self, block);
    }

    fn visit_ident(&mut self, _: &Ident) {}

    fn visit_path(&mut self, _: &Path) {}

    fn visit_pat(&mut self, pat: &Pat) {
        walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &QubitInit) {
        walk_qubit_init(self, init);
    }
}

pub fn walk_project(vis: &mut impl Visitor, project: &Project) {
    project
        .namespaces
        .iter()
        .for_each(|n| vis.visit_namespace(n));
}

pub fn walk_namespace(vis: &mut impl Visitor, namespace: &Namespace) {
    vis.visit_path(&namespace.name);
    namespace.items.iter().for_each(|i| vis.visit_item(i));
}

pub fn walk_item(vis: &mut impl Visitor, item: &Item) {
    match &item.kind {
        ItemKind::Open(path, ident) => {
            vis.visit_path(path);
            vis.visit_ident(ident);
        }
        ItemKind::Type(info, ident, def) => {
            vis.visit_decl_info(info);
            vis.visit_ident(ident);
            vis.visit_ty_def(def);
        }
        ItemKind::Callable(info, header, body) => {
            vis.visit_decl_info(info);
            vis.visit_call_header(header);
            vis.visit_call_body(body);
        }
    }
}

pub fn walk_decl_info(vis: &mut impl Visitor, info: &DeclInfo) {
    info.attributes.iter().for_each(|a| vis.visit_attribute(a));
}

pub fn walk_attribute(vis: &mut impl Visitor, attr: &Attribute) {
    vis.visit_path(&attr.name);
    vis.visit_expr(&attr.arg);
}

pub fn walk_ty_def(vis: &mut impl Visitor, def: &TyDef) {
    match def {
        TyDef::Field(name, ty) => {
            name.iter().for_each(|n| vis.visit_ident(n));
            vis.visit_ty(ty);
        }
        TyDef::Tuple(defs) => {
            defs.iter().for_each(|d| vis.visit_ty_def(d));
        }
    }
}

pub fn walk_call_header(vis: &mut impl Visitor, header: &CallHeader) {
    vis.visit_ident(&header.name);
    header.ty_params.iter().for_each(|i| vis.visit_ident(i));
    vis.visit_pat(&header.input);
    vis.visit_ty(&header.output);
    vis.visit_functor_expr(&header.functors);
}

pub fn walk_call_body(vis: &mut impl Visitor, body: &CallBody) {
    match body {
        CallBody::Single(body) => vis.visit_spec_body(body),
        CallBody::Full(decls) => decls.iter().for_each(|d| vis.visit_spec_decl(d)),
    }
}

pub fn walk_spec_decl(vis: &mut impl Visitor, decl: &SpecDecl) {
    vis.visit_spec_body(&decl.body);
}

pub fn walk_spec_body(vis: &mut impl Visitor, body: &SpecBody) {
    match body {
        SpecBody::Impl(pat, block) => {
            vis.visit_pat(pat);
            vis.visit_block(block);
        }
        SpecBody::Gen(_) => {}
    }
}

pub fn walk_functor_expr(vis: &mut impl Visitor, expr: &FunctorExpr) {
    match expr {
        FunctorExpr::BinOp(_, lhs, rhs) => {
            vis.visit_functor_expr(lhs);
            vis.visit_functor_expr(rhs);
        }
        FunctorExpr::Lit(_) | FunctorExpr::Null => {}
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
            vis.visit_functor_expr(functors);
        }
        TyKind::Path(path) => vis.visit_path(path),
        TyKind::Tuple(tys) => {
            tys.iter().for_each(|t| vis.visit_ty(t));
        }
        TyKind::Hole | TyKind::Prim(_) | TyKind::Var(_) => {}
    }
}

pub fn walk_expr(vis: &mut impl Visitor, expr: &Expr) {
    match &expr.kind {
        ExprKind::Array(exprs) => {
            exprs.iter().for_each(|e| vis.visit_expr(e));
        }
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
        ExprKind::Fail(msg) => {
            vis.visit_expr(msg);
        }
        ExprKind::Field(record, name) => {
            vis.visit_expr(record);
            vis.visit_ident(name);
        }
        ExprKind::For(pat, iter, block) => {
            vis.visit_pat(pat);
            vis.visit_expr(iter);
            vis.visit_block(block);
        }
        ExprKind::If(branches, default) => {
            for (cond, block) in branches {
                vis.visit_expr(cond);
                vis.visit_block(block);
            }
            default.iter().for_each(|d| vis.visit_block(d));
        }
        ExprKind::Index(array, index) => {
            vis.visit_expr(array);
            vis.visit_expr(index);
        }
        ExprKind::Interp(_, exprs) => {
            exprs.iter().for_each(|e| vis.visit_expr(e));
        }
        ExprKind::Lambda(_, pat, expr) => {
            vis.visit_pat(pat);
            vis.visit_expr(expr);
        }
        ExprKind::Let(pat, value) => {
            vis.visit_pat(pat);
            vis.visit_expr(value);
        }
        ExprKind::Path(path) => vis.visit_path(path),
        ExprKind::Qubit(_, pat, init, block) => {
            vis.visit_pat(pat);
            vis.visit_qubit_init(init);
            block.iter().for_each(|b| vis.visit_block(b));
        }
        ExprKind::Range(start, step, end) => {
            vis.visit_expr(start);
            vis.visit_expr(step);
            vis.visit_expr(end);
        }
        ExprKind::Repeat(body, until, fixup) => {
            vis.visit_block(body);
            vis.visit_expr(until);
            fixup.iter().for_each(|f| vis.visit_block(f));
        }
        ExprKind::Return(expr) | ExprKind::UnOp(_, expr) => {
            vis.visit_expr(expr);
        }
        ExprKind::TernOp(_, e1, e2, e3) => {
            vis.visit_expr(e1);
            vis.visit_expr(e2);
            vis.visit_expr(e3);
        }
        ExprKind::Tuple(exprs) => {
            exprs.iter().for_each(|e| vis.visit_expr(e));
        }
        ExprKind::While(cond, block) => {
            vis.visit_expr(cond);
            vis.visit_block(block);
        }
        ExprKind::Hole | ExprKind::Lit(_) => {}
    }
}

pub fn walk_block(vis: &mut impl Visitor, block: &Block) {
    block.exprs.iter().for_each(|e| vis.visit_expr(e));
}

pub fn walk_pat(vis: &mut impl Visitor, pat: &Pat) {
    match &pat.kind {
        PatKind::Bind(_, name, ty) => {
            vis.visit_ident(name);
            vis.visit_ty(ty);
        }
        PatKind::Discard(ty) => {
            vis.visit_ty(ty);
        }
        PatKind::Tuple(pats) => {
            pats.iter().for_each(|p| vis.visit_pat(p));
        }
        PatKind::Omit => {}
    }
}

pub fn walk_qubit_init(vis: &mut impl Visitor, init: &QubitInit) {
    match &init.kind {
        QubitInitKind::Single => {}
        QubitInitKind::Tuple(inits) => {
            inits.iter().for_each(|i| vis.visit_qubit_init(i));
        }
        QubitInitKind::Array(len) => {
            vis.visit_expr(len);
        }
    }
}
