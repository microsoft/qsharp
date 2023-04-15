// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    infer::{Functor, Ty},
    rules::{self, SpecImpl},
    Error, ErrorKind, Tys,
};
use crate::{
    compile::PackageId,
    resolve::{Link, Resolutions},
};
use qsc_ast::{
    ast::{
        self, CallableBody, CallableDecl, Expr, FunctorExpr, NodeId, Package, Pat, PatKind, Spec,
        SpecBody, TyKind,
    },
    visit::Visitor,
};
use qsc_data_structures::span::Span;
use qsc_hir::{hir, visit as hir_visit};
use std::{
    collections::{HashMap, HashSet},
    convert::Into,
};

pub(crate) struct GlobalTable<'a> {
    resolutions: &'a Resolutions<NodeId>,
    globals: HashMap<Link<NodeId>, Ty>,
    package: Option<PackageId>,
    errors: Vec<Error>,
}

impl<'a> GlobalTable<'a> {
    pub(crate) fn new(resolutions: &'a Resolutions<NodeId>) -> Self {
        Self {
            resolutions,
            globals: HashMap::new(),
            package: None,
            errors: Vec::new(),
        }
    }

    pub(crate) fn set_package(&mut self, package: PackageId) {
        self.package = Some(package);
    }

    pub(crate) fn into_checker(self) -> Checker<'a> {
        Checker {
            resolutions: self.resolutions,
            globals: self.globals,
            tys: Tys::new(),
            errors: self.errors,
        }
    }
}

impl Visitor<'_> for GlobalTable<'_> {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        assert!(
            self.package.is_none(),
            "AST callable should only be in local package"
        );
        let (ty, errors) = callable_ty(decl);
        self.globals.insert(Link::Internal(decl.name.id), ty);
        for MissingTyError(span) in errors {
            self.errors.push(Error(ErrorKind::MissingItemTy(span)));
        }
    }
}

impl hir_visit::Visitor<'_> for GlobalTable<'_> {
    fn visit_callable_decl(&mut self, decl: &hir::CallableDecl) {
        let package = self
            .package
            .expect("HIR callable should only be in external package");
        let (ty, errors) = hir_callable_ty(decl);
        self.globals
            .insert(Link::External(package, decl.name.id), ty);
        for MissingTyError(span) in errors {
            self.errors.push(Error(ErrorKind::MissingItemTy(span)));
        }
    }
}

pub(crate) struct Checker<'a> {
    resolutions: &'a Resolutions<NodeId>,
    globals: HashMap<Link<NodeId>, Ty>,
    tys: Tys<NodeId>,
    errors: Vec<Error>,
}

impl Checker<'_> {
    pub(crate) fn into_tys(self) -> (Tys<NodeId>, Vec<Error>) {
        (self.tys, self.errors)
    }

    fn check_callable_signature(&mut self, decl: &CallableDecl) {
        if !callable_functors(decl).is_empty() {
            match &decl.output.kind {
                TyKind::Tuple(items) if items.is_empty() => {}
                _ => self.errors.push(Error(ErrorKind::TypeMismatch(
                    Ty::UNIT,
                    convert_ty(&decl.output).0,
                    decl.output.span,
                ))),
            }
        }
    }

    fn check_spec(&mut self, spec: SpecImpl) {
        let errors = rules::spec(self.resolutions, &self.globals, &mut self.tys, spec);
        self.errors.extend(errors);
    }

    fn check_entry_expr(&mut self, entry: &Expr) {
        let errors = rules::entry_expr(self.resolutions, &self.globals, &mut self.tys, entry);
        self.errors.extend(errors);
    }
}

impl Visitor<'_> for Checker<'_> {
    fn visit_package(&mut self, package: &Package) {
        for namespace in &package.namespaces {
            self.visit_namespace(namespace);
        }
        if let Some(entry) = &package.entry {
            self.check_entry_expr(entry);
        }
    }

    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        self.tys.insert(decl.name.id, callable_ty(decl).0);
        self.check_callable_signature(decl);

        let output = convert_ty(&decl.output).0;
        match &decl.body {
            CallableBody::Block(block) => self.check_spec(SpecImpl {
                spec: Spec::Body,
                callable_input: &decl.input,
                spec_input: None,
                output: &output,
                block,
            }),
            CallableBody::Specs(specs) => {
                for spec in specs {
                    if let SpecBody::Impl(input, block) = &spec.body {
                        self.check_spec(SpecImpl {
                            spec: spec.spec,
                            callable_input: &decl.input,
                            spec_input: Some(input),
                            output: &output,
                            block,
                        });
                    }
                }
            }
        }
    }
}

struct MissingTyError(Span);

