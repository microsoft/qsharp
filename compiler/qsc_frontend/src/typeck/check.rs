// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    rules::{self, SpecImpl},
    ty::{self, Ty},
    Error, ErrorKind, Tys,
};
use crate::{
    compile::PackageId,
    resolve::{Link, Resolutions},
    typeck::ty::MissingTyError,
};
use qsc_ast::{
    ast::{CallableBody, CallableDecl, Expr, NodeId, Package, Spec, SpecBody, TyKind},
    visit::Visitor,
};
use qsc_hir::{hir, visit as hir_visit};
use std::collections::HashMap;

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
        let (ty, errors) = Ty::of_ast_callable(decl);
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
        let (ty, errors) = Ty::of_hir_callable(decl);
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
        if !ty::ast_callable_functors(decl).is_empty() {
            match &decl.output.kind {
                TyKind::Tuple(items) if items.is_empty() => {}
                _ => self.errors.push(Error(ErrorKind::TypeMismatch(
                    Ty::UNIT,
                    Ty::from_ast(&decl.output).0,
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
        self.tys.insert(decl.name.id, Ty::of_ast_callable(decl).0);
        self.check_callable_signature(decl);

        let output = Ty::from_ast(&decl.output).0;
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
