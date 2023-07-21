// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::resolve::{self, Names};
use qsc_ast::ast::{
    self, CallableBody, CallableDecl, CallableKind, FunctorExpr, FunctorExprKind, Ident, Pat,
    PatKind, SetOp, Spec, TyDef, TyDefKind, TyKind,
};
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir,
    ty::{
        Arrow, FunctorSet, FunctorSetValue, GenericParam, ParamId, Scheme, Ty, UdtDef, UdtDefKind,
        UdtField,
    },
};

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
                // a path should never resolve to a parameter,
                // as there is a syntactic difference between
                // paths and parameters.
                // So realistically, by construction, `Param` here is unreachable.
                Some(resolve::Res::Local(_) | resolve::Res::Param(_)) => unreachable!(
                    "A path should never resolve \
                    to a local or a parameter, as there is syntactic differentiation."
                ),
                None => Ty::Err,
            };
            (ty, Vec::new())
        }
        TyKind::Param(name) => match names.get(name.id) {
            Some(resolve::Res::Param(id)) => (Ty::Param(*id), Vec::new()),
            Some(_) => unreachable!(
                "A parameter should never resolve to a non-parameter type, as there \
                    is syntactic differentiation"
            ),
            None => (Ty::Err, Vec::new()),
        },
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
    id: hir::ItemId,
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

fn ast_ty_def_base(names: &Names, def: &TyDef) -> (Ty, Vec<MissingTyError>) {
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

pub(super) fn ast_ty_def(names: &Names, def: &TyDef) -> (UdtDef, Vec<MissingTyError>) {
    if let TyDefKind::Paren(inner) = &*def.kind {
        return ast_ty_def(names, inner);
    }

    let mut errors = Vec::new();
    let def = UdtDef {
        span: def.span,
        kind: match &*def.kind {
            TyDefKind::Field(name, ty) => {
                let (ty, item_errors) = ty_from_ast(names, ty);
                errors.extend(item_errors);
                let (name_span, name) = match name {
                    Some(name) => (Some(name.span), Some(name.name.clone())),
                    None => (None, None),
                };
                let field = UdtField {
                    name_span,
                    name,
                    ty,
                };
                UdtDefKind::Field(field)
            }
            TyDefKind::Paren(_) => unreachable!("parentheses should be removed earlier"),
            TyDefKind::Tuple(items) => UdtDefKind::Tuple(
                items
                    .iter()
                    .map(|i| {
                        let (item_def, item_errors) = ast_ty_def(names, i);
                        errors.extend(item_errors);
                        item_def
                    })
                    .collect(),
            ),
        },
    };

    (def, errors)
}

pub(super) fn ast_callable_scheme(
    names: &Names,
    callable: &CallableDecl,
) -> (Scheme, Vec<MissingTyError>) {
    let kind = callable_kind_from_ast(callable.kind);
    let (mut input, mut errors) = ast_pat_ty(names, &callable.input);
    let (output, output_errors) = ty_from_ast(names, &callable.output);
    errors.extend(output_errors);

    let mut params = ast_callable_generics(&callable.generics);
    let mut functor_params = synthesize_functor_params(&mut params.len().into(), &mut input);
    params.append(&mut functor_params);

    let ty = Arrow {
        kind,
        input: Box::new(input),
        output: Box::new(output),
        functors: FunctorSet::Value(ast_callable_functors(callable)),
    };

    (Scheme::new(params, Box::new(ty)), errors)
}

pub(crate) fn synthesize_callable_generics(
    generics: &[Box<Ident>],
    input: &mut hir::Pat,
) -> Vec<GenericParam> {
    let mut params = ast_callable_generics(generics);
    let mut functor_params = synthesize_functor_params_in_pat(&mut params.len().into(), input);
    params.append(&mut functor_params);
    params
}

fn synthesize_functor_params(next_param: &mut ParamId, ty: &mut Ty) -> Vec<GenericParam> {
    match ty {
        Ty::Array(item) => synthesize_functor_params(next_param, item),
        Ty::Arrow(arrow) => match arrow.functors {
            FunctorSet::Value(functors) if arrow.kind == hir::CallableKind::Operation => {
                let param = GenericParam::Functor(functors);
                arrow.functors = FunctorSet::Param(*next_param);
                *next_param = next_param.successor();
                vec![param]
            }
            _ => Vec::new(),
        },
        Ty::Tuple(items) => items
            .iter_mut()
            .flat_map(|item| synthesize_functor_params(next_param, item))
            .collect(),
        Ty::Infer(_) | Ty::Param(_) | Ty::Prim(_) | Ty::Udt(_) | Ty::Err => Vec::new(),
    }
}

fn synthesize_functor_params_in_pat(
    next_param: &mut ParamId,
    pat: &mut hir::Pat,
) -> Vec<GenericParam> {
    match &mut pat.kind {
        hir::PatKind::Discard | hir::PatKind::Bind(_) => {
            synthesize_functor_params(next_param, &mut pat.ty)
        }
        hir::PatKind::Tuple(items) => {
            let mut params = Vec::new();
            for item in items.iter_mut() {
                params.append(&mut synthesize_functor_params_in_pat(next_param, item));
            }
            if !params.is_empty() {
                pat.ty = Ty::Tuple(items.iter().map(|i| i.ty.clone()).collect());
            }
            params
        }
    }
}

fn ast_callable_generics(generics: &[Box<Ident>]) -> Vec<GenericParam> {
    generics.iter().map(|_param| GenericParam::Ty).collect()
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

pub(crate) fn ast_callable_functors(callable: &CallableDecl) -> FunctorSetValue {
    let mut functors = callable
        .functors
        .as_ref()
        .map_or(FunctorSetValue::Empty, |f| eval_functor_expr(f.as_ref()));

    if let CallableBody::Specs(specs) = callable.body.as_ref() {
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
