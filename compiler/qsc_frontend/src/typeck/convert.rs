// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::resolve::{self, Res};
use qsc_ast::ast::{
    self, CallableBody, CallableDecl, CallableKind, FunctorExpr, FunctorExprKind, NodeId, Pat,
    PatKind, SetOp, Spec, TyDef, TyDefKind, TyKind,
};
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::hir::{self, FieldPath, Functor, ItemId, Ty, UdtField};
use std::{collections::HashSet, rc::Rc};

pub(crate) struct MissingTyError(pub(super) Span);

pub(crate) fn ty_from_ast(
    names: &IndexMap<NodeId, Res>,
    ty: &ast::Ty,
) -> (Ty, Vec<MissingTyError>) {
    match &ty.kind {
        TyKind::Array(item) => {
            let (item, errors) = ty_from_ast(names, item);
            (Ty::Array(Box::new(item)), errors)
        }
        TyKind::Arrow(kind, input, output, functors) => {
            let (input, mut errors) = ty_from_ast(names, input);
            let (output, output_errors) = ty_from_ast(names, output);
            errors.extend(output_errors);
            let functors = functors.as_ref().map_or(HashSet::new(), eval_functor_expr);
            let ty = Ty::Arrow(
                callable_kind_from_ast(*kind),
                Box::new(input),
                Box::new(output),
                functors,
            );
            (ty, errors)
        }
        TyKind::Hole => (Ty::Err, vec![MissingTyError(ty.span)]),
        TyKind::Paren(inner) => ty_from_ast(names, inner),
        TyKind::Path(path) => {
            let ty = match names.get(path.id) {
                Some(&resolve::Res::Item(item)) => Ty::Udt(hir::Res::Item(item)),
                Some(&resolve::Res::PrimTy(prim)) => Ty::Prim(prim),
                Some(resolve::Res::UnitTy) => Ty::Tuple(Vec::new()),
                Some(resolve::Res::Local(_)) | None => Ty::Err,
            };
            (ty, Vec::new())
        }
        TyKind::Param(name) => (Ty::Param(name.name.to_string()), Vec::new()),
        TyKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = ty_from_ast(names, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
    }
}

pub(super) fn ast_ty_def_cons(
    names: &IndexMap<NodeId, Res>,
    id: ItemId,
    def: &TyDef,
) -> (Ty, Vec<MissingTyError>) {
    let (input, errors) = ast_ty_def_base(names, def);
    let ty = Ty::Arrow(
        hir::CallableKind::Function,
        Box::new(input),
        Box::new(Ty::Udt(hir::Res::Item(id))),
        HashSet::new(),
    );
    (ty, errors)
}

pub(super) fn ast_ty_def_base(
    names: &IndexMap<NodeId, Res>,
    def: &TyDef,
) -> (Ty, Vec<MissingTyError>) {
    match &def.kind {
        TyDefKind::Field(_, ty) => ty_from_ast(names, ty),
        TyDefKind::Paren(inner) => ast_ty_def_base(names, inner),
        TyDefKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = ast_ty_def_base(names, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }

            (Ty::Tuple(tys), errors)
        }
    }
}

pub(super) fn ast_ty_def_fields(def: &TyDef) -> Vec<UdtField> {
    match &def.kind {
        TyDefKind::Field(Some(name), _) => {
            vec![UdtField {
                name: Rc::clone(&name.name),
                path: FieldPath::default(),
            }]
        }
        TyDefKind::Field(None, _) => Vec::new(),
        TyDefKind::Paren(inner) => ast_ty_def_fields(inner),
        TyDefKind::Tuple(items) => {
            let mut fields = Vec::new();
            for (index, item) in items.iter().enumerate() {
                for mut field in ast_ty_def_fields(item) {
                    field.path.indices.insert(0, index);
                    fields.push(field);
                }
            }
            fields
        }
    }
}

pub(super) fn ast_callable_ty(
    names: &IndexMap<NodeId, Res>,
    decl: &CallableDecl,
) -> (Ty, Vec<MissingTyError>) {
    let kind = callable_kind_from_ast(decl.kind);
    let (input, mut errors) = ast_pat_ty(names, &decl.input);
    let (output, output_errors) = ty_from_ast(names, &decl.output);
    errors.extend(output_errors);
    let functors = ast_callable_functors(decl);
    let ty = Ty::Arrow(kind, Box::new(input), Box::new(output), functors);
    (ty, errors)
}

pub(crate) fn ast_pat_ty(names: &IndexMap<NodeId, Res>, pat: &Pat) -> (Ty, Vec<MissingTyError>) {
    match &pat.kind {
        PatKind::Bind(_, None) | PatKind::Discard(None) | PatKind::Elided => {
            (Ty::Err, vec![MissingTyError(pat.span)])
        }
        PatKind::Bind(_, Some(ty)) | PatKind::Discard(Some(ty)) => ty_from_ast(names, ty),
        PatKind::Paren(inner) => ast_pat_ty(names, inner),
        PatKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = ast_pat_ty(names, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
    }
}

pub(super) fn ast_callable_functors(decl: &CallableDecl) -> HashSet<Functor> {
    let mut functors = decl
        .functors
        .as_ref()
        .map_or(HashSet::new(), eval_functor_expr);

    if let CallableBody::Specs(specs) = &decl.body {
        for spec in specs {
            match spec.spec {
                Spec::Body => {}
                Spec::Adj => functors.extend([Functor::Adj]),
                Spec::Ctl => functors.extend([Functor::Ctl]),
                Spec::CtlAdj => functors.extend([Functor::Adj, Functor::Ctl]),
            }
        }
    }

    functors
}

pub(super) fn callable_kind_from_ast(kind: CallableKind) -> hir::CallableKind {
    match kind {
        CallableKind::Function => hir::CallableKind::Function,
        CallableKind::Operation => hir::CallableKind::Operation,
    }
}

pub(crate) fn eval_functor_expr(expr: &FunctorExpr) -> HashSet<Functor> {
    match &expr.kind {
        FunctorExprKind::BinOp(op, lhs, rhs) => {
            let mut functors = eval_functor_expr(lhs);
            let rhs_functors = eval_functor_expr(rhs);
            match op {
                SetOp::Union => functors.extend(rhs_functors),
                SetOp::Intersect => functors.retain(|f| rhs_functors.contains(f)),
            }
            functors
        }
        FunctorExprKind::Lit(ast::Functor::Adj) => [Functor::Adj].into(),
        FunctorExprKind::Lit(ast::Functor::Ctl) => [Functor::Ctl].into(),
        FunctorExprKind::Paren(inner) => eval_functor_expr(inner),
    }
}
