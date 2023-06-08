// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::resolve::{self, Names};
use qsc_ast::ast::{
    self, CallableBody, CallableDecl, CallableKind, FunctorExpr, FunctorExprKind, Pat, PatKind,
    SetOp, Spec, TyDef, TyDefKind, TyKind,
};
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{self, FieldPath, ItemId},
    ty::{
        Arrow, FunctorSet, FunctorSetValue, GenericParam, ParamId, ParamKind, ParamName, Scheme,
        Ty, UdtField,
    },
};
use std::rc::Rc;

pub(crate) struct MissingTyError(pub(super) Span);

pub(crate) fn ty_from_ast(names: &Names, ty: &ast::Ty) -> (Ty, Vec<MissingTyError>) {
    match &*ty.kind {
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
                .map_or(FunctorSetValue::Empty, |f| eval_functor_expr(f.as_ref()));
            let ty = Ty::Arrow(Box::new(Arrow {
                kind: callable_kind_from_ast(*kind),
                input: Box::new(input),
                output: Box::new(output),
                functors: FunctorSet::Value(functors),
            }));
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
        TyKind::Param(name) => (Ty::Param((*name.name).into()), Vec::new()),
        TyKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items.iter() {
                let (item_ty, item_errors) = ty_from_ast(names, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
    }
}

pub(super) fn ast_ty_def_cons(
    names: &Names,
    id: ItemId,
    def: &TyDef,
) -> (Scheme, Vec<MissingTyError>) {
    let (input, errors) = ast_ty_def_base(names, def);
    let ty = Arrow {
        kind: hir::CallableKind::Function,
        input: Box::new(input),
        output: Box::new(Ty::Udt(hir::Res::Item(id))),
        functors: FunctorSet::Value(FunctorSetValue::Empty),
    };
    let scheme = Scheme::new(Vec::new(), Box::new(ty));
    (scheme, errors)
}

pub(super) fn ast_ty_def_base(names: &Names, def: &TyDef) -> (Ty, Vec<MissingTyError>) {
    match &*def.kind {
        TyDefKind::Field(_, ty) => ty_from_ast(names, ty),
        TyDefKind::Paren(inner) => ast_ty_def_base(names, inner),
        TyDefKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items.iter() {
                let (item_ty, item_errors) = ast_ty_def_base(names, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }

            (Ty::Tuple(tys), errors)
        }
    }
}

pub(super) fn ast_ty_def_fields(def: &TyDef) -> Vec<UdtField> {
    match &*def.kind {
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

pub(super) fn ast_callable_scheme(
    names: &Names,
    decl: &CallableDecl,
) -> (Scheme, Vec<MissingTyError>) {
    let kind = callable_kind_from_ast(decl.kind);
    let (mut input, mut errors) = ast_pat_ty(names, &decl.input);
    let functor_params = synthesize_functor_params_in_ty(&mut ParamId::default(), &mut input);
    let (output, output_errors) = ty_from_ast(names, &decl.output);
    errors.extend(output_errors);
    let functors = ast_callable_functors(decl);
    let ty = Arrow {
        kind,
        input: Box::new(input),
        output: Box::new(output),
        functors: FunctorSet::Value(functors),
    };

    let params = decl
        .generics
        .iter()
        .map(|param| GenericParam {
            name: ParamName::Symbol((*param.name).into()),
            kind: ParamKind::Ty,
        })
        .chain(functor_params)
        .collect();
    let scheme = Scheme::new(params, Box::new(ty));

    (scheme, errors)
}

fn synthesize_functor_params_in_ty(next_functor: &mut ParamId, ty: &mut Ty) -> Vec<GenericParam> {
    match ty {
        Ty::Array(item) => synthesize_functor_params_in_ty(next_functor, item),
        Ty::Arrow(arrow) if arrow.kind == hir::CallableKind::Operation => {
            let functors = arrow
                .functors
                .expect_value("type should have concrete functors");
            let param = GenericParam {
                name: ParamName::Id(*next_functor),
                kind: ParamKind::Functor(functors),
            };
            arrow.functors = FunctorSet::Param(*next_functor);
            *next_functor = next_functor.successor();
            vec![param]
        }
        Ty::Tuple(items) => items
            .iter_mut()
            .flat_map(|item| synthesize_functor_params_in_ty(next_functor, item))
            .collect(),
        Ty::Arrow(_) | Ty::Infer(_) | Ty::Param(_) | Ty::Prim(_) | Ty::Udt(_) | Ty::Err => {
            Vec::new()
        }
    }
}

pub(crate) fn synthesize_functor_params_in_pat(
    next_functor: &mut ParamId,
    pat: &mut hir::Pat,
) -> Vec<GenericParam> {
    match &mut pat.kind {
        hir::PatKind::Discard | hir::PatKind::Bind(_) | hir::PatKind::Elided => {
            synthesize_functor_params_in_ty(next_functor, &mut pat.ty)
        }
        hir::PatKind::Tuple(items) => {
            let mut params = Vec::new();
            for item in items.iter_mut() {
                params.append(&mut synthesize_functor_params_in_pat(next_functor, item));
            }
            pat.ty = Ty::Tuple(items.iter().map(|i| i.ty.clone()).collect());
            params
        }
    }
}

pub(crate) fn ast_pat_ty(names: &Names, pat: &Pat) -> (Ty, Vec<MissingTyError>) {
    match &*pat.kind {
        PatKind::Bind(_, None) | PatKind::Discard(None) | PatKind::Elided => {
            (Ty::Err, vec![MissingTyError(pat.span)])
        }
        PatKind::Bind(_, Some(ty)) | PatKind::Discard(Some(ty)) => ty_from_ast(names, ty),
        PatKind::Paren(inner) => ast_pat_ty(names, inner),
        PatKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items.iter() {
                let (item_ty, item_errors) = ast_pat_ty(names, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
    }
}

pub(crate) fn ast_callable_functors(decl: &CallableDecl) -> FunctorSetValue {
    let mut functors = decl
        .functors
        .as_ref()
        .map_or(FunctorSetValue::Empty, |f| eval_functor_expr(f.as_ref()));

    if let CallableBody::Specs(specs) = decl.body.as_ref() {
        for spec in specs.iter() {
            let spec_functors = match spec.spec {
                Spec::Body => FunctorSetValue::Empty,
                Spec::Adj => FunctorSetValue::Adj,
                Spec::Ctl => FunctorSetValue::Ctl,
                Spec::CtlAdj => FunctorSetValue::CtlAdj,
            };
            functors = functors.union(&spec_functors);
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

pub(crate) fn eval_functor_expr(expr: &FunctorExpr) -> FunctorSetValue {
    match expr.kind.as_ref() {
        FunctorExprKind::BinOp(op, lhs, rhs) => {
            let lhs_functors = eval_functor_expr(lhs);
            let rhs_functors = eval_functor_expr(rhs);
            match op {
                SetOp::Union => lhs_functors.union(&rhs_functors),
                SetOp::Intersect => lhs_functors.intersect(&rhs_functors),
            }
        }
        FunctorExprKind::Lit(ast::Functor::Adj) => FunctorSetValue::Adj,
        FunctorExprKind::Lit(ast::Functor::Ctl) => FunctorSetValue::Ctl,
        FunctorExprKind::Paren(inner) => eval_functor_expr(inner),
    }
}
