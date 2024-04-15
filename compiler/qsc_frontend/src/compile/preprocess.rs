// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::str::FromStr;
use qsc_ast::{
    ast::{Attr, ExprKind, ItemKind, Namespace, Stmt, StmtKind},
    mut_visit::MutVisitor,
};
use qsc_hir::hir;
use std::rc::Rc;

use super::RuntimeCapabilityFlags;

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
    let attrs: Vec<_> = attrs
        .iter()
        .filter(|attr| hir::Attr::from_str(attr.name.name.as_ref()) == Ok(hir::Attr::Config))
        .collect();

    if attrs.is_empty() {
        return true;
    }
    let mut found_capabilities = RuntimeCapabilityFlags::empty();

    for attr in attrs {
        if let ExprKind::Paren(inner) = attr.arg.kind.as_ref() {
            match inner.kind.as_ref() {
                ExprKind::Path(path) => {
                    if let Ok(capability) =
                        RuntimeCapabilityFlags::from_str(path.name.name.as_ref())
                    {
                        found_capabilities |= capability;
                    } else {
                        return true; // Unknown capability, so we assume it matches
                    }
                }
                _ => return true, // Unknown config attribute, so we assume it matches
            }
        } else {
            // Something other than a parenthesized expression, so we assume it matches
            return true;
        }
    }
    if found_capabilities == RuntimeCapabilityFlags::empty() {
        // There was at least one config attribute, but it was None
        // Therefore, we only match if there are no capabilities
        return capabilities == RuntimeCapabilityFlags::empty();
    }
    capabilities.contains(found_capabilities)
}

#[cfg(test)]
mod test {
    use qsc_ast::ast::{Attr, Expr, ExprKind, Ident, NodeId, Path};
    use qsc_data_structures::span::Span;

    use crate::compile::{preprocess::matches_config, RuntimeCapabilityFlags};

    fn named_attr(name: &str) -> Attr {
        Attr {
            name: Box::new(Ident {
                name: name.into(),
                span: Span::default(),
                id: NodeId::default(),
            }),
            arg: Box::new(Expr {
                id: NodeId::default(),
                span: Span::default(),
                kind: Box::new(ExprKind::Tuple(Box::new([]))),
            }),
            span: Span::default(),
            id: NodeId::default(),
        }
    }

    fn name_value_attr(name: &str, value: &str) -> Attr {
        Attr {
            name: Box::new(Ident {
                name: name.into(),
                span: Span::default(),
                id: NodeId::default(),
            }),
            arg: Box::new(Expr {
                id: NodeId::default(),
                span: Span::default(),
                kind: Box::new(ExprKind::Paren(Box::new(Expr {
                    id: NodeId::default(),
                    span: Span::default(),
                    kind: Box::new(ExprKind::Path(Box::new(Path {
                        id: NodeId::default(),
                        span: Span::default(),
                        namespace: None,
                        name: Box::new(Ident {
                            name: value.into(),
                            span: Span::default(),
                            id: NodeId::default(),
                        }),
                    }))),
                }))),
            }),
            span: Span::default(),
            id: NodeId::default(),
        }
    }

    #[test]
    fn no_attrs_matches() {
        assert!(matches_config(&[], RuntimeCapabilityFlags::empty()));
    }

    #[test]
    fn unknown_attrs_matches() {
        assert!(matches_config(
            &[Box::new(named_attr("unknown"))],
            RuntimeCapabilityFlags::empty()
        ));
    }

    #[test]
    fn none_attrs_matches_empty() {
        assert!(matches_config(
            &[Box::new(name_value_attr("Config", "None"))],
            RuntimeCapabilityFlags::empty()
        ));
    }

    #[test]
    fn none_attrs_does_not_match_all() {
        assert!(!matches_config(
            &[Box::new(name_value_attr("Config", "None"))],
            RuntimeCapabilityFlags::all()
        ));
    }

    #[test]
    fn none_attrs_does_not_match_forwardbranching() {
        assert!(!matches_config(
            &[Box::new(name_value_attr("Config", "None"))],
            RuntimeCapabilityFlags::ForwardBranching
        ));
    }

    #[test]
    fn forwardbranching_attrs_does_not_match_empty() {
        assert!(!matches_config(
            &[Box::new(name_value_attr("Config", "ForwardBranching"))],
            RuntimeCapabilityFlags::empty()
        ));
    }

    #[test]
    fn integercomputations_attrs_does_not_match_empty() {
        assert!(!matches_config(
            &[Box::new(name_value_attr("Config", "IntegerComputations"))],
            RuntimeCapabilityFlags::empty()
        ));
    }

    #[test]
    fn floatingpointcomputations_attrs_does_not_match_empty() {
        assert!(!matches_config(
            &[Box::new(name_value_attr(
                "Config",
                "FloatingPointComputations"
            ))],
            RuntimeCapabilityFlags::empty()
        ));
    }

    #[test]
    fn unrestricted_attrs_does_not_match_empty() {
        assert!(!matches_config(
            &[Box::new(name_value_attr("Config", "Unrestricted"))],
            RuntimeCapabilityFlags::empty()
        ));
    }

    #[test]
    fn unrestricted_attrs_matches_all() {
        assert!(matches_config(
            &[Box::new(name_value_attr("Config", "Unrestricted"))],
            RuntimeCapabilityFlags::all()
        ));
    }
}
