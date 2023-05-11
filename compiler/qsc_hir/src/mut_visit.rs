// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::hir::{
    Block, CallableBody, CallableDecl, Expr, ExprKind, FunctorExpr, FunctorExprKind, Ident, Item,
    ItemKind, Package, Pat, PatKind, QubitInit, QubitInitKind, SpecBody, SpecDecl, Stmt, StmtKind,
    TyDef, TyDefKind,
};
use qsc_data_structures::span::Span;

pub trait MutVisitor: Sized {
    fn visit_package(&mut self, package: &mut Package) {
        walk_package(self, package);
    }

    fn visit_item(&mut self, item: &mut Item) {
        walk_item(self, item);
    }

    fn visit_ty_def(&mut self, def: &mut TyDef) {
        walk_ty_def(self, def);
    }

    fn visit_callable_decl(&mut self, decl: &mut CallableDecl) {
        walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &mut SpecDecl) {
        walk_spec_decl(self, decl);
    }

    fn visit_functor_expr(&mut self, expr: &mut FunctorExpr) {
        walk_functor_expr(self, expr);
    }

    fn visit_block(&mut self, block: &mut Block) {
        walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &mut Pat) {
        walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &mut QubitInit) {
        walk_qubit_init(self, init);
    }

    fn visit_ident(&mut self, ident: &mut Ident) {
        walk_ident(self, ident);
    }

    fn visit_span(&mut self, _: &mut Span) {}
}

pub fn walk_package(vis: &mut impl MutVisitor, package: &mut Package) {
    package.items.values_mut().for_each(|i| vis.visit_item(i));
    package.entry.iter_mut().for_each(|e| vis.visit_expr(e));
}

pub fn walk_item(vis: &mut impl MutVisitor, item: &mut Item) {
    vis.visit_span(&mut item.span);

    match &mut item.kind {
        ItemKind::Callable(decl) => vis.visit_callable_decl(decl),
        ItemKind::Namespace(name, _) => vis.visit_ident(name),
        ItemKind::Ty(ident, def) => {
            vis.visit_ident(ident);
            vis.visit_ty_def(def);
        }
    }
}

pub fn walk_ty_def(vis: &mut impl MutVisitor, def: &mut TyDef) {
    vis.visit_span(&mut def.span);

    match &mut def.kind {
        TyDefKind::Field(name, _) => name.iter_mut().for_each(|n| vis.visit_ident(n)),
        TyDefKind::Tuple(defs) => defs.iter_mut().for_each(|d| vis.visit_ty_def(d)),
    }
}

pub fn walk_callable_decl(vis: &mut impl MutVisitor, decl: &mut CallableDecl) {
    vis.visit_span(&mut decl.span);
    vis.visit_ident(&mut decl.name);
    decl.ty_params.iter_mut().for_each(|p| vis.visit_ident(p));
    vis.visit_pat(&mut decl.input);
    decl.functors
        .iter_mut()
        .for_each(|f| vis.visit_functor_expr(f));

    match &mut decl.body {
        CallableBody::Block(block) => vis.visit_block(block),
        CallableBody::Specs(specs) => specs.iter_mut().for_each(|s| vis.visit_spec_decl(s)),
    }
}

pub fn walk_spec_decl(vis: &mut impl MutVisitor, decl: &mut SpecDecl) {
    vis.visit_span(&mut decl.span);

    match &mut decl.body {
        SpecBody::Gen(_) => {}
        SpecBody::Impl(pat, block) => {
            vis.visit_pat(pat);
            vis.visit_block(block);
        }
    }
}

pub fn walk_functor_expr(vis: &mut impl MutVisitor, expr: &mut FunctorExpr) {
    vis.visit_span(&mut expr.span);

    match &mut expr.kind {
        FunctorExprKind::BinOp(_, lhs, rhs) => {
            vis.visit_functor_expr(lhs);
            vis.visit_functor_expr(rhs);
        }
        FunctorExprKind::Lit(_) => {}
    }
}

