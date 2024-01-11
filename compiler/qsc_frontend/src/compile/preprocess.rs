// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::str::FromStr;
use qsc_ast::{
    ast::{Attr, ExprKind, ItemKind, Namespace, Stmt, StmtKind},
    mut_visit::MutVisitor,
};
use qsc_hir::hir;
use std::rc::Rc;

use super::{ConfigAttr, RuntimeCapabilityFlags};

#[derive(PartialEq, Hash, Clone, Debug)]
pub struct TrackedName {
    pub name: Rc<str>,
    pub namespace: Rc<str>,
}

pub(crate) struct Conditional {
    capabilities: RuntimeCapabilityFlags,
    dropped_names: Vec<TrackedName>,
    included_names: Vec<TrackedName>,
}

impl Conditional {
    pub(crate) fn new(capabilities: RuntimeCapabilityFlags) -> Self {
        Self {
            capabilities,
            dropped_names: Vec::new(),
            included_names: Vec::new(),
        }
    }

    pub(crate) fn into_names(self) -> Vec<TrackedName> {
        self.dropped_names
            .into_iter()
            .filter(|n| !self.included_names.contains(n))
            .collect()
    }
}

impl MutVisitor for Conditional {
    fn visit_namespace(&mut self, namespace: &mut Namespace) {
        namespace.items = namespace
            .items
            .iter()
            .filter_map(|item| {
                if matches_config(&item.attrs, self.capabilities) {
                    match item.kind.as_ref() {
                        ItemKind::Callable(callable) => {
                            self.included_names.push(TrackedName {
                                name: callable.name.name.clone(),
                                namespace: namespace.name.name.clone(),
                            });
                        }
                        ItemKind::Ty(ident, _) => self.included_names.push(TrackedName {
                            name: ident.name.clone(),
                            namespace: namespace.name.name.clone(),
                        }),
                        _ => {}
                    }
                    Some(item.clone())
                } else {
                    match item.kind.as_ref() {
                        ItemKind::Callable(callable) => {
                            self.dropped_names.push(TrackedName {
                                name: callable.name.name.clone(),
                                namespace: namespace.name.name.clone(),
                            });
                        }
                        ItemKind::Ty(ident, _) => self.dropped_names.push(TrackedName {
                            name: ident.name.clone(),
                            namespace: namespace.name.name.clone(),
                        }),
                        _ => {}
                    }
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        if let StmtKind::Item(item) = stmt.kind.as_mut() {
            if matches_config(&item.attrs, self.capabilities) {
                match item.kind.as_ref() {
                    ItemKind::Callable(callable) => {
                        self.included_names.push(TrackedName {
                            name: callable.name.name.clone(),
                            namespace: Rc::from(""),
                        });
                    }
                    ItemKind::Ty(ident, _) => self.included_names.push(TrackedName {
                        name: ident.name.clone(),
                        namespace: Rc::from(""),
                    }),
                    _ => {}
                }
            } else {
                match item.kind.as_ref() {
                    ItemKind::Callable(callable) => {
                        self.dropped_names.push(TrackedName {
                            name: callable.name.name.clone(),
                            namespace: Rc::from(""),
                        });
                    }
                    ItemKind::Ty(ident, _) => self.dropped_names.push(TrackedName {
                        name: ident.name.clone(),
                        namespace: Rc::from(""),
                    }),
                    _ => {}
                }
                stmt.kind = Box::new(StmtKind::Empty);
            }
        }
    }
}

fn matches_config(attrs: &[Box<Attr>], capabilities: RuntimeCapabilityFlags) -> bool {
    attrs.iter().all(|attr| {
        if hir::Attr::from_str(attr.name.name.as_ref()) == Ok(hir::Attr::Config) {
            if let ExprKind::Paren(inner) = attr.arg.kind.as_ref() {
                match inner.kind.as_ref() {
                    // If there is no config attribute, then we assume that the item matches
                    // the target. We can't do membership tests on the capabilities because
                    // Base is not a subset of any capabilities, it is a lack of capabilities.
                    ExprKind::Path(path) => match ConfigAttr::from_str(path.name.name.as_ref()) {
                        Ok(ConfigAttr::Unrestricted) => capabilities.is_all(),
                        Ok(ConfigAttr::Base) => capabilities.is_empty(),
                        _ => true,
                    },
                    _ => true, // Unknown config attribute, so we assume it matches
                }
            } else {
                // Something other than a parenthesized expression, so we assume it matches
                true
            }
        } else {
            // Unknown attribute, so we assume it matches
            true
        }
    })
}
