// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_frontend::compile::PackageStore;
use qsc_hir::{
    hir::{CallableDecl, Item, ItemKind, NodeId, PackageId},
    visit::Visitor,
};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

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

#[must_use]
pub fn extract_callables(store: &PackageStore) -> HashMap<GlobalId, &CallableDecl> {
    let mut callables = HashMap::default();
    for (package_id, unit) in store.iter() {
        let mut visitor = CallableVisitor {
            callables: &mut callables,
            package_id: *package_id,
        };
        visitor.visit_package(&unit.package);
    }

    callables
}

struct CallableVisitor<'a, 'b> {
    callables: &'a mut HashMap<GlobalId, &'b CallableDecl>,
    package_id: PackageId,
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
