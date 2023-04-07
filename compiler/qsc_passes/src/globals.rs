// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

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

impl Display for GlobalId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<node {} in package {}>", self.node, self.package)
    }
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

#[must_use]
pub fn extract_callables(store: &PackageStore) -> HashMap<GlobalId, &CallableDecl> {
    let mut callables = HashMap::default();
    for (package_id, unit) in store.iter() {
        let mut visitor = CallableVisitor::new(*package_id, &mut callables);
        visitor.visit_package(&unit.package);
    }

    callables
}
