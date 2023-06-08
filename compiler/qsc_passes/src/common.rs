// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::span::Span;
use qsc_hir::{
    global::Table,
    hir::{
        Expr, ExprKind, Field, GenericArg, Ident, Mutability, NodeId, Pat, PatKind, PrimField,
        PrimTy, Res, Stmt, StmtKind, Ty,
    },
};
use std::rc::Rc;

pub(crate) struct IdentTemplate {
    pub id: NodeId,
    pub span: Span,
    pub name: Rc<str>,
    pub ty: Ty,
}

impl IdentTemplate {
    pub fn gen_local_ref(&self) -> Expr {
        Expr {
            id: NodeId::default(),
            span: self.span,
            ty: self.ty.clone(),
            kind: ExprKind::Var(Res::Local(self.id), Vec::new()),
        }
    }

    fn gen_pat(&self) -> Pat {
        Pat {
            id: NodeId::default(),
            span: self.span,
            ty: self.ty.clone(),
            kind: PatKind::Bind(Ident {
                id: self.id,
                span: self.span,
                name: self.name.clone(),
            }),
        }
    }

    pub fn gen_field_access(&self, field: PrimField) -> Expr {
        Expr {
            id: NodeId::default(),
            span: self.span,
            ty: Ty::Prim(PrimTy::Int),
            kind: ExprKind::Field(Box::new(self.gen_local_ref()), Field::Prim(field)),
        }
    }

    pub fn gen_id_init(&self, mutability: Mutability, expr: Expr) -> Stmt {
        Stmt {
            id: NodeId::default(),
            span: self.span,
            kind: StmtKind::Local(mutability, self.gen_pat(), expr),
        }
    }
}

pub(crate) fn create_gen_core_ref(
    core: &Table,
    namespace: &str,
    name: &str,
    generics: Vec<GenericArg>,
    span: Span,
) -> Expr {
    let term = core
        .resolve_term(namespace, name)
        .expect("term should resolve");

    let ty = term
        .scheme
        .instantiate(&generics)
        .expect("generic arguments should match type scheme");

    Expr {
        id: NodeId::default(),
        span,
        ty: Ty::Arrow(Box::new(ty)),
        kind: ExprKind::Var(Res::Item(term.id), generics),
    }
}
