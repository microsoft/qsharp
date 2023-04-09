// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    infer::{self, SpecImpl},
    solve::Ty,
    Error, ErrorKind, Tys,
};
use crate::{
    compile::PackageId,
    resolve::{DefId, PackageSrc, Resolutions},
};
use qsc_ast::{
    ast::{
        self, CallableBody, CallableDecl, Functor, FunctorExpr, Package, Pat, PatKind, Span, Spec,
        SpecBody, TyKind,
    },
    visit::Visitor,
};
use std::collections::{HashMap, HashSet};

pub(crate) struct Checker<'a> {
    resolutions: &'a Resolutions,
    globals: HashMap<DefId, Ty>,
    tys: Tys,
    errors: Vec<Error>,
}

impl Checker<'_> {
    pub(crate) fn into_tys(self) -> (Tys, Vec<Error>) {
        (self.tys, self.errors)
    }
}

impl Visitor<'_> for Checker<'_> {
    fn visit_package(&mut self, package: &Package) {
        for namespace in &package.namespaces {
            self.visit_namespace(namespace);
        }

        if let Some(entry) = &package.entry {
            let (tys, errors) = infer::entry_expr(self.resolutions, &self.globals, entry);
            self.tys.extend(tys);
            self.errors.extend(errors);
        }
    }

    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        let id = DefId {
            package: PackageSrc::Local,
            node: decl.name.id,
        };
        let ty = self.globals.get(&id).expect("callable should have type");
        let Ty::Arrow(_, _, output, functors) = ty else { panic!("callable should have arrow type") };

        match &decl.body {
            CallableBody::Block(block) => {
                let spec = SpecImpl {
                    kind: Spec::Body,
                    input: None,
                    callable_input: &decl.input,
                    output,
                    output_span: decl.output.span,
                    functors,
                    block,
                };
                let (tys, errors) = infer::spec(self.resolutions, &self.globals, spec);
                self.tys.extend(tys);
                self.errors.extend(errors);
            }
            CallableBody::Specs(specs) => {
                for spec in specs {
                    match &spec.body {
                        SpecBody::Gen(_) => {}
                        SpecBody::Impl(input, block) => {
                            let spec = SpecImpl {
                                kind: spec.spec,
                                input: Some(input),
                                callable_input: &decl.input,
                                output,
                                output_span: decl.output.span,
                                functors,
                                block,
                            };
                            let (tys, errors) = infer::spec(self.resolutions, &self.globals, spec);
                            self.tys.extend(tys);
                            self.errors.extend(errors);
                        }
                    }
                }
            }
        }
    }
}

pub(crate) struct GlobalTable<'a> {
    resolutions: &'a Resolutions,
    globals: HashMap<DefId, Ty>,
    package: PackageSrc,
    errors: Vec<Error>,
}

impl<'a> GlobalTable<'a> {
    pub(crate) fn new(resolutions: &'a Resolutions) -> Self {
        Self {
            resolutions,
            globals: HashMap::new(),
            package: PackageSrc::Local,
            errors: Vec::new(),
        }
    }

    pub(crate) fn set_package(&mut self, package: PackageId) {
        self.package = PackageSrc::Extern(package);
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
        let (ty, errors) = callable_ty(self.resolutions, decl);
        let id = DefId {
            package: self.package,
            node: decl.name.id,
        };
        self.globals.insert(id, ty);
        for error in errors {
            self.errors.push(Error(ErrorKind::MissingItemTy(error.0)));
        }
    }
}

struct MissingTyError(Span);

fn callable_ty(resolutions: &Resolutions, decl: &CallableDecl) -> (Ty, Vec<MissingTyError>) {
    let (input, mut errors) = try_pat_ty(resolutions, &decl.input);
    let (output, output_errors) = try_convert_ty(resolutions, &decl.output);
    errors.extend(output_errors);

    let sig_functors = decl
        .functors
        .as_ref()
        .map_or(HashSet::new(), FunctorExpr::to_set);
    let body_functors = match &decl.body {
        CallableBody::Block(_) => HashSet::new(),
        CallableBody::Specs(specs) => specs
            .iter()
            .flat_map(|spec| match spec.spec {
                Spec::Body => Vec::new(),
                Spec::Adj => vec![Functor::Adj],
                Spec::Ctl => vec![Functor::Ctl],
                Spec::CtlAdj => vec![Functor::Adj, Functor::Ctl],
            })
            .collect(),
    };

    let functors = sig_functors.union(&body_functors).copied().collect();
    let ty = Ty::Arrow(decl.kind, Box::new(input), Box::new(output), functors);
    (ty, errors)
}

fn try_convert_ty(resolutions: &Resolutions, ty: &ast::Ty) -> (Ty, Vec<MissingTyError>) {
    match &ty.kind {
        TyKind::Array(item) => {
            let (new_item, errors) = try_convert_ty(resolutions, item);
            (Ty::Array(Box::new(new_item)), errors)
        }
        TyKind::Arrow(kind, input, output, functors) => {
            let (input, mut errors) = try_convert_ty(resolutions, input);
            let (output, output_errors) = try_convert_ty(resolutions, output);
            errors.extend(output_errors);
            let functors = functors
                .as_ref()
                .map_or(HashSet::new(), FunctorExpr::to_set);
            let ty = Ty::Arrow(*kind, Box::new(input), Box::new(output), functors);
            (ty, errors)
        }
        TyKind::Hole => (Ty::Err, vec![MissingTyError(ty.span)]),
        TyKind::Paren(inner) => try_convert_ty(resolutions, inner),
        TyKind::Path(path) => (
            resolutions
                .get(&path.id)
                .copied()
                .map_or(Ty::Err, Ty::DefId),
            Vec::new(),
        ),
        &TyKind::Prim(prim) => (Ty::Prim(prim), Vec::new()),
        TyKind::Tuple(items) => {
            let mut new_items = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (new_item, item_errors) = try_convert_ty(resolutions, item);
                new_items.push(new_item);
                errors.extend(item_errors);
            }
            (Ty::Tuple(new_items), errors)
        }
        TyKind::Var(name) => (Ty::Param(name.name.clone()), Vec::new()),
    }
}

fn try_pat_ty(resolutions: &Resolutions, pat: &Pat) -> (Ty, Vec<MissingTyError>) {
    match &pat.kind {
        PatKind::Bind(_, None) | PatKind::Discard(None) | PatKind::Elided => {
            (Ty::Err, vec![MissingTyError(pat.span)])
        }
        PatKind::Bind(_, Some(ty)) | PatKind::Discard(Some(ty)) => try_convert_ty(resolutions, ty),
        PatKind::Paren(inner) => try_pat_ty(resolutions, inner),
        PatKind::Tuple(items) => {
            let mut new_items = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (new_item, item_errors) = try_pat_ty(resolutions, item);
                new_items.push(new_item);
                errors.extend(item_errors);
            }
            (Ty::Tuple(new_items), errors)
        }
    }
}
