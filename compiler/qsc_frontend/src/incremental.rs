// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::PackageStore,
    lower::Lowerer,
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

    pub fn compile_fragments(&mut self, source: &str) -> Vec<Fragment> {
        let (stmts, errors) = parse::many_stmt(source);
        if !errors.is_empty() {
            return vec![Fragment::Error(
                errors
                    .into_iter()
                    .map(|e| Error(ErrorKind::Parse(e)))
                    .collect(),
            )];
        }

        let mut fragments = Vec::new();
        for stmt in stmts {
            if let Some(fragment) = self.compile_stmt(stmt) {
                fragments.push(fragment);
            }
            for item in self.lowerer.drain_items() {
                fragments.push(Fragment::Item(item));
            }
        }

        fragments
    }

    fn compile_stmt(&mut self, mut stmt: ast::Stmt) -> Option<Fragment> {
        self.assigner.visit_stmt(&mut stmt);
        self.resolver.visit_stmt(&stmt);
        self.checker
            .check_stmt_fragment(self.resolver.resolutions(), &stmt);

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
            .collect()
    }
}
