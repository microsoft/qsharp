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
    ast::{self, ItemKind, NodeId},
    mut_visit::MutVisitor,
    visit::Visitor as AstVisitor,
};
use qsc_hir::{
    hir::{self, PackageId},
    visit::Visitor as HirVisitor,
};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct Error(ErrorKind);

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
enum ErrorKind {
    Parse(parse::Error),
    Resolve(resolve::Error),
    Type(typeck::Error),
}

pub enum Fragment {
    Stmt(hir::Stmt),
    Callable(hir::CallableDecl),
    Error(Vec<Error>),
}

pub struct Compiler<'a> {
    assigner: Assigner,
    resolver: Resolver<'a>,
    checker: Checker,
    scope: HashMap<&'a str, NodeId>,
    lowerer: Lowerer,
}

impl<'a> Compiler<'a> {
    pub fn new(store: &'a PackageStore, dependencies: impl IntoIterator<Item = PackageId>) -> Self {
        let mut resolve_globals = resolve::GlobalTable::new();
        let mut typeck_globals = typeck::GlobalTable::new();
        for dependency in dependencies {
            let unit = store
                .get(dependency)
                .expect("dependency should be added to package store before compilation");
            resolve_globals.set_package(dependency);
            HirVisitor::visit_package(&mut resolve_globals, &unit.package);
            typeck_globals.set_package(dependency);
            HirVisitor::visit_package(&mut typeck_globals, &unit.package);
        }

        Self {
            assigner: Assigner::new(),
            resolver: resolve_globals.into_resolver(),
            checker: typeck_globals.into_checker(),
            scope: HashMap::new(),
            lowerer: Lowerer::new(),
        }
    }

    /// Compile a single string as either a callable declaration or a statement into a `Fragment`.
    /// # Errors
    /// This will Err if the fragment cannot be compiled due to parsing or symbol resolution errors.
    pub fn compile_fragment(&mut self, source: impl AsRef<str>) -> Vec<Fragment> {
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

    fn compile_callable_decl(&mut self, decl: ast::CallableDecl) -> Fragment {
        let decl = Box::leak(Box::new(decl));
        self.assigner.visit_callable_decl(decl);
        self.resolver.with_scope(&mut self.scope, |resolver| {
            resolver.add_global_callable(decl);
            AstVisitor::visit_callable_decl(resolver, decl);
        });
        self.checker.add_global_callable(decl);
        self.checker
            .check_callable_decl(self.resolver.resolutions(), decl);

        let errors = self.drain_errors();
        if errors.is_empty() {
            Fragment::Callable(
                self.lowerer
                    .with(self.resolver.resolutions(), self.checker.tys())
                    .lower_callable_decl(decl),
            )
        } else {
            Fragment::Error(errors)
        }
    }

    fn compile_stmt(&mut self, stmt: ast::Stmt) -> Fragment {
        let stmt = Box::leak(Box::new(stmt));
        self.assigner.visit_stmt(stmt);
        self.resolver.with_scope(&mut self.scope, |resolver| {
            resolver.visit_stmt(stmt);
        });
        self.checker.check_stmt(self.resolver.resolutions(), stmt);

        let errors = self.drain_errors();
        if errors.is_empty() {
            Fragment::Stmt(
                self.lowerer
                    .with(self.resolver.resolutions(), self.checker.tys())
                    .lower_stmt(stmt),
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
