// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    rules::{self, SpecImpl},
    Error, ErrorKind, Tys,
};
use crate::{
    resolve::{Res, Resolutions},
    typeck::convert::{self, MissingTyError},
};
use qsc_ast::{ast, visit::Visitor as AstVisitor};
use qsc_hir::{
    hir::{self, PackageId, Ty},
    visit::Visitor as HirVisitor,
};
use std::{collections::HashMap, vec};

pub(crate) struct GlobalTable {
    globals: HashMap<Res, Ty>,
    package: Option<PackageId>,
    errors: Vec<Error>,
}

impl GlobalTable {
    pub(crate) fn new() -> Self {
        Self {
            globals: HashMap::new(),
            package: None,
            errors: Vec::new(),
        }
    }

    pub(crate) fn set_package(&mut self, package: PackageId) {
        self.package = Some(package);
    }

    pub(crate) fn into_checker(self) -> Checker {
        Checker {
            globals: self.globals,
            tys: Tys::new(),
            errors: self.errors,
        }
    }
}

impl AstVisitor<'_> for GlobalTable {
    fn visit_callable_decl(&mut self, decl: &ast::CallableDecl) {
        assert!(
            self.package.is_none(),
            "package ID should not be set before visiting AST"
        );

        let (ty, errors) = convert::ast_callable_ty(decl);
        self.globals.insert(Res::Internal(decl.name.id), ty);
        for MissingTyError(span) in errors {
            self.errors.push(Error(ErrorKind::MissingItemTy(span)));
        }
    }
}

impl HirVisitor<'_> for GlobalTable {
    fn visit_callable_decl(&mut self, decl: &hir::CallableDecl) {
        let package = self
            .package
            .expect("package ID should be set before visiting HIR");
        self.globals.insert(
            Res::External(package, decl.name.id),
            convert::hir_callable_ty(decl),
        );
    }
}

pub(crate) struct Checker {
    globals: HashMap<Res, Ty>,
    tys: Tys,
    errors: Vec<Error>,
}

impl Checker {
    pub(crate) fn tys(&self) -> &Tys {
        &self.tys
    }

    pub(crate) fn drain_errors(&mut self) -> vec::Drain<Error> {
        self.errors.drain(..)
    }

    pub(crate) fn add_global_callable(&mut self, decl: &ast::CallableDecl) {
        let (ty, errors) = convert::ast_callable_ty(decl);
        self.globals.insert(Res::Internal(decl.name.id), ty);
        for MissingTyError(span) in errors {
            self.errors.push(Error(ErrorKind::MissingItemTy(span)));
        }
    }

    pub(crate) fn into_tys(self) -> (Tys, Vec<Error>) {
        (self.tys, self.errors)
    }

    pub(crate) fn with<'a>(&'a mut self, resolutions: &'a Resolutions) -> With {
        With {
            checker: self,
            resolutions,
        }
    }
}

pub(crate) struct With<'a> {
    checker: &'a mut Checker,
    resolutions: &'a Resolutions,
}

impl With<'_> {
    fn check_callable_signature(&mut self, decl: &ast::CallableDecl) {
        if !convert::ast_callable_functors(decl).is_empty() {
            match &decl.output.kind {
                ast::TyKind::Tuple(items) if items.is_empty() => {}
                _ => self.checker.errors.push(Error(ErrorKind::TypeMismatch(
                    Ty::UNIT,
                    convert::ty_from_ast(&decl.output).0,
                    decl.output.span,
                ))),
            }
        }
    }

    fn check_spec(&mut self, spec: SpecImpl) {
        self.checker.errors.append(&mut rules::spec(
            self.resolutions,
            &self.checker.globals,
            &mut self.checker.tys,
            spec,
        ));
    }
}

impl AstVisitor<'_> for With<'_> {
    fn visit_package(&mut self, package: &ast::Package) {
        for namespace in &package.namespaces {
            self.visit_namespace(namespace);
        }

        if let Some(entry) = &package.entry {
            self.checker.errors.append(&mut rules::expr(
                self.resolutions,
                &self.checker.globals,
                &mut self.checker.tys,
                entry,
            ));
        }
    }

    fn visit_callable_decl(&mut self, decl: &ast::CallableDecl) {
        self.checker
            .tys
            .insert(decl.name.id, convert::ast_callable_ty(decl).0);
        self.check_callable_signature(decl);

        let output = convert::ty_from_ast(&decl.output).0;
        match &decl.body {
            ast::CallableBody::Block(block) => self.check_spec(SpecImpl {
                spec: ast::Spec::Body,
                callable_input: &decl.input,
                spec_input: None,
                output: &output,
                block,
            }),
            ast::CallableBody::Specs(specs) => {
                for spec in specs {
                    if let ast::SpecBody::Impl(input, block) = &spec.body {
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

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        // TODO: Probably more correct for the REPL to keep the Inferrer around.
        self.checker.errors.append(&mut rules::stmt(
            self.resolutions,
            &self.checker.globals,
            &mut self.checker.tys,
            stmt,
        ));
    }
}
