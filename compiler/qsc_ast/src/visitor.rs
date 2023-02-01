// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    Attribute, Block, CallBody, CallHeader, DeclInfo, Expr, ExprKind, FunctorExpr, Ident, Item,
    Namespace, Pat, Path, Project, QubitInit, SpecBody, SpecDecl, Stage, Ty, TypeDef,
};

pub trait Visitor<S: Stage>: Sized {
    fn visit_project(&mut self, project: &Project<S>) {
        walk_project(self, project);
    }

    fn visit_namespace(&mut self, namespace: &Namespace<S>) {
        walk_namespace(self, namespace);
    }

    fn visit_item(&mut self, item: &Item<S>) {
        walk_item(self, item);
    }

    fn visit_decl_info(&mut self, info: &DeclInfo<S>) {
        walk_decl_info(self, info);
    }

    fn visit_attribute(&mut self, attr: &Attribute<S>) {
        walk_attribute(self, attr);
    }

    fn visit_type_def(&mut self, def: &TypeDef<S>) {
        walk_type_def(self, def);
    }

    fn visit_call_header(&mut self, header: &CallHeader<S>) {
        walk_call_header(self, header);
    }

    fn visit_call_body(&mut self, body: &CallBody<S>) {
        walk_call_body(self, body);
    }

    fn visit_spec_decl(&mut self, decl: &SpecDecl<S>) {
        walk_spec_decl(self, decl);
    }

    fn visit_spec_body(&mut self, body: &SpecBody<S>) {
        walk_spec_body(self, body);
    }

    fn visit_functor_expr(&mut self, expr: &FunctorExpr<S>) {
        walk_functor_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &Ty<S>) {
        walk_ty(self, ty);
    }

    fn visit_expr(&mut self, expr: &Expr<S>) {
        walk_expr(self, expr);
    }

    fn visit_block(&mut self, block: &Block<S>) {
        walk_block(self, block);
    }

    fn visit_ident(&mut self, _: &Ident<S>) {}

    fn visit_path(&mut self, _: &Path<S>) {}

    fn visit_pat(&mut self, pat: &Pat<S>) {
        walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &QubitInit<S>) {
        walk_qubit_init(self, init);
    }
}

pub fn walk_project<S: Stage>(visitor: &mut impl Visitor<S>, project: &Project<S>) {
    project
        .namespaces
        .iter()
        .for_each(|n| visitor.visit_namespace(n));
}

pub fn walk_namespace<S: Stage>(visitor: &mut impl Visitor<S>, namespace: &Namespace<S>) {
    visitor.visit_path(&namespace.name);
    namespace.items.iter().for_each(|i| visitor.visit_item(i));
}

pub fn walk_item<S: Stage>(visitor: &mut impl Visitor<S>, item: &Item<S>) {
    match item {
        Item::Open(_, path, ident) => {
            visitor.visit_path(path);
            visitor.visit_ident(ident);
        }
        Item::Type(_, info, ident, def) => {
            visitor.visit_decl_info(info);
            visitor.visit_ident(ident);
            visitor.visit_type_def(def);
        }
        Item::Callable(_, info, header, body) => {
            visitor.visit_decl_info(info);
            visitor.visit_call_header(header);
            visitor.visit_call_body(body);
        }
        Item::X(_) => {}
    }
}

pub fn walk_decl_info<S: Stage>(visitor: &mut impl Visitor<S>, info: &DeclInfo<S>) {
    info.attributes
        .iter()
        .for_each(|a| visitor.visit_attribute(a));
}

pub fn walk_attribute<S: Stage>(visitor: &mut impl Visitor<S>, attr: &Attribute<S>) {
    visitor.visit_path(&attr.name);
    visitor.visit_expr(&attr.arg);
}

