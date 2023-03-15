// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use qsc_ast::{
    ast::{CallableDecl, Item, ItemKind, NodeId},
    visit::Visitor,
};
use qsc_frontend::compile::{PackageId, PackageStore};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GlobalId {
    pub package: PackageId,
    pub node: NodeId,
}

struct CallableVisitor<'a, 'b> {
    callables: &'a mut HashMap<GlobalId, &'b CallableDecl>,
    package_id: PackageId,
}

impl<'a, 'b> CallableVisitor<'a, 'b> {
    fn new(package_id: PackageId, callables: &'a mut HashMap<GlobalId, &'b CallableDecl>) -> Self {
        Self {
            callables,
            package_id,
        }
    }
}

impl<'a, 'b> Visitor<'b> for CallableVisitor<'a, 'b> {
    fn visit_item(&mut self, item: &'b Item) {
        if let ItemKind::Callable(decl) = &item.kind {
            self.callables.insert(
                GlobalId {
                    package: self.package_id,
                    node: decl.name.id,
                },
                decl,
            );
        }
    }
}

pub(super) fn extract_callables(store: &PackageStore) -> HashMap<GlobalId, &CallableDecl> {
    let mut callables = HashMap::default();
    for (package_id, unit) in store.iter() {
        let mut visitor = CallableVisitor::new(*package_id, &mut callables);
        visitor.visit_package(&unit.package);
    }

    callables
}