fn convert_ty(ty: &ast::Ty) -> (Ty, Vec<MissingTyError>) {
    match &ty.kind {
        TyKind::Array(item) => {
            let (item, errors) = convert_ty(item);
            (Ty::Array(Box::new(item)), errors)
        }
        TyKind::Arrow(kind, input, output, functors) => {
            let (input, mut errors) = convert_ty(input);
            let (output, output_errors) = convert_ty(output);
            errors.extend(output_errors);

            let functors = functors
                .as_ref()
                .map_or(HashSet::new(), FunctorExpr::to_set)
                .into_iter()
                .map(Into::into)
                .collect();

            (
                Ty::Arrow(kind.into(), Box::new(input), Box::new(output), functors),
                errors,
            )
        }
        TyKind::Hole => (Ty::Err, vec![MissingTyError(ty.span)]),
        TyKind::Paren(inner) => convert_ty(inner),
        TyKind::Path(_) => (Ty::Err, Vec::new()), // TODO: Resolve user-defined types.
        &TyKind::Prim(prim) => (Ty::Prim(prim.into()), Vec::new()),
        TyKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = convert_ty(item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
        TyKind::Var(name) => (Ty::Param(name.name.clone()), Vec::new()),
    }
}

fn convert_hir_ty(ty: &hir::Ty) -> (Ty, Vec<MissingTyError>) {
    match &ty.kind {
        hir::TyKind::Array(item) => {
            let (item, errors) = convert_hir_ty(item);
            (Ty::Array(Box::new(item)), errors)
        }
        hir::TyKind::Arrow(kind, input, output, functors) => {
            let (input, mut errors) = convert_hir_ty(input);
            let (output, output_errors) = convert_hir_ty(output);
            errors.extend(output_errors);

            let functors = functors
                .as_ref()
                .map_or(HashSet::new(), hir::FunctorExpr::to_set)
                .into_iter()
                .map(Into::into)
                .collect();

            (
                Ty::Arrow(kind.into(), Box::new(input), Box::new(output), functors),
                errors,
            )
        }
        hir::TyKind::Hole => (Ty::Err, vec![MissingTyError(ty.span)]),
        hir::TyKind::Paren(inner) => convert_hir_ty(inner),
        hir::TyKind::Path(_) => (Ty::Err, Vec::new()), // TODO: Resolve user-defined types.
        &hir::TyKind::Prim(prim) => (Ty::Prim(prim.into()), Vec::new()),
        hir::TyKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = convert_hir_ty(item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
        hir::TyKind::Var(name) => (Ty::Param(name.name.clone()), Vec::new()),
    }
}

fn pat_ty(pat: &Pat) -> (Ty, Vec<MissingTyError>) {
    match &pat.kind {
        PatKind::Bind(_, None) | PatKind::Discard(None) | PatKind::Elided => {
            (Ty::Err, vec![MissingTyError(pat.span)])
        }
        PatKind::Bind(_, Some(ty)) | PatKind::Discard(Some(ty)) => convert_ty(ty),
        PatKind::Paren(inner) => pat_ty(inner),
        PatKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = pat_ty(item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
    }
}

fn hir_pat_ty(pat: &hir::Pat) -> (Ty, Vec<MissingTyError>) {
    match &pat.kind {
        hir::PatKind::Bind(_, None) | hir::PatKind::Discard(None) | hir::PatKind::Elided => {
            (Ty::Err, vec![MissingTyError(pat.span)])
        }
        hir::PatKind::Bind(_, Some(ty)) | hir::PatKind::Discard(Some(ty)) => convert_hir_ty(ty),
        hir::PatKind::Paren(inner) => hir_pat_ty(inner),
        hir::PatKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = hir_pat_ty(item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
    }
}

fn callable_ty(decl: &CallableDecl) -> (Ty, Vec<MissingTyError>) {
    let (input, mut errors) = pat_ty(&decl.input);
    let (output, output_errors) = convert_ty(&decl.output);
    errors.extend(output_errors);
    let functors = callable_functors(decl);

    (
        Ty::Arrow(
            decl.kind.into(),
            Box::new(input),
            Box::new(output),
            functors,
        ),
        errors,
    )
}

fn hir_callable_ty(decl: &hir::CallableDecl) -> (Ty, Vec<MissingTyError>) {
    let (input, mut errors) = hir_pat_ty(&decl.input);
    let (output, output_errors) = convert_hir_ty(&decl.output);
    errors.extend(output_errors);
    let functors = hir_callable_functors(decl);

    (
        Ty::Arrow(
            decl.kind.into(),
            Box::new(input),
            Box::new(output),
            functors,
        ),
        errors,
    )
}

fn callable_functors(decl: &CallableDecl) -> HashSet<Functor> {
    let mut functors: HashSet<_> = decl
        .functors
        .as_ref()
        .map_or(HashSet::new(), FunctorExpr::to_set)
        .into_iter()
        .map(Into::into)
        .collect();

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

fn hir_callable_functors(decl: &hir::CallableDecl) -> HashSet<Functor> {
    let mut functors: HashSet<_> = decl
        .functors
        .as_ref()
        .map_or(HashSet::new(), hir::FunctorExpr::to_set)
        .into_iter()
        .map(Into::into)
        .collect();

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