pub fn walk_type_def<S: Stage>(visitor: &mut impl Visitor<S>, def: &TypeDef<S>) {
    match def {
        TypeDef::Field(_, name, ty) => {
            name.iter().for_each(|n| visitor.visit_ident(n));
            visitor.visit_ty(ty);
        }
        TypeDef::Tuple(_, defs) => {
            defs.iter().for_each(|d| visitor.visit_type_def(d));
        }
        TypeDef::X(_) => {}
    }
}

pub fn walk_call_header<S: Stage>(visitor: &mut impl Visitor<S>, header: &CallHeader<S>) {
    visitor.visit_ident(&header.name);
    header.ty_params.iter().for_each(|i| visitor.visit_ident(i));
    visitor.visit_pat(&header.input);
    visitor.visit_ty(&header.output);
    visitor.visit_functor_expr(&header.functors);
}

pub fn walk_call_body<S: Stage>(visitor: &mut impl Visitor<S>, body: &CallBody<S>) {
    match body {
        CallBody::Single(_, body) => visitor.visit_spec_body(body),
        CallBody::Full(_, decls) => decls.iter().for_each(|d| visitor.visit_spec_decl(d)),
        CallBody::X(_) => {}
    }
}

pub fn walk_spec_decl<S: Stage>(visitor: &mut impl Visitor<S>, decl: &SpecDecl<S>) {
    visitor.visit_spec_body(&decl.body);
}

pub fn walk_spec_body<S: Stage>(visitor: &mut impl Visitor<S>, body: &SpecBody<S>) {
    match body {
        SpecBody::Impl(_, pat, block) => {
            visitor.visit_pat(pat);
            visitor.visit_block(block);
        }
        SpecBody::Gen(_, _) | SpecBody::X(_) => {}
    }
}

pub fn walk_functor_expr<S: Stage>(visitor: &mut impl Visitor<S>, expr: &FunctorExpr<S>) {
    match expr {
        FunctorExpr::BinOp(_, _, lhs, rhs) => {
            visitor.visit_functor_expr(lhs);
            visitor.visit_functor_expr(rhs);
        }
        FunctorExpr::Lit(_, _) | FunctorExpr::Null(_) | FunctorExpr::X(_) => {}
    }
}

pub fn walk_ty<S: Stage>(visitor: &mut impl Visitor<S>, ty: &Ty<S>) {
    match ty {
        Ty::App(_, ty, tys) => {
            visitor.visit_ty(ty);
            tys.iter().for_each(|t| visitor.visit_ty(t));
        }
        Ty::Arrow(_, _, lhs, rhs, functors) => {
            visitor.visit_ty(lhs);
            visitor.visit_ty(rhs);
            visitor.visit_functor_expr(functors);
        }
        Ty::Path(_, path) => visitor.visit_path(path),
        Ty::Tuple(_, tys) => {
            tys.iter().for_each(|t| visitor.visit_ty(t));
        }
        Ty::Hole(_) | Ty::Prim(_, _) | Ty::Var(_, _) | Ty::X(_) => {}
    }
}

