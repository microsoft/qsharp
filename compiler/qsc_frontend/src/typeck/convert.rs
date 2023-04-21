// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::ast;
use qsc_data_structures::span::Span;
use qsc_hir::hir::{self, Functor, Ty};
use std::collections::HashSet;

pub(crate) struct MissingTyError(pub(super) Span);

pub(crate) fn ty_from_ast(ty: &ast::Ty) -> (Ty, Vec<MissingTyError>) {
    match &ty.kind {
        ast::TyKind::Array(item) => {
            let (item, errors) = ty_from_ast(item);
            (Ty::Array(Box::new(item)), errors)
        }
        ast::TyKind::Arrow(kind, input, output, functors) => {
            let (input, mut errors) = ty_from_ast(input);
            let (output, output_errors) = ty_from_ast(output);
            errors.extend(output_errors);
            let functors = functors.as_ref().map_or(HashSet::new(), |f| {
                f.to_set().into_iter().map(functor_from_ast).collect()
            });
            let ty = Ty::Arrow(
                callable_kind_from_ast(*kind),
                Box::new(input),
                Box::new(output),
                functors,
            );
            (ty, errors)
        }
        ast::TyKind::Hole => (Ty::Err, vec![MissingTyError(ty.span)]),
        ast::TyKind::Paren(inner) => ty_from_ast(inner),
        ast::TyKind::Path(_) => (Ty::Err, Vec::new()), // TODO: Resolve user-defined types.
        ast::TyKind::Param(name) => (Ty::Param(name.name.clone()), Vec::new()),
        &ast::TyKind::Prim(prim) => (Ty::Prim(prim_from_ast(prim)), Vec::new()),
        ast::TyKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = ty_from_ast(item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
    }
}

pub(super) fn ast_callable_ty(decl: &ast::CallableDecl) -> (Ty, Vec<MissingTyError>) {
    let kind = callable_kind_from_ast(decl.kind);
    let (input, mut errors) = ast_pat_ty(&decl.input);
    let (output, output_errors) = ty_from_ast(&decl.output);
    errors.extend(output_errors);
    let functors = ast_callable_functors(decl);
    let ty = Ty::Arrow(kind, Box::new(input), Box::new(output), functors);
    (ty, errors)
}

pub(super) fn hir_callable_ty(decl: &hir::CallableDecl) -> Ty {
    Ty::Arrow(
        decl.kind,
        Box::new(decl.input.ty.clone()),
        Box::new(decl.output.clone()),
        hir_callable_functors(decl),
    )
}

pub(crate) fn ast_pat_ty(pat: &ast::Pat) -> (Ty, Vec<MissingTyError>) {
    match &pat.kind {
        ast::PatKind::Bind(_, None) | ast::PatKind::Discard(None) | ast::PatKind::Elided => {
            (Ty::Err, vec![MissingTyError(pat.span)])
        }
        ast::PatKind::Bind(_, Some(ty)) | ast::PatKind::Discard(Some(ty)) => ty_from_ast(ty),
        ast::PatKind::Paren(inner) => ast_pat_ty(inner),
        ast::PatKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = ast_pat_ty(item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
    }
}

pub(super) fn ast_callable_functors(decl: &ast::CallableDecl) -> HashSet<Functor> {
    let mut functors = decl.functors.as_ref().map_or(HashSet::new(), |f| {
        f.to_set().into_iter().map(functor_from_ast).collect()
    });

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

fn hir_callable_functors(decl: &hir::CallableDecl) -> HashSet<Functor> {
    let mut functors = decl.functors.as_ref().map_or(HashSet::new(), |f| {
        f.to_set().into_iter().map(Into::into).collect()
    });

    if let hir::CallableBody::Specs(specs) = &decl.body {
        for spec in specs {
            match spec.spec {
                hir::Spec::Body => {}
                hir::Spec::Adj => functors.extend([Functor::Adj]),
                hir::Spec::Ctl => functors.extend([Functor::Ctl]),
                hir::Spec::CtlAdj => functors.extend([Functor::Adj, Functor::Ctl]),
            }
        }
    }

    functors
}

pub(super) fn functor_from_ast(functor: ast::Functor) -> hir::Functor {
    match functor {
        ast::Functor::Adj => hir::Functor::Adj,
        ast::Functor::Ctl => hir::Functor::Ctl,
    }
}

pub(super) fn prim_from_ast(prim: ast::PrimTy) -> hir::PrimTy {
    match prim {
        ast::PrimTy::BigInt => hir::PrimTy::BigInt,
        ast::PrimTy::Bool => hir::PrimTy::Bool,
        ast::PrimTy::Double => hir::PrimTy::Double,
        ast::PrimTy::Int => hir::PrimTy::Int,
        ast::PrimTy::Pauli => hir::PrimTy::Pauli,
        ast::PrimTy::Qubit => hir::PrimTy::Qubit,
        ast::PrimTy::Range => hir::PrimTy::Range,
        ast::PrimTy::Result => hir::PrimTy::Result,
        ast::PrimTy::String => hir::PrimTy::String,
    }
}

pub(super) fn callable_kind_from_ast(kind: ast::CallableKind) -> hir::CallableKind {
    match kind {
        ast::CallableKind::Function => hir::CallableKind::Function,
        ast::CallableKind::Operation => hir::CallableKind::Operation,
    }
}
