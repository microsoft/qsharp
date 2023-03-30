// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::ast::{
    Attr, BinOp, Block, CallableBody, CallableDecl, CallableKind, Expr, ExprKind, FunctorExpr,
    FunctorExprKind, Ident, Item, ItemKind, Lit, Namespace, Package, Pat, PatKind, Path, QubitInit,
    QubitInitKind, SpecBody, SpecDecl, Stmt, StmtKind, TernOp, Ty, TyDef, TyDefKind, TyKind, UnOp,
};

pub trait Visitor<'a>: Sized {
    fn visit_package(&mut self, package: &'a Package) {
        walk_package(self, package);
    }

    fn visit_namespace(&mut self, namespace: &'a Namespace) {
        walk_namespace(self, namespace);
    }

    fn visit_item(&mut self, item: &'a Item) {
        walk_item(self, item);
    }

    fn visit_attr(&mut self, attr: &'a Attr) {
        walk_attr(self, attr);
    }

    fn visit_ty_def(&mut self, def: &'a TyDef) {
        walk_ty_def(self, def);
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
        walk_spec_decl(self, decl);
    }

    fn visit_functor_expr(&mut self, expr: &'a FunctorExpr) {
        walk_functor_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &'a Ty) {
        walk_ty(self, ty);
    }

    fn visit_block(&mut self, block: &'a Block) {
        walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &'a Pat) {
        walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &'a QubitInit) {
        walk_qubit_init(self, init);
    }

    fn visit_path(&mut self, path: &'a Path) {
        walk_path(self, path);
    }

    fn visit_ident(&mut self, _: &'a Ident) {}

    fn visit_array(&mut self, exprs: &'a Vec<Expr>) {
        walk_array(self, exprs);
    }

    fn visit_array_repeat(&mut self, item: &'a Expr, size: &'a Expr) {
        walk_array_repeat(self, item, size);
    }

    fn visit_assign(&mut self, lhs: &'a Expr, rhs: &'a Expr) {
        walk_assign(self, lhs, rhs);
    }

    fn visit_assign_op(&mut self, op: &'a BinOp, lhs: &'a Expr, rhs: &'a Expr) {
        walk_assign_op(self, op, lhs, rhs);
    }

    fn visit_assign_update(&mut self, record: &'a Expr, index: &'a Expr, value: &'a Expr) {
        walk_assign_update(self, record, index, value);
    }

    fn visit_bin_op(&mut self, op: &'a BinOp, lhs: &'a Expr, rhs: &'a Expr) {
        walk_bin_op(self, op, lhs, rhs);
    }

    fn visit_call(&mut self, callee: &'a Expr, arg: &'a Expr) {
        walk_call(self, callee, arg);
    }

    fn visit_conjugate(&mut self, within: &'a Block, apply: &'a Block) {
        walk_conjugate(self, within, apply);
    }

    fn visit_err(&mut self) {
        walk_err(self);
    }

    fn visit_fail(&mut self, msg: &'a Expr) {
        walk_fail(self, msg);
    }

    fn visit_field(&mut self, record: &'a Expr, name: &'a Ident) {
        walk_field(self, record, name);
    }

    fn visit_for(&mut self, pat: &'a Pat, iter: &'a Expr, block: &'a Block) {
        walk_for(self, pat, iter, block);
    }

    fn visit_hole(&mut self) {
        walk_hole(self);
    }

    fn visit_if(&mut self, cond: &'a Expr, body: &'a Block, otherwise: &'a Option<Box<Expr>>) {
        walk_if(self, cond, body, otherwise);
    }

    fn visit_index(&mut self, array: &'a Expr, index: &'a Expr) {
        walk_index(self, array, index);
    }

    fn visit_lambda(&mut self, kind: &'a CallableKind, pat: &'a Pat, expr: &'a Expr) {
        walk_lambda(self, kind, pat, expr);
    }

    fn visit_lit(&mut self, lit: &'a Lit) {
        walk_lit(self, lit);
    }

    fn visit_paren(&mut self, expr: &'a Expr) {
        walk_paren(self, expr);
    }

    fn visit_range(
        &mut self,
        start: &'a Option<Box<Expr>>,
        step: &'a Option<Box<Expr>>,
        end: &'a Option<Box<Expr>>,
    ) {
        walk_range(self, start, step, end);
    }

    fn visit_repeat(&mut self, body: &'a Block, until: &'a Expr, fixup: &'a Option<Block>) {
        walk_repeat(self, body, until, fixup);
    }

    fn visit_return(&mut self, expr: &'a Expr) {
        walk_return(self, expr);
    }

    fn visit_tern_op(&mut self, op: &'a TernOp, e1: &'a Expr, e2: &'a Expr, e3: &'a Expr) {
        walk_tern_op(self, op, e1, e2, e3);
    }

    fn visit_tuple(&mut self, exprs: &'a Vec<Expr>) {
        walk_tuple(self, exprs);
    }

    fn visit_un_op(&mut self, op: &'a UnOp, expr: &'a Expr) {
        walk_un_op(self, op, expr);
    }

    fn visis_while(&mut self, cond: &'a Expr, block: &'a Block) {
        walk_while(self, cond, block);
    }
}

