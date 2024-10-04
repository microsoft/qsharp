// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::str::FromStr;
use qsc_ast::{
    ast::{Attr, ExprKind, Idents, ItemKind, Namespace, PathKind, Stmt, StmtKind, UnOp},
    mut_visit::MutVisitor,
};
use qsc_hir::hir;
use std::rc::Rc;

use super::TargetCapabilityFlags;

#[cfg(test)]
mod tests;

#[derive(PartialEq, Hash, Clone, Debug)]
pub struct TrackedName {
    pub name: Rc<str>,
    pub namespace: Rc<str>,
}

pub(crate) struct Conditional {
    capabilities: TargetCapabilityFlags,
    dropped_names: Vec<TrackedName>,
    included_names: Vec<TrackedName>,
}

impl Conditional {
    pub(crate) fn new(capabilities: TargetCapabilityFlags) -> Self {
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
                                namespace: namespace.name.full_name(),
                            });
                        }
                        ItemKind::Ty(ident, _) => self.included_names.push(TrackedName {
                            name: ident.name.clone(),
                            namespace: namespace.name.full_name(),
                        }),
                        _ => {}
                    }
                    Some(item.clone())
                } else {
                    match item.kind.as_ref() {
                        ItemKind::Callable(callable) => {
                            self.dropped_names.push(TrackedName {
                                name: callable.name.name.clone(),
                                namespace: namespace.name.full_name(),
                            });
                        }
                        ItemKind::Ty(ident, _) => self.dropped_names.push(TrackedName {
                            name: ident.name.clone(),
                            namespace: namespace.name.full_name(),
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

fn matches_config(attrs: &[Box<Attr>], capabilities: TargetCapabilityFlags) -> bool {
    let attrs: Vec<_> = attrs
        .iter()
        .filter(|attr| hir::Attr::from_str(attr.name.name.as_ref()) == Ok(hir::Attr::Config))
        .collect();

    if attrs.is_empty() {
        return true;
    }
    let mut found_capabilities = TargetCapabilityFlags::empty();
    let mut disallowed_capabilities = TargetCapabilityFlags::empty();
    let mut base = false;
    let mut not_base = false;

    // When checking attributes, anything we don't recognize (invalid form or invalid capability) gets
    // left in the compilation by returning true. This ensures that later compilation steps, specifically lowering
    // from AST to HIR, can check the attributes and return errors as appropriate.
    for attr in attrs {
        if let ExprKind::Paren(inner) = attr.arg.kind.as_ref() {
            match inner.kind.as_ref() {
                ExprKind::Path(PathKind::Ok(path)) => {
                    if let Ok(capability) = TargetCapabilityFlags::from_str(path.name.name.as_ref())
                    {
                        if capability.is_empty() {
                            base = true;
                        }
                        found_capabilities |= capability;
                    } else {
                        return true; // Unknown capability, so we assume it matches
                    }
                }
                ExprKind::UnOp(UnOp::NotL, inner) => {
                    if let ExprKind::Path(PathKind::Ok(path)) = inner.kind.as_ref() {
                        if let Ok(capability) =
                            TargetCapabilityFlags::from_str(path.name.name.as_ref())
                        {
                            if capability.is_empty() {
                                not_base = true;
                            }
                            disallowed_capabilities |= capability;
                        } else {
                            return true; // Unknown capability, so we assume it matches
                        }
                    } else {
                        return true; // Unknown config attribute, so we assume it matches
                    }
                }
                _ => return true, // Unknown config attribute, so we assume it matches
            }
        } else {
            // Something other than a parenthesized expression, so we assume it matches
            return true;
        }
    }
    if found_capabilities.is_empty() && disallowed_capabilities.is_empty() {
        if not_base && !base {
            // There was at least one config attribute, but it was "not Base" so
            // ensure that the capabilities are not empty.
            return capabilities != TargetCapabilityFlags::empty();
        } else if base && !not_base {
            // There was at least one config attribute, but it was Base
            // Therefore, we only match if there are no capabilities
            return capabilities == TargetCapabilityFlags::empty();
        }

        // The config specified both "Base" and "not Base" which is a contradiction, but we
        // drop the item in this case.
        return false;
    }
    capabilities.contains(found_capabilities)
        && (disallowed_capabilities.is_empty() || !capabilities.contains(disallowed_capabilities))
}
