// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::{PackageId, PackageStore},
    id::Assigner,
    parse,
    resolve::{DefId, GlobalTable, Resolutions, Resolver},
};
use qsc_ast::{
    ast::{CallableDecl, ItemKind, Stmt},
    mut_visit::MutVisitor,
    visit::Visitor,
};
use std::collections::HashMap;

pub struct Compiler<'a> {
    assigner: Assigner,
    resolver: Resolver<'a>,
    fragments_scope: HashMap<&'a str, DefId>,
}

impl<'a> Compiler<'a> {
    pub fn new(store: &'a PackageStore, dependencies: impl IntoIterator<Item = PackageId>) -> Self {
        let mut globals = GlobalTable::new();
        for dependency in dependencies {
            globals.set_package(dependency);
            let unit = store
                .get(dependency)
                .expect("dependency should be added to package store before compilation");
            globals.visit_package(&unit.package);
        }

        Self {
            assigner: Assigner::new(),
            resolver: globals.into_resolver(),
            fragments_scope: HashMap::new(),
        }
    }

    #[must_use]
    pub fn resolutions(&self) -> &Resolutions {
        self.resolver.resolutions()
    }

    pub fn compile_fragment(&mut self, source: &str) -> Fragment<'static> {
        let (item, errors) = parse::item(source);
        match item.kind {
            ItemKind::Callable(mut decl) if errors.is_empty() => {
                self.assigner.visit_callable_decl(&mut decl);
                let decl = Box::leak(Box::new(decl));
                self.resolver
                    .with_scope(&mut self.fragments_scope, |resolver| {
                        resolver.add_global_callable(decl);
                        resolver.visit_callable_decl(decl);
                        assert!(resolver.errors().is_empty(), "resolution failed");
                    });
                Fragment::Callable(decl)
            }
            _ => {
                let (mut stmt, errors) = parse::stmt(source);
                assert!(errors.is_empty(), "parsing failed");
                self.assigner.visit_stmt(&mut stmt);
                let stmt = Box::leak(Box::new(stmt));
                self.resolver
                    .with_scope(&mut self.fragments_scope, |resolver| {
                        resolver.visit_stmt(stmt);
                        assert!(resolver.errors().is_empty(), "resolution failed");
                    });
                Fragment::Stmt(stmt)
            }
        }
    }
}

pub enum Fragment<'a> {
    Stmt(&'a Stmt),
    Callable(&'a CallableDecl),
}
