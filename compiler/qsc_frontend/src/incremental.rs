// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::PackageStore,
    lower::{self, Lowerer},
    parse,
    resolve::{self, Resolver},
    typeck::{self, Checker},
};
use miette::Diagnostic;
use qsc_ast::{assigner::Assigner, ast, mut_visit::MutVisitor, visit::Visitor};
use qsc_hir::hir::{self, PackageId};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct Error(ErrorKind);

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
enum ErrorKind {
    #[error("syntax error")]
    Parse(#[from] parse::Error),
    #[error("name error")]
    Resolve(#[from] resolve::Error),
    #[error("type error")]
    Type(#[from] typeck::Error),
    #[error(transparent)]
    Lower(#[from] lower::Error),
}

pub enum Fragment {
    Stmt(hir::Stmt),
    Item(hir::Item),
    Error(Vec<Error>),
}

pub struct Compiler {
    assigner: Assigner,
    resolver: Resolver,
    checker: Checker,
    lowerer: Lowerer,
}

impl Compiler {
    pub fn new(store: &PackageStore, dependencies: impl IntoIterator<Item = PackageId>) -> Self {
        let mut resolve_globals = resolve::GlobalTable::new();
        let mut typeck_globals = typeck::GlobalTable::new();
        for id in dependencies {
            let unit = store
                .get(id)
                .expect("dependency should be added to package store before compilation");
            resolve_globals.add_external_package(id, &unit.package);
            typeck_globals.add_external_package(id, &unit.package);
        }

        Self {
            assigner: Assigner::new(),
            resolver: Resolver::with_persistent_local_scope(resolve_globals),
            checker: Checker::new(typeck_globals),
            lowerer: Lowerer::new(),
        }
    }

    pub fn assigner_mut(&mut self) -> &mut qsc_hir::assigner::Assigner {
        self.lowerer.assigner_mut()
    }

    pub fn compile_fragments(&mut self, input: &str) -> Vec<Fragment> {
        let (fragments, errors) = parse::fragments(input);
        if !errors.is_empty() {
            return vec![Fragment::Error(
                errors
                    .into_iter()
                    .map(|e| Error(ErrorKind::Parse(e)))
                    .collect(),
            )];
        }

        fragments
            .into_iter()
            .flat_map(|f| self.compile_fragment(f))
            .collect()
    }

    fn compile_fragment(&mut self, fragment: parse::Fragment) -> Vec<Fragment> {
        let fragment = match fragment {
            parse::Fragment::Namespace(namespace) => {
                self.compile_namespace(namespace).err().map(Fragment::Error)
            }
            parse::Fragment::Stmt(stmt) => self.compile_stmt(stmt),
        };

        fragment
            .into_iter()
            .chain(self.lowerer.drain_items().map(Fragment::Item))
            .collect()
    }

    fn compile_namespace(&mut self, mut namespace: ast::Namespace) -> Result<(), Vec<Error>> {
        self.assigner.visit_namespace(&mut namespace);
        self.resolver.visit_namespace(&namespace);
        self.checker
            .check_namespace(self.resolver.resolutions().1, &namespace);

        let errors = self.drain_errors();
        if errors.is_empty() {
            self.lowerer
                .with(self.resolver.resolutions(), self.checker.tys())
                .lower_namespace(&namespace);
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn compile_stmt(&mut self, mut stmt: ast::Stmt) -> Option<Fragment> {
        self.assigner.visit_stmt(&mut stmt);
        self.resolver.visit_stmt(&stmt);
        self.checker
            .check_stmt_fragment(self.resolver.resolutions().1, &stmt);

        let errors = self.drain_errors();
        if errors.is_empty() {
            self.lowerer
                .with(self.resolver.resolutions(), self.checker.tys())
                .lower_stmt(&stmt)
                .map(Fragment::Stmt)
        } else {
            Some(Fragment::Error(errors))
        }
    }

    fn drain_errors(&mut self) -> Vec<Error> {
        self.resolver
            .drain_errors()
            .map(|e| Error(e.into()))
            .chain(self.checker.drain_errors().map(|e| Error(e.into())))
            .chain(self.lowerer.drain_errors().map(|e| Error(e.into())))
            .collect()
    }
}
