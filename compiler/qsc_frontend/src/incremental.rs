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
use qsc_ast::{
    assigner::Assigner,
    ast::{self, ItemKind},
    mut_visit::MutVisitor,
    visit::Visitor,
};
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
    Callable(hir::CallableDecl),
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

        let mut resolver = resolve_globals.into_resolver();
        resolver.add_scope();

        Self {
            assigner: Assigner::new(),
            resolver,
            checker: typeck_globals.into_checker(),
            lowerer: Lowerer::new(),
        }
    }

    pub fn assigner_mut(&mut self) -> &mut qsc_hir::assigner::Assigner {
        self.lowerer.assigner_mut()
    }

    pub fn compile_fragments(&mut self, source: impl AsRef<str>) -> Vec<Fragment> {
        let (item, errors) = parse::item(source.as_ref());
        match item.kind {
            ItemKind::Callable(decl) if errors.is_empty() => {
                return vec![self.compile_callable_decl(decl)];
            }
            _ => {}
        }

        let (stmts, errors) = parse::stmts(source.as_ref());
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
            fragments.push(self.compile_stmt(stmt));
            if matches!(fragments.last(), Some(Fragment::Error(_))) {
                break;
            }
        }
        fragments
    }

    fn compile_callable_decl(&mut self, mut decl: ast::CallableDecl) -> Fragment {
        self.assigner.visit_callable_decl(&mut decl);
        self.resolver.add_global_callable(&decl);
        self.resolver.visit_callable_decl(&decl);
        self.checker
            .add_global_callable(self.resolver.resolutions(), &decl);
        self.checker
            .check_callable_decl(self.resolver.resolutions(), &decl);

        let errors = self.drain_errors();
        if errors.is_empty() {
            Fragment::Callable(
                self.lowerer
                    .with(self.resolver.resolutions(), self.checker.tys())
                    .lower_callable_decl(&decl),
            )
        } else {
            Fragment::Error(errors)
        }
    }

    fn compile_stmt(&mut self, mut stmt: ast::Stmt) -> Fragment {
        self.assigner.visit_stmt(&mut stmt);
        self.resolver.visit_stmt(&stmt);
        self.checker.check_stmt(self.resolver.resolutions(), &stmt);

        let errors = self.drain_errors();
        if errors.is_empty() {
            Fragment::Stmt(
                self.lowerer
                    .with(self.resolver.resolutions(), self.checker.tys())
                    .lower_stmt(&stmt),
            )
        } else {
            Fragment::Error(errors)
        }
    }

    fn drain_errors(&mut self) -> Vec<Error> {
        let mut errors = Vec::new();
        errors.extend(
            self.resolver
                .drain_errors()
                .map(|e| Error(ErrorKind::Resolve(e))),
        );
        errors.extend(
            self.checker
                .drain_errors()
                .map(|e| Error(ErrorKind::Type(e))),
        );
        errors
    }
}
