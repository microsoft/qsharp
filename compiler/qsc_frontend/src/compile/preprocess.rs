// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::str::FromStr;
use qsc_ast::{
    ast::{Attr, ExprKind, ItemKind, Namespace},
    mut_visit::MutVisitor,
};
use std::rc::Rc;

use super::TargetProfile;

pub(super) struct Conditional {
    target: TargetProfile,
    names: Vec<Rc<str>>,
}

impl Conditional {
    pub(super) fn new(target: TargetProfile) -> Self {
        Self {
            target,
            names: Vec::new(),
        }
    }

    pub(super) fn into_names(self) -> Vec<Rc<str>> {
        self.names
    }
}

impl MutVisitor for Conditional {
    fn visit_namespace(&mut self, namespace: &mut Namespace) {
        namespace.items = namespace
            .items
            .iter()
            .filter_map(|item| {
                if matches_target(&item.attrs, self.target) {
                    match item.kind.as_ref() {
                        ItemKind::Callable(callable) => {
                            self.names.retain(|n| n != &callable.name.name);
                        }
                        ItemKind::Ty(ident, _) => self.names.retain(|n| n != &ident.name),
                        _ => {}
                    }
                    Some(item.clone())
                } else {
                    match item.kind.as_ref() {
                        ItemKind::Callable(callable) => self.names.push(callable.name.name.clone()),
                        ItemKind::Ty(ident, _) => self.names.push(ident.name.clone()),
                        _ => {}
                    }
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
    }
}

fn matches_target(attrs: &[Box<Attr>], target: TargetProfile) -> bool {
    attrs.iter().all(|attr| {
        if attr.name.name.as_ref() == "Config" {
            if let ExprKind::Paren(inner) = attr.arg.kind.as_ref() {
                match inner.kind.as_ref() {
                    ExprKind::Path(path) => {
                        match TargetProfile::from_str(path.name.name.as_ref()) {
                            Ok(t) => t == target,
                            Err(_) => true,
                        }
                    }
                    _ => true,
                }
            } else {
                true
            }
        } else {
            true
        }
    })
}
