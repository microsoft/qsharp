// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::resolve::{self, Names};
use qsc_ast::ast::{
    self, CallableBody, CallableDecl, CallableKind, FunctorExpr, FunctorExprKind, Pat, PatKind,
    SetOp, Spec, TyDef, TyDefKind, TyKind,
};
use qsc_data_structures::span::Span;
use qsc_hir::hir::{self, FieldPath, FunctorSet, ItemId, Ty, UdtField};
use std::rc::Rc;

pub(crate) struct MissingTyError(pub(super) Span);

pub(crate) fn ty_from_ast(names: &Names, ty: &ast::Ty) -> (Ty, Vec<MissingTyError>) {
    match &ty.kind {
        TyKind::Array(item) => {
            let (item, errors) = ty_from_ast(names, item);
            (Ty::Array(Box::new(item)), errors)
        }
        TyKind::Arrow(kind, input, output, functors) => {
            let (input, mut errors) = ty_from_ast(names, input);
            let (output, output_errors) = ty_from_ast(names, output);
            errors.extend(output_errors);
            let functors = functors
                .as_ref()
                .map_or(FunctorSet::Empty, eval_functor_expr);
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

pub(super) fn ast_ty_def_cons(names: &Names, id: ItemId, def: &TyDef) -> (Ty, Vec<MissingTyError>) {
    let (input, errors) = ast_ty_def_base(names, def);
    let ty = Ty::Arrow(
        hir::CallableKind::Function,
        Box::new(input),
        Box::new(Ty::Udt(hir::Res::Item(id))),
        FunctorSet::Empty,
    );
    (ty, errors)
}

pub(super) fn ast_ty_def_base(names: &Names, def: &TyDef) -> (Ty, Vec<MissingTyError>) {
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

pub(super) fn ast_callable_ty(names: &Names, decl: &CallableDecl) -> (Ty, Vec<MissingTyError>) {
    let kind = callable_kind_from_ast(decl.kind);
    let (input, mut errors) = ast_pat_ty(names, &decl.input);
    let (output, output_errors) = ty_from_ast(names, &decl.output);
    errors.extend(output_errors);
    let functors = ast_callable_functors(decl);
    let ty = Ty::Arrow(kind, Box::new(input), Box::new(output), functors);
    (ty, errors)
}

pub(crate) fn ast_pat_ty(names: &Names, pat: &Pat) -> (Ty, Vec<MissingTyError>) {
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

pub(crate) fn ast_callable_functors(decl: &CallableDecl) -> FunctorSet {
    let mut functors = decl
        .functors
        .as_ref()
        .map_or(FunctorSet::Empty, eval_functor_expr);

    if let CallableBody::Specs(specs) = &decl.body {
        for spec in specs {
            let spec_functors = match spec.spec {
                Spec::Body => FunctorSet::Empty,
                Spec::Adj => FunctorSet::Adj,
                Spec::Ctl => FunctorSet::Ctl,
                Spec::CtlAdj => FunctorSet::AdjCtl,
            };
            functors = functors
                .union(&spec_functors)
                .expect("union on known functors should always succeed");
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

pub(crate) fn eval_functor_expr(expr: &FunctorExpr) -> FunctorSet {
    match &expr.kind {
        FunctorExprKind::BinOp(op, lhs, rhs) => {
            let lhs_functors = eval_functor_expr(lhs);
            let rhs_functors = eval_functor_expr(rhs);
            match op {
                SetOp::Union => lhs_functors.union(&rhs_functors),
                SetOp::Intersect => lhs_functors.intersect(&rhs_functors),
            }
            .expect("union or intersect on set from functor expression should always succeed")
        }
        FunctorExprKind::Lit(ast::Functor::Adj) => FunctorSet::Adj,
        FunctorExprKind::Lit(ast::Functor::Ctl) => FunctorSet::Ctl,
        FunctorExprKind::Paren(inner) => eval_functor_expr(inner),
    }
}
