// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::span::Span;
use qsc_hir::{
    assigner::Assigner,
    global::Table,
    hir::{
        Expr, ExprKind, Field, Ident, Mutability, NodeId, Pat, PatKind, PrimField, Res, Stmt,
        StmtKind,
    },
    ty::{GenericArg, Prim, Ty},
};
use std::rc::Rc;

pub(crate) fn generated_name(name: &str) -> Rc<str> {
    Rc::from(format!("@{name}"))
}

#[derive(Debug, Clone)]
pub(crate) struct IdentTemplate {
    pub id: NodeId,
    pub span: Span,
    pub name: Rc<str>,
    pub ty: Ty,
}

impl IdentTemplate {
    pub fn gen_local_ref(&self, assigner: &mut Assigner) -> Expr {
        Expr {
            id: assigner.next_node(),
            span: self.span,
            ty: self.ty.clone(),
            kind: ExprKind::Var(Res::Local(self.id), Vec::new()),
        }
    }

    fn gen_pat(&self, assigner: &mut Assigner) -> Pat {
        Pat {
            id: assigner.next_node(),
            span: self.span,
            ty: self.ty.clone(),
            kind: PatKind::Bind(Ident {
                id: self.id,
                span: self.span,
                name: self.name.clone(),
            }),
        }
    }

    pub fn gen_field_access(&self, field: PrimField, assigner: &mut Assigner) -> Expr {
        Expr {
            id: assigner.next_node(),
            span: self.span,
            ty: Ty::Prim(Prim::Int),
            kind: ExprKind::Field(Box::new(self.gen_local_ref(assigner)), Field::Prim(field)),
        }
    }

    pub fn gen_id_init(&self, mutability: Mutability, expr: Expr, assigner: &mut Assigner) -> Stmt {
        Stmt {
            id: assigner.next_node(),
            span: self.span,
            kind: StmtKind::Local(mutability, self.gen_pat(assigner), expr),
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