pub fn walk_package<'a>(vis: &mut impl Visitor<'a>, package: &'a Package) {
    package
        .namespaces
        .iter()
        .for_each(|n| vis.visit_namespace(n));
    package.entry.iter().for_each(|e| vis.visit_expr(e));
}

pub fn walk_namespace<'a>(vis: &mut impl Visitor<'a>, namespace: &'a Namespace) {
    vis.visit_ident(&namespace.name);
    namespace.items.iter().for_each(|i| vis.visit_item(i));
}

pub fn walk_item<'a>(vis: &mut impl Visitor<'a>, item: &'a Item) {
    item.meta.attrs.iter().for_each(|a| vis.visit_attr(a));
    match &item.kind {
        ItemKind::Err => {}
        ItemKind::Callable(decl) => vis.visit_callable_decl(decl),
        ItemKind::Open(ns, alias) => {
            vis.visit_ident(ns);
            alias.iter().for_each(|a| vis.visit_ident(a));
        }
        ItemKind::Ty(ident, def) => {
            vis.visit_ident(ident);
            vis.visit_ty_def(def);
        }
    }
}

pub fn walk_attr<'a>(vis: &mut impl Visitor<'a>, attr: &'a Attr) {
    vis.visit_path(&attr.name);
    vis.visit_expr(&attr.arg);
}

pub fn walk_ty_def<'a>(vis: &mut impl Visitor<'a>, def: &'a TyDef) {
    match &def.kind {
        TyDefKind::Field(name, ty) => {
            name.iter().for_each(|n| vis.visit_ident(n));
            vis.visit_ty(ty);
        }
        TyDefKind::Paren(def) => vis.visit_ty_def(def),
        TyDefKind::Tuple(defs) => defs.iter().for_each(|d| vis.visit_ty_def(d)),
    }
}

pub fn walk_callable_decl<'a>(vis: &mut impl Visitor<'a>, decl: &'a CallableDecl) {
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

pub fn walk_spec_decl<'a>(vis: &mut impl Visitor<'a>, decl: &'a SpecDecl) {
    match &decl.body {
        SpecBody::Gen(_) => {}
        SpecBody::Impl(pat, block) => {
            vis.visit_pat(pat);
            vis.visit_block(block);
        }
    }
}

pub fn walk_functor_expr<'a>(vis: &mut impl Visitor<'a>, expr: &'a FunctorExpr) {
    match &expr.kind {
        FunctorExprKind::BinOp(_, lhs, rhs) => {
            vis.visit_functor_expr(lhs);
            vis.visit_functor_expr(rhs);
        }
        FunctorExprKind::Lit(_) => {}
        FunctorExprKind::Paren(expr) => vis.visit_functor_expr(expr),
    }
}

pub fn walk_ty<'a>(vis: &mut impl Visitor<'a>, ty: &'a Ty) {
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

pub fn walk_block<'a>(vis: &mut impl Visitor<'a>, block: &'a Block) {
    block.stmts.iter().for_each(|s| vis.visit_stmt(s));
}

pub fn walk_stmt<'a>(vis: &mut impl Visitor<'a>, stmt: &'a Stmt) {
    match &stmt.kind {
        StmtKind::Empty => {}
        StmtKind::Expr(expr) | StmtKind::Semi(expr) => vis.visit_expr(expr),
        StmtKind::Local(_, pat, value) => {
            vis.visit_pat(pat);
            vis.visit_expr(value);
        }
        StmtKind::Qubit(_, pat, init, block) => {
            vis.visit_pat(pat);
            vis.visit_qubit_init(init);
            block.iter().for_each(|b| vis.visit_block(b));
        }
    }
}

pub fn walk_expr<'a>(vis: &mut impl Visitor<'a>, expr: &'a Expr) {
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
        ExprKind::Range(start, step, end) => {
            start.iter().for_each(|s| vis.visit_expr(s));
            step.iter().for_each(|s| vis.visit_expr(s));
            end.iter().for_each(|e| vis.visit_expr(e));
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
        ExprKind::Err | ExprKind::Hole | ExprKind::Lit(_) => {}
    }
}

pub fn walk_pat<'a>(vis: &mut impl Visitor<'a>, pat: &'a Pat) {
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

pub fn walk_qubit_init<'a>(vis: &mut impl Visitor<'a>, init: &'a QubitInit) {
    match &init.kind {
        QubitInitKind::Array(len) => vis.visit_expr(len),
        QubitInitKind::Paren(init) => vis.visit_qubit_init(init),
        QubitInitKind::Single => {}
        QubitInitKind::Tuple(inits) => inits.iter().for_each(|i| vis.visit_qubit_init(i)),
    }
}

pub fn walk_path<'a>(vis: &mut impl Visitor<'a>, path: &'a Path) {
    path.namespace.iter().for_each(|n| vis.visit_ident(n));
    vis.visit_ident(&path.name);
}

fn walk_array<'a>(vis: &mut impl Visitor<'a>, exprs: &'a Vec<Expr>) {
    exprs.iter().for_each(|e| vis.visit_expr(e));
}

