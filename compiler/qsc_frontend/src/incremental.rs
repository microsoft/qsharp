// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::{Offsetter, PackageStore, SourceMap},
    lower::{self, Lowerer},
    resolve::{self, Resolver},
    typeck::{self, Checker},
};
use miette::Diagnostic;
use qsc_ast::{assigner::Assigner as AstAssigner, ast, mut_visit::MutVisitor, visit::Visitor};
use qsc_hir::{
    assigner::Assigner as HirAssigner,
    hir::{self, PackageId},
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct Error(ErrorKind);

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
enum ErrorKind {
    #[error("syntax error")]
    Parse(#[from] qsc_parse::Error),
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
}

pub struct Compiler {
    lines: usize,
    sources: SourceMap,
    ast_assigner: AstAssigner,
    hir_assigner: HirAssigner,
    resolver: Resolver,
    checker: Checker,
    lowerer: Lowerer,
}

impl Compiler {
    pub fn new(store: &PackageStore, dependencies: impl IntoIterator<Item = PackageId>) -> Self {
        let mut resolve_globals = resolve::GlobalTable::new();
        let mut typeck_globals = typeck::GlobalTable::new();
        if let Some(unit) = store.get(PackageId::CORE) {
            resolve_globals.add_external_package(PackageId::CORE, &unit.package);
            typeck_globals.add_external_package(PackageId::CORE, &unit.package);
        }

        for id in dependencies {
            let unit = store
                .get(id)
                .expect("dependency should be added to package store before compilation");
            resolve_globals.add_external_package(id, &unit.package);
            typeck_globals.add_external_package(id, &unit.package);
        }

        Self {
            lines: 0,
            sources: SourceMap::default(),
            ast_assigner: AstAssigner::new(),
            hir_assigner: HirAssigner::new(),
            resolver: Resolver::with_persistent_local_scope(resolve_globals),
            checker: Checker::new(typeck_globals),
            lowerer: Lowerer::new(),
        }
    }

    pub fn assigner_mut(&mut self) -> &mut qsc_hir::assigner::Assigner {
        &mut self.hir_assigner
    }

    #[must_use]
    pub fn source_map(&self) -> &SourceMap {
        &self.sources
    }

    /// Compile a string with one or more fragments of Q# code.
    /// # Errors
    /// Returns a vector of errors if any of the input fails compilation.
    pub fn compile_fragments(&mut self, input: &str) -> Result<Vec<Fragment>, Vec<Error>> {
        // Append the line to the source map with the appropriate offset
        let offset = self
            .sources
            .push(format!("line_{}", self.lines).into(), input.into());
        self.lines += 1;

        let (mut fragments, errors) = qsc_parse::fragments(input);
        if !errors.is_empty() {
            return Err(errors
                .into_iter()
                .map(|e| Error(ErrorKind::Parse(e.with_offset(offset))))
                .collect());
        }

        let mut offsetter = Offsetter(offset);

        for fragment in &mut fragments {
            match fragment {
                qsc_parse::Fragment::Namespace(namespace) => offsetter.visit_namespace(namespace),
                qsc_parse::Fragment::Stmt(stmt) => offsetter.visit_stmt(stmt),
            }
        }

        for fragment in &mut fragments {
            match fragment {
                qsc_parse::Fragment::Namespace(namespace) => self.check_namespace(namespace),
                qsc_parse::Fragment::Stmt(stmt) => self.check_stmt(stmt),
            }
        }
        self.checker.solve(self.resolver.names());

        let fragments = fragments
            .into_iter()
            .flat_map(|f| self.lower_fragment(f))
            .collect();

        let errors = self.drain_errors();
        if errors.is_empty() {
            Ok(fragments)
        } else {
            self.lowerer.clear_items();
            Err(errors)
        }
    }

    fn lower_fragment(&mut self, fragment: qsc_parse::Fragment) -> Vec<Fragment> {
        let fragment = match fragment {
            qsc_parse::Fragment::Namespace(namespace) => {
                self.lower_namespace(&namespace);
                None
            }
            qsc_parse::Fragment::Stmt(stmt) => self.lower_stmt(&stmt),
        };

        self.lowerer
            .drain_items()
            .map(Fragment::Item)
            .chain(fragment)
            .collect()
    }

    fn check_namespace(&mut self, namespace: &mut ast::Namespace) {
        self.ast_assigner.visit_namespace(namespace);
        self.resolver
            .with(&mut self.hir_assigner)
            .visit_namespace(namespace);
        self.checker
            .check_namespace(self.resolver.names(), namespace);
    }

    fn lower_namespace(&mut self, namespace: &ast::Namespace) {
        self.lowerer
            .with(
                &mut self.hir_assigner,
                self.resolver.names(),
                self.checker.table(),
            )
            .lower_namespace(namespace);
    }

    fn check_stmt(&mut self, stmt: &mut ast::Stmt) {
        self.ast_assigner.visit_stmt(stmt);
        self.resolver.with(&mut self.hir_assigner).visit_stmt(stmt);
        self.checker
            .check_stmt_fragment(self.resolver.names(), stmt);
    }

    fn lower_stmt(&mut self, stmt: &ast::Stmt) -> Option<Fragment> {
        self.lowerer
            .with(
                &mut self.hir_assigner,
                self.resolver.names(),
                self.checker.table(),
            )
            .lower_stmt(stmt)
            .map(Fragment::Stmt)
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
