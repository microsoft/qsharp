// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    rules::{self, SpecImpl},
    ty::{self, Ty},
    Error, ErrorKind, Tys,
};
use crate::{
    resolve::{Res, Resolutions},
    typeck::ty::MissingTyError,
};
use qsc_ast::{ast, visit::Visitor};
use qsc_hir::hir::{self, ItemId, PackageId};
use std::collections::HashMap;

pub(crate) struct GlobalTable<'a> {
    resolutions: &'a Resolutions,
    globals: HashMap<ItemId, Ty>,
    errors: Vec<Error>,
}

impl<'a> GlobalTable<'a> {
    pub(crate) fn new(resolutions: &'a Resolutions) -> Self {
        Self {
            resolutions,
            globals: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub(crate) fn add_local_package(&mut self, package: &ast::Package) {
        for namespace in &package.namespaces {
            for item in &namespace.items {
                if let ast::ItemKind::Callable(decl) = &item.kind {
                    let (ty, errors) = Ty::of_ast_callable(decl);
                    let Some(&Res::Item(item)) = self.resolutions.get(decl.name.id) else {
                        panic!("callable should have item ID")
                    };
                    self.globals.insert(item, ty);
                    for MissingTyError(span) in errors {
                        self.errors.push(Error(ErrorKind::MissingItemTy(span)));
                    }
                }
            }
        }
    }

    pub(crate) fn add_external_package(&mut self, id: PackageId, package: &hir::Package) {
        for item in package.items.values() {
            if let hir::ItemKind::Callable(decl) = &item.kind {
                let item_id = ItemId {
                    package: Some(id),
                    item: item.id,
                };
                let (ty, errors) = Ty::of_hir_callable(decl);
                self.globals.insert(item_id, ty);
                for MissingTyError(span) in errors {
                    self.errors.push(Error(ErrorKind::MissingItemTy(span)));
                }
            }
        }
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

pub(crate) struct Checker<'a> {
    resolutions: &'a Resolutions,
    globals: HashMap<ItemId, Ty>,
    tys: Tys<ast::NodeId>,
    errors: Vec<Error>,
}

impl Checker<'_> {
    pub(crate) fn into_tys(self) -> (Tys<ast::NodeId>, Vec<Error>) {
        (self.tys, self.errors)
    }

    fn check_callable_signature(&mut self, decl: &ast::CallableDecl) {
        if !ty::ast_callable_functors(decl).is_empty() {
            match &decl.output.kind {
                ast::TyKind::Tuple(items) if items.is_empty() => {}
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

    fn check_entry_expr(&mut self, entry: &ast::Expr) {
        let errors = rules::entry_expr(self.resolutions, &self.globals, &mut self.tys, entry);
        self.errors.extend(errors);
    }
}

impl Visitor<'_> for Checker<'_> {
    fn visit_package(&mut self, package: &ast::Package) {
        for namespace in &package.namespaces {
            self.visit_namespace(namespace);
        }
        if let Some(entry) = &package.entry {
            self.check_entry_expr(entry);
        }
    }

    fn visit_callable_decl(&mut self, decl: &ast::CallableDecl) {
        self.tys.insert(decl.name.id, Ty::of_ast_callable(decl).0);
        self.check_callable_signature(decl);

        let output = Ty::from_ast(&decl.output).0;
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
}
