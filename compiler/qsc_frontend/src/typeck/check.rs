// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    infer::Inferrer,
    rules::{self, SpecImpl},
    Error, ErrorKind, Table,
};
use crate::{
    resolve::{Names, Res},
    typeck::convert::{self, MissingTyError},
};
use qsc_ast::{
    ast::{self, NodeId},
    visit::{self, Visitor},
};
use qsc_data_structures::index_map::IndexMap;
use qsc_hir::{
    hir::{self, ItemId, PackageId},
    ty::{FunctorSetValue, Scheme, Ty, Udt},
};
use std::{collections::HashMap, vec};

pub(crate) struct GlobalTable {
    udts: HashMap<ItemId, Udt>,
    terms: HashMap<ItemId, Scheme>,
    errors: Vec<Error>,
}

impl GlobalTable {
    pub(crate) fn new() -> Self {
        Self {
            udts: HashMap::new(),
            terms: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub(crate) fn add_external_package(&mut self, id: PackageId, package: &hir::Package) {
        for item in package.items.values() {
            let item_id = ItemId {
                package: Some(id),
                item: item.id,
            };

            match &item.kind {
                hir::ItemKind::Callable(decl) => self.terms.insert(item_id, decl.scheme()),
                hir::ItemKind::Namespace(..) => None,
                hir::ItemKind::Ty(_, udt) => {
                    self.udts.insert(item_id, udt.clone());
                    self.terms.insert(item_id, udt.cons_scheme(item_id))
                }
            };
        }
    }
}

pub(crate) struct Checker {
    globals: HashMap<ItemId, Scheme>,
    table: Table,
    inferrer: Inferrer,
    new: Vec<NodeId>,
    errors: Vec<Error>,
}

impl Checker {
    pub(crate) fn new(globals: GlobalTable) -> Self {
        Checker {
            globals: globals.terms,
            table: Table {
                udts: globals.udts,
                terms: IndexMap::new(),
                generics: IndexMap::new(),
            },
            inferrer: Inferrer::new(),
            new: Vec::new(),
            errors: globals.errors,
        }
    }

    pub(crate) fn table(&self) -> &Table {
        &self.table
    }

    pub(crate) fn into_table(self) -> (Table, Vec<Error>) {
        (self.table, self.errors)
    }

    pub(crate) fn drain_errors(&mut self) -> vec::Drain<Error> {
        self.errors.drain(..)
    }

    pub(crate) fn check_package(&mut self, names: &Names, package: &ast::Package) {
        ItemCollector::new(self, names).visit_package(package);
        ItemChecker::new(self, names).visit_package(package);

        if let Some(entry) = &package.entry {
            self.errors.append(&mut rules::expr(
                names,
                &self.globals,
                &mut self.table,
                entry,
            ));
        }
    }

    pub(crate) fn check_namespace(&mut self, names: &Names, namespace: &ast::Namespace) {
        ItemCollector::new(self, names).visit_namespace(namespace);
        ItemChecker::new(self, names).visit_namespace(namespace);
    }

    fn check_callable_decl(&mut self, names: &Names, decl: &ast::CallableDecl) {
        self.check_callable_signature(names, decl);
        let output = convert::ty_from_ast(names, &decl.output).0;
        match &*decl.body {
            ast::CallableBody::Block(block) => self.check_spec(
                names,
                SpecImpl {
                    spec: ast::Spec::Body,
                    callable_input: &decl.input,
                    spec_input: None,
                    output: &output,
                    block,
                },
            ),
            ast::CallableBody::Specs(specs) => {
                for spec in specs.iter() {
                    if let ast::SpecBody::Impl(input, block) = &spec.body {
                        self.check_spec(
                            names,
                            SpecImpl {
                                spec: spec.spec,
                                callable_input: &decl.input,
                                spec_input: Some(input),
                                output: &output,
                                block,
                            },
                        );
                    }
                }
            }
        }
    }

    fn check_callable_signature(&mut self, names: &Names, decl: &ast::CallableDecl) {
        if convert::ast_callable_functors(decl) != FunctorSetValue::Empty {
            let output = convert::ty_from_ast(names, &decl.output).0;
            match &output {
                Ty::Tuple(items) if items.is_empty() => {}
                _ => self.errors.push(Error(ErrorKind::TyMismatch(
                    Ty::UNIT,
                    output,
                    decl.output.span,
                ))),
            }
        }
    }

    fn check_spec(&mut self, names: &Names, spec: SpecImpl) {
        self.errors.append(&mut rules::spec(
            names,
            &self.globals,
            &mut self.table,
            spec,
        ));
    }

    pub(crate) fn check_stmt_fragment(&mut self, names: &Names, stmt: &ast::Stmt) {
        ItemCollector::new(self, names).visit_stmt(stmt);
        ItemChecker::new(self, names).visit_stmt(stmt);

        self.new.append(&mut rules::stmt_fragment(
            names,
            &self.globals,
            &mut self.table,
            &mut self.inferrer,
            stmt,
        ));
    }

    pub(crate) fn solve(&mut self, names: &Names) {
        self.errors.append(&mut rules::solve(
            names,
            &self.globals,
            &mut self.table,
            &mut self.inferrer,
            std::mem::take(&mut self.new),
        ));
    }
}

struct ItemCollector<'a> {
    checker: &'a mut Checker,
    names: &'a Names,
}

impl<'a> ItemCollector<'a> {
    fn new(checker: &'a mut Checker, names: &'a Names) -> Self {
        Self { checker, names }
    }
}

impl Visitor<'_> for ItemCollector<'_> {
    fn visit_item(&mut self, item: &ast::Item) {
        match &*item.kind {
            ast::ItemKind::Callable(decl) => {
                let Some(&Res::Item(item)) = self.names.get(decl.name.id) else {
                    panic!("callable should have item ID");
                };

                let (scheme, errors) = convert::ast_callable_scheme(self.names, decl);
                for MissingTyError(span) in errors {
                    self.checker
                        .errors
                        .push(Error(ErrorKind::MissingItemTy(span)));
                }

                self.checker.globals.insert(item, scheme);
            }
            ast::ItemKind::Ty(name, def) => {
                let span = item.span;
                let Some(&Res::Item(item)) = self.names.get(name.id) else {
                    panic!("type should have item ID");
                };

                let (cons, cons_errors) = convert::ast_ty_def_cons(self.names, item, def);
                let (udt_def, def_errors) = convert::ast_ty_def(self.names, def);
                self.checker.errors.extend(
                    cons_errors
                        .into_iter()
                        .chain(def_errors)
                        .map(|MissingTyError(span)| Error(ErrorKind::MissingItemTy(span))),
                );

                self.checker.table.udts.insert(
                    item,
                    Udt {
                        name: name.name.clone(),
                        span,
                        definition: udt_def,
                    },
                );
                self.checker.globals.insert(item, cons);
            }
            _ => {}
        }

        visit::walk_item(self, item);
    }
}

struct ItemChecker<'a> {
    checker: &'a mut Checker,
    names: &'a Names,
}

impl<'a> ItemChecker<'a> {
    fn new(checker: &'a mut Checker, names: &'a Names) -> Self {
        Self { checker, names }
    }
}

impl Visitor<'_> for ItemChecker<'_> {
    fn visit_callable_decl(&mut self, decl: &ast::CallableDecl) {
        self.checker.check_callable_decl(self.names, decl);
        visit::walk_callable_decl(self, decl);
    }
}