pub fn walk_block(vis: &mut impl MutVisitor, block: &mut Block) {
    vis.visit_span(&mut block.span);
    block.stmts.iter_mut().for_each(|s| vis.visit_stmt(s));
}

pub fn walk_stmt(vis: &mut impl MutVisitor, stmt: &mut Stmt) {
    vis.visit_span(&mut stmt.span);

    match &mut stmt.kind {
        StmtKind::Item(_) => {}
        StmtKind::Expr(expr) | StmtKind::Semi(expr) => vis.visit_expr(expr),
        StmtKind::Local(_, pat, value) => {
            vis.visit_pat(pat);
            vis.visit_expr(value);
        }
        StmtKind::Qubit(_, pat, init, block) => {
            vis.visit_pat(pat);
            vis.visit_qubit_init(init);
            block.iter_mut().for_each(|b| vis.visit_block(b));
        }
    }
}

pub fn walk_expr(vis: &mut impl MutVisitor, expr: &mut Expr) {
    vis.visit_span(&mut expr.span);

    match &mut expr.kind {
        ExprKind::Array(exprs) => exprs.iter_mut().for_each(|e| vis.visit_expr(e)),
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
        ExprKind::Field(record, _) => vis.visit_expr(record),
        ExprKind::For(pat, iter, block) => {
            vis.visit_pat(pat);
            vis.visit_expr(iter);
            vis.visit_block(block);
        }
        ExprKind::If(cond, body, otherwise) => {
            vis.visit_expr(cond);
            vis.visit_block(body);
            otherwise.iter_mut().for_each(|e| vis.visit_expr(e));
        }
        ExprKind::Index(array, index) => {
            vis.visit_expr(array);
            vis.visit_expr(index);
        }
        ExprKind::Lambda(_, pat, expr) => {
            vis.visit_pat(pat);
            vis.visit_expr(expr);
        }
        ExprKind::Return(expr) | ExprKind::UnOp(_, expr) => {
            vis.visit_expr(expr);
        }
        ExprKind::Range(start, step, end) => {
            start.iter_mut().for_each(|s| vis.visit_expr(s));
            step.iter_mut().for_each(|s| vis.visit_expr(s));
            end.iter_mut().for_each(|e| vis.visit_expr(e));
        }
        ExprKind::Repeat(body, until, fixup) => {
            vis.visit_block(body);
            vis.visit_expr(until);
            fixup.iter_mut().for_each(|f| vis.visit_block(f));
        }
        ExprKind::TernOp(_, e1, e2, e3) => {
            vis.visit_expr(e1);
            vis.visit_expr(e2);
            vis.visit_expr(e3);
        }
        ExprKind::Tuple(exprs) => exprs.iter_mut().for_each(|e| vis.visit_expr(e)),
        ExprKind::While(cond, block) => {
            vis.visit_expr(cond);
            vis.visit_block(block);
        }
        ExprKind::Err | ExprKind::Hole | ExprKind::Lit(_) | ExprKind::Var(_) => {}
    }
}

pub fn walk_pat(vis: &mut impl MutVisitor, pat: &mut Pat) {
    vis.visit_span(&mut pat.span);

    match &mut pat.kind {
        PatKind::Bind(name) => vis.visit_ident(name),
        PatKind::Discard | PatKind::Elided => {}
        PatKind::Tuple(pats) => pats.iter_mut().for_each(|p| vis.visit_pat(p)),
    }
}

pub fn walk_qubit_init(vis: &mut impl MutVisitor, init: &mut QubitInit) {
    vis.visit_span(&mut init.span);

    match &mut init.kind {
        QubitInitKind::Array(len) => vis.visit_expr(len),
        QubitInitKind::Single => {}
        QubitInitKind::Tuple(inits) => inits.iter_mut().for_each(|i| vis.visit_qubit_init(i)),
    }
}

pub fn walk_ident(vis: &mut impl MutVisitor, ident: &mut Ident) {
    vis.visit_span(&mut ident.span);
}