pub fn walk_expr<S: Stage>(visitor: &mut impl Visitor<S>, expr: &Expr<S>) {
    match &expr.kind {
        ExprKind::Array(_, exprs) => {
            exprs.iter().for_each(|e| visitor.visit_expr(e));
        }
        ExprKind::ArrayRepeat(_, item, size) => {
            visitor.visit_expr(item);
            visitor.visit_expr(size);
        }
        ExprKind::Assign(_, lhs, rhs)
        | ExprKind::AssignOp(_, _, lhs, rhs)
        | ExprKind::BinOp(_, _, lhs, rhs) => {
            visitor.visit_expr(lhs);
            visitor.visit_expr(rhs);
        }
        ExprKind::AssignUpdate(_, record, index, value) => {
            visitor.visit_expr(record);
            visitor.visit_expr(index);
            visitor.visit_expr(value);
        }
        ExprKind::Block(_, block) => visitor.visit_block(block),
        ExprKind::Call(_, callee, arg) => {
            visitor.visit_expr(callee);
            visitor.visit_expr(arg);
        }
        ExprKind::Conjugate(_, within, apply) => {
            visitor.visit_block(within);
            visitor.visit_block(apply);
        }
        ExprKind::Fail(_, msg) => {
            visitor.visit_expr(msg);
        }
        ExprKind::Field(_, record, name) => {
            visitor.visit_expr(record);
            visitor.visit_ident(name);
        }
        ExprKind::For(_, pat, iter, block) => {
            visitor.visit_pat(pat);
            visitor.visit_expr(iter);
            visitor.visit_block(block);
        }
        ExprKind::If(_, branches, default) => {
            for (cond, block) in branches {
                visitor.visit_expr(cond);
                visitor.visit_block(block);
            }
            default.iter().for_each(|d| visitor.visit_block(d));
        }
        ExprKind::Index(_, array, index) => {
            visitor.visit_expr(array);
            visitor.visit_expr(index);
        }
        ExprKind::Interp(_, _, exprs) => {
            exprs.iter().for_each(|e| visitor.visit_expr(e));
        }
        ExprKind::Lambda(_, _, pat, expr) => {
            visitor.visit_pat(pat);
            visitor.visit_expr(expr);
        }
        ExprKind::Let(_, pat, value) => {
            visitor.visit_pat(pat);
            visitor.visit_expr(value);
        }
        ExprKind::Path(_, path) => visitor.visit_path(path),
        ExprKind::Qubit(_, _, pat, init, block) => {
            visitor.visit_pat(pat);
            visitor.visit_qubit_init(init);
            block.iter().for_each(|b| visitor.visit_block(b));
        }
        ExprKind::Range(_, start, step, end) => {
            visitor.visit_expr(start);
            visitor.visit_expr(step);
            visitor.visit_expr(end);
        }
        ExprKind::Repeat(_, body, until, fixup) => {
            visitor.visit_block(body);
            visitor.visit_expr(until);
            fixup.iter().for_each(|f| visitor.visit_block(f));
        }
        ExprKind::Return(_, expr) | ExprKind::UnOp(_, _, expr) => {
            visitor.visit_expr(expr);
        }
        ExprKind::TernOp(_, _, e1, e2, e3) => {
            visitor.visit_expr(e1);
            visitor.visit_expr(e2);
            visitor.visit_expr(e3);
        }
        ExprKind::Tuple(_, exprs) => {
            exprs.iter().for_each(|e| visitor.visit_expr(e));
        }
        ExprKind::While(_, cond, block) => {
            visitor.visit_expr(cond);
            visitor.visit_block(block);
        }
        ExprKind::Hole(_) | ExprKind::Lit(_, _) | ExprKind::X(_) => {}
    }
}

pub fn walk_block<S: Stage>(visitor: &mut impl Visitor<S>, block: &Block<S>) {
    block.exprs.iter().for_each(|e| visitor.visit_expr(e));
}

pub fn walk_pat<S: Stage>(visitor: &mut impl Visitor<S>, pat: &Pat<S>) {
    match pat {
        Pat::Bind(_, _, name, ty) => {
            visitor.visit_ident(name);
            visitor.visit_ty(ty);
        }
        Pat::Discard(_, ty) => {
            visitor.visit_ty(ty);
        }
        Pat::Tuple(_, pats) => {
            pats.iter().for_each(|p| visitor.visit_pat(p));
        }
        Pat::Omit(_) | Pat::X(_) => {}
    }
}

pub fn walk_qubit_init<S: Stage>(visitor: &mut impl Visitor<S>, init: &QubitInit<S>) {
    match init {
        QubitInit::Tuple(_, inits) => {
            inits.iter().for_each(|i| visitor.visit_qubit_init(i));
        }
        QubitInit::Array(_, len) => {
            visitor.visit_expr(len);
        }
        QubitInit::Single(_) | QubitInit::X(_) => {}
    }
}
