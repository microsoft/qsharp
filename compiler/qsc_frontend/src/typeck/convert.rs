// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::resolve::{self, Resolutions};
use qsc_ast::ast;
use qsc_data_structures::span::Span;
use qsc_hir::hir::{self, FieldPath, Functor, ItemId, Ty, UdtField};
use std::{collections::HashSet, rc::Rc};

pub(crate) struct MissingTyError(pub(super) Span);

pub(crate) fn ty_from_ast(resolutions: &Resolutions, ty: &ast::Ty) -> (Ty, Vec<MissingTyError>) {
    match &ty.kind {
        ast::TyKind::Array(item) => {
            let (item, errors) = ty_from_ast(resolutions, item);
            (Ty::Array(Box::new(item)), errors)
        }
        ast::TyKind::Arrow(kind, input, output, functors) => {
            let (input, mut errors) = ty_from_ast(resolutions, input);
            let (output, output_errors) = ty_from_ast(resolutions, output);
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
        ast::TyKind::Hole => (Ty::Err, vec![MissingTyError(ty.span)]),
        ast::TyKind::Paren(inner) => ty_from_ast(resolutions, inner),
        ast::TyKind::Path(path) => {
            let ty = match resolutions.get(path.id) {
                Some(&resolve::Res::Item(item)) => Ty::Udt(hir::Res::Item(item)),
                Some(&resolve::Res::PrimTy(prim)) => Ty::Prim(prim),
                Some(resolve::Res::UnitTy) => Ty::Tuple(Vec::new()),
                Some(resolve::Res::Local(_)) | None => Ty::Err,
            };
            (ty, Vec::new())
        }
        ast::TyKind::Param(name) => (Ty::Param(name.name.to_string()), Vec::new()),
        ast::TyKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = ty_from_ast(resolutions, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
    }
}

pub(super) fn ast_ty_def_cons(
    resolutions: &Resolutions,
    id: ItemId,
    def: &ast::TyDef,
) -> (Ty, Vec<MissingTyError>) {
    let (input, errors) = ast_ty_def_base(resolutions, def);
    let ty = Ty::Arrow(
        hir::CallableKind::Function,
        Box::new(input),
        Box::new(Ty::Udt(hir::Res::Item(id))),
        HashSet::new(),
    );
    (ty, errors)
}

pub(super) fn ast_ty_def_base(
    resolutions: &Resolutions,
    def: &ast::TyDef,
) -> (Ty, Vec<MissingTyError>) {
    match &def.kind {
        ast::TyDefKind::Field(_, ty) => ty_from_ast(resolutions, ty),
        ast::TyDefKind::Paren(inner) => ast_ty_def_base(resolutions, inner),
        ast::TyDefKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = ast_ty_def_base(resolutions, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }

            (Ty::Tuple(tys), errors)
        }
    }
}

pub(super) fn ast_ty_def_fields(def: &ast::TyDef) -> Vec<UdtField> {
    match &def.kind {
        ast::TyDefKind::Field(Some(name), _) => {
            vec![UdtField {
                name: Rc::clone(&name.name),
                path: FieldPath::default(),
            }]
        }
        ast::TyDefKind::Field(None, _) => Vec::new(),
        ast::TyDefKind::Paren(inner) => ast_ty_def_fields(inner),
        ast::TyDefKind::Tuple(items) => {
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
    resolutions: &Resolutions,
    decl: &ast::CallableDecl,
) -> (Ty, Vec<MissingTyError>) {
    let kind = callable_kind_from_ast(decl.kind);
    let (input, mut errors) = ast_pat_ty(resolutions, &decl.input);
    let (output, output_errors) = ty_from_ast(resolutions, &decl.output);
    errors.extend(output_errors);
    let functors = ast_callable_functors(decl);
    let ty = Ty::Arrow(kind, Box::new(input), Box::new(output), functors);
    (ty, errors)
}

pub(crate) fn ast_pat_ty(resolutions: &Resolutions, pat: &ast::Pat) -> (Ty, Vec<MissingTyError>) {
    match &pat.kind {
        ast::PatKind::Bind(_, None) | ast::PatKind::Discard(None) | ast::PatKind::Elided => {
            (Ty::Err, vec![MissingTyError(pat.span)])
        }
        ast::PatKind::Bind(_, Some(ty)) | ast::PatKind::Discard(Some(ty)) => {
            ty_from_ast(resolutions, ty)
        }
        ast::PatKind::Paren(inner) => ast_pat_ty(resolutions, inner),
        ast::PatKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = ast_pat_ty(resolutions, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
    }
}

pub(super) fn ast_callable_functors(decl: &ast::CallableDecl) -> HashSet<Functor> {
    let mut functors = decl
        .functors
        .as_ref()
        .map_or(HashSet::new(), eval_functor_expr);

    if let ast::CallableBody::Specs(specs) = &decl.body {
        for spec in specs {
            match spec.spec {
                ast::Spec::Body => {}
                ast::Spec::Adj => functors.extend([Functor::Adj]),
                ast::Spec::Ctl => functors.extend([Functor::Ctl]),
                ast::Spec::CtlAdj => functors.extend([Functor::Adj, Functor::Ctl]),
            }
        }
    }

    functors
}

pub(super) fn callable_kind_from_ast(kind: ast::CallableKind) -> hir::CallableKind {
    match kind {
        ast::CallableKind::Function => hir::CallableKind::Function,
        ast::CallableKind::Operation => hir::CallableKind::Operation,
    }
}

pub(crate) fn eval_functor_expr(expr: &ast::FunctorExpr) -> HashSet<Functor> {
    match &expr.kind {
        ast::FunctorExprKind::BinOp(op, lhs, rhs) => {
            let mut functors = eval_functor_expr(lhs);
            let rhs_functors = eval_functor_expr(rhs);
            match op {
                ast::SetOp::Union => functors.extend(rhs_functors),
                ast::SetOp::Intersect => functors.retain(|f| rhs_functors.contains(f)),
            }
            functors
        }
        ast::FunctorExprKind::Lit(ast::Functor::Adj) => [hir::Functor::Adj].into(),
        ast::FunctorExprKind::Lit(ast::Functor::Ctl) => [hir::Functor::Ctl].into(),
        ast::FunctorExprKind::Paren(inner) => eval_functor_expr(inner),
    }
}
