// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use qsc_ast::{
    ast::{CallableDecl, Item, ItemKind},
    visit::Visitor,
};
use qsc_frontend::{compile::PackageStore, resolve::DefId};

#[derive(Default)]
struct CallableVisitor<'a> {
    callables: Vec<&'a CallableDecl>,
}

impl<'a> Visitor<'a> for CallableVisitor<'a> {
    fn visit_item(&mut self, item: &'a Item) {
        if let ItemKind::Callable(decl) = &item.kind {
            self.callables.push(decl);
        }
    }
}

pub fn extract_callables(store: &PackageStore) -> HashMap<DefId, &CallableDecl> {
    let mut callables = HashMap::default();
    for (package_id, unit) in store.iter() {
        let mut visitor = CallableVisitor::default();
        visitor.visit_package(&unit.package);
        for callable in visitor.callables {
            callables.insert(
                DefId {
                    package: qsc_frontend::resolve::PackageSrc::Extern(*package_id),
                    node: callable.name.id,
                },
                callable,
            );
        }
    }

    callables
}
