// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::{PackageId, PackageStore},
    lower::Lowerer,
    parse,
    resolve::{self, GlobalTable, Link, Resolutions, Resolver},
};
use miette::Diagnostic;
use qsc_ast::{
    assigner::Assigner,
    ast::{ItemKind, NodeId},
    mut_visit::MutVisitor as AstMutVisitor,
    visit::Visitor as AstVisitor,
};
use qsc_data_structures::index_map::IndexMap;
use qsc_hir::{hir, visit::Visitor as HirVisitor};
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
}

pub struct Compiler<'a> {
    assigner: Assigner,
    resolver: Resolver<'a>,
    fragments_scope: HashMap<&'a str, NodeId>,
    lowerer: Lowerer,
    resolutions: Resolutions<hir::NodeId>,
}

impl<'a> Compiler<'a> {
    pub fn new(store: &'a PackageStore, dependencies: impl IntoIterator<Item = PackageId>) -> Self {
        let mut globals = GlobalTable::new();
        for dependency in dependencies {
            let unit = store
                .get(dependency)
                .expect("dependency should be added to package store before compilation");
            globals.set_package(dependency);
            HirVisitor::visit_package(&mut globals, &unit.package);
        }

        Self {
            assigner: Assigner::new(),
            resolver: globals.into_resolver(),
            fragments_scope: HashMap::new(),
            lowerer: Lowerer::new(),
            resolutions: IndexMap::new(),
        }
    }

    #[must_use]
    pub fn resolutions(&self) -> &Resolutions<hir::NodeId> {
        &self.resolutions
    }

    /// Compile a single string as either a callable declaration or a statement into a `Fragment`.
    /// # Errors
    /// This will Err if the fragment cannot be compiled due to parsing or symbol resolution errors.
    pub fn compile_fragment(&mut self, source: impl AsRef<str>) -> Vec<Fragment> {
        self.resolver.reset_errors();
        let (item, errors) = parse::item(source.as_ref());

        match item.kind {
            ItemKind::Callable(mut decl) if errors.is_empty() => {
                self.assigner.visit_callable_decl(&mut decl);
                let decl = Box::leak(Box::new(decl));
                let mut errors = vec![];
                self.resolver
                    .with_scope(&mut self.fragments_scope, |resolver| {
                        resolver.add_global_callable(decl);
                        AstVisitor::visit_callable_decl(resolver, decl);
                        errors.extend(
                            resolver
                                .errors()
                                .iter()
                                .map(|e| Error(ErrorKind::Resolve(e.clone()))),
                        );
                    });

                let decl = self.lowerer.lower_callable_decl(decl);
                self.lower_resolutions();
                if errors.is_empty() {
                    vec![Fragment::Callable(decl)]
                } else {
                    vec![Fragment::Error(errors)]
                }
            }
            _ => {
                let (stmts, errors) = parse::stmts(source.as_ref());
                if !errors.is_empty() {
                    let mut parse_errors = vec![];
                    parse_errors.extend(errors.iter().map(|e| Error(ErrorKind::Parse(*e))));
                    return vec![Fragment::Error(parse_errors)];
                }

                let mut fragments = vec![];
                for mut stmt in stmts {
                    self.assigner.visit_stmt(&mut stmt);
                    let stmt = Box::leak(Box::new(stmt));
                    let mut errors = vec![];
                    self.resolver
                        .with_scope(&mut self.fragments_scope, |resolver| {
                            resolver.visit_stmt(stmt);
                            errors.extend(
                                resolver
                                    .errors()
                                    .iter()
                                    .map(|e| Error(ErrorKind::Resolve(e.clone()))),
                            );
                        });
                    let stmt = self.lowerer.lower_stmt(stmt);
                    self.lower_resolutions();

                    if !errors.is_empty() {
                        // bail out on first error
                        fragments.push(Fragment::Error(errors));
                        return fragments;
                    }

                    fragments.push(Fragment::Stmt(stmt));
                }

                fragments
            }
        }
    }

    fn lower_resolutions(&mut self) {
        for (id, link) in self.resolver.drain() {
            if let Some(id) = self.lowerer.get_id(id) {
                let link = match link {
                    Link::Internal(node) => Link::Internal(
                        self.lowerer
                            .get_id(node)
                            .expect("lowered node should not resolve to deleted node"),
                    ),
                    Link::External(package, node) => Link::External(package, node),
                };
                self.resolutions.insert(id, link);
            }
        }
    }
}

pub enum Fragment {
    Stmt(hir::Stmt),
    Callable(hir::CallableDecl),
    Error(Vec<Error>),
}
