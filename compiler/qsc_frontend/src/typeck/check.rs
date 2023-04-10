// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    infer::Ty,
    rules::{self, SpecImpl},
    Error, ErrorKind, Tys,
};
use crate::{
    compile::PackageId,
    resolve::{DefId, PackageSrc, Resolutions},
};
use qsc_ast::{
    ast::{
        self, CallableBody, CallableDecl, Expr, Functor, FunctorExpr, Package, Pat, PatKind, Span,
        Spec, SpecBody, TyKind,
    },
    visit::Visitor,
};
use std::collections::{HashMap, HashSet};

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
        for MissingTyError(span) in errors {
            self.errors.push(Error(ErrorKind::MissingItemTy(span)));
        }
    }
}

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

    fn check_callable_signature(&mut self, decl: &CallableDecl) {
        match convert_ty(self.resolutions, &decl.output).0 {
            _ if callable_functors(decl).is_empty() => {}
            Ty::Tuple(items) if items.is_empty() => {}
            output => self.errors.push(Error(ErrorKind::TypeMismatch(
                Ty::UNIT,
                output,
                decl.output.span,
            ))),
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
        self.check_callable_signature(decl);
        let output = convert_ty(self.resolutions, &decl.output).0;
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

fn convert_ty(resolutions: &Resolutions, ty: &ast::Ty) -> (Ty, Vec<MissingTyError>) {
    match &ty.kind {
        TyKind::Array(item) => {
            let (item, errors) = convert_ty(resolutions, item);
            (Ty::Array(Box::new(item)), errors)
        }
        TyKind::Arrow(kind, input, output, functors) => {
            let (input, mut errors) = convert_ty(resolutions, input);
            let (output, output_errors) = convert_ty(resolutions, output);
            errors.extend(output_errors);
            let functors = functors
                .as_ref()
                .map_or(HashSet::new(), FunctorExpr::to_set);
            (
                Ty::Arrow(*kind, Box::new(input), Box::new(output), functors),
                errors,
            )
        }
        TyKind::Hole => (Ty::Err, vec![MissingTyError(ty.span)]),
        TyKind::Paren(inner) => convert_ty(resolutions, inner),
        TyKind::Path(path) => (
            resolutions
                .get(&path.id)
                .copied()
                .map_or(Ty::Err, Ty::DefId),
            Vec::new(),
        ),
        &TyKind::Prim(prim) => (Ty::Prim(prim), Vec::new()),
        TyKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = convert_ty(resolutions, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
        TyKind::Var(name) => (Ty::Param(name.name.clone()), Vec::new()),
    }
}

fn pat_ty(resolutions: &Resolutions, pat: &Pat) -> (Ty, Vec<MissingTyError>) {
    match &pat.kind {
        PatKind::Bind(_, None) | PatKind::Discard(None) | PatKind::Elided => {
            (Ty::Err, vec![MissingTyError(pat.span)])
        }
        PatKind::Bind(_, Some(ty)) | PatKind::Discard(Some(ty)) => convert_ty(resolutions, ty),
        PatKind::Paren(inner) => pat_ty(resolutions, inner),
        PatKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = pat_ty(resolutions, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
    }
}

fn callable_ty(resolutions: &Resolutions, decl: &CallableDecl) -> (Ty, Vec<MissingTyError>) {
    let (input, mut errors) = pat_ty(resolutions, &decl.input);
    let (output, output_errors) = convert_ty(resolutions, &decl.output);
    errors.extend(output_errors);
    let functors = callable_functors(decl);
    (
        Ty::Arrow(decl.kind, Box::new(input), Box::new(output), functors),
        errors,
    )
}

fn callable_functors(decl: &CallableDecl) -> HashSet<Functor> {
    let mut functors = decl
        .functors
        .as_ref()
        .map_or(HashSet::new(), FunctorExpr::to_set);

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