fn walk_array_repeat<'a>(vis: &mut impl Visitor<'a>, item: &'a Expr, size: &'a Expr) {
    vis.visit_expr(item);
    vis.visit_expr(size);
}

fn walk_assign<'a>(vis: &mut impl Visitor<'a>, lhs: &'a Expr, rhs: &'a Expr) {
    vis.visit_expr(lhs);
    vis.visit_expr(rhs);
}

fn walk_assign_op<'a>(vis: &mut impl Visitor<'a>, op: &'a BinOp, lhs: &'a Expr, rhs: &'a Expr) {
    vis.visit_expr(lhs);
    vis.visit_expr(rhs);
}

fn walk_assign_update<'a>(
    vis: &mut impl Visitor<'a>,
    record: &'a Expr,
    index: &'a Expr,
    value: &'a Expr,
) {
    vis.visit_expr(record);
    vis.visit_expr(index);
    vis.visit_expr(value);
}

fn walk_bin_op<'a>(vis: &mut impl Visitor<'a>, op: &'a BinOp, lhs: &'a Expr, rhs: &'a Expr) {
    vis.visit_expr(lhs);
    vis.visit_expr(rhs);
}

fn walk_call<'a>(vis: &mut impl Visitor<'a>, callee: &'a Expr, arg: &'a Expr) {
    vis.visit_expr(callee);
    vis.visit_expr(arg);
}

fn walk_conjugate<'a>(vis: &mut impl Visitor<'a>, within: &'a Block, apply: &'a Block) {
    vis.visit_block(within);
    vis.visit_block(apply);
}

fn walk_err<'a>(vis: &mut impl Visitor<'a>) {}

fn walk_fail<'a>(vis: &mut impl Visitor<'a>, msg: &'a Expr) {
    vis.visit_expr(msg);
}

fn walk_field<'a>(vis: &mut impl Visitor<'a>, record: &'a Expr, name: &'a Ident) {
    vis.visit_expr(record);
    vis.visit_ident(name);
}

fn walk_for<'a>(vis: &mut impl Visitor<'a>, pat: &'a Pat, iter: &'a Expr, block: &'a Block) {
    vis.visit_pat(pat);
    vis.visit_expr(iter);
    vis.visit_block(block);
}

fn walk_hole<'a>(vis: &mut impl Visitor<'a>) {}

fn walk_if<'a>(
    vis: &mut impl Visitor<'a>,
    cond: &'a Expr,
    body: &'a Block,
    otherwise: &'a Option<Box<Expr>>,
) {
    vis.visit_expr(cond);
    vis.visit_block(body);
    if let Some(e) = otherwise {
        vis.visit_expr(e);
    }
}

fn walk_index<'a>(vis: &mut impl Visitor<'a>, array: &'a Expr, index: &'a Expr) {
    vis.visit_expr(array);
    vis.visit_expr(index);
}

fn walk_lambda<'a>(
    vis: &mut impl Visitor<'a>,
    kind: &'a CallableKind,
    pat: &'a Pat,
    expr: &'a Expr,
) {
    vis.visit_pat(pat);
    vis.visit_expr(expr);
}

fn walk_lit<'a>(vis: &mut impl Visitor<'a>, lit: &'a Lit) {}

fn walk_paren<'a>(vis: &mut impl Visitor<'a>, expr: &'a Expr) {
    vis.visit_expr(expr);
}

fn walk_range<'a>(
    vis: &mut impl Visitor<'a>,
    start: &'a Option<Box<Expr>>,
    step: &'a Option<Box<Expr>>,
    end: &'a Option<Box<Expr>>,
) {
    if let Some(s) = start {
        vis.visit_expr(s);
    }

    if let Some(s) = step {
        vis.visit_expr(s);
    }

    if let Some(e) = end {
        vis.visit_expr(e);
    }
}

fn walk_repeat<'a>(
    vis: &mut impl Visitor<'a>,
    body: &'a Block,
    until: &'a Expr,
    fixup: &'a Option<Block>,
) {
    vis.visit_block(body);
    vis.visit_expr(until);
    if let Some(f) = fixup {
        vis.visit_block(f);
    }
}

fn walk_return<'a>(vis: &mut impl Visitor<'a>, expr: &'a Expr) {
    vis.visit_expr(expr);
}

fn walk_tern_op<'a>(
    vis: &mut impl Visitor<'a>,
    op: &'a TernOp,
    e1: &'a Expr,
    e2: &'a Expr,
    e3: &'a Expr,
) {
    vis.visit_expr(e1);
    vis.visit_expr(e2);
    vis.visit_expr(e3);
}

fn walk_tuple<'a>(vis: &mut impl Visitor<'a>, exprs: &'a Vec<Expr>) {
    exprs.iter().for_each(|e| vis.visit_expr(e));
}

fn walk_un_op<'a>(vis: &mut impl Visitor<'a>, op: &'a UnOp, expr: &'a Expr) {
    vis.visit_expr(expr);
}

fn walk_while<'a>(vis: &mut impl Visitor<'a>, cond: &'a Expr, block: &'a Block) {
    vis.visit_expr(cond);
    vis.visit_block(block);
}
