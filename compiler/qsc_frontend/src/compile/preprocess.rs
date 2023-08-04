// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::str::FromStr;
use qsc_ast::{
    ast::{Attr, ExprKind, Namespace},
    mut_visit::MutVisitor,
};

use super::Target;

pub(super) struct Conditional {
    pub(super) target: Target,
}

impl MutVisitor for Conditional {
    fn visit_namespace(&mut self, namespace: &mut Namespace) {
        namespace.items = namespace
            .items
            .iter()
            .filter_map(|item| {
                if matches_target(&item.attrs, self.target) {
                    Some(item.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
    }
}

fn matches_target(attrs: &[Box<Attr>], target: Target) -> bool {
    attrs.iter().all(|attr| {
        if attr.name.name.as_ref() == "TargetProfile" {
            if let ExprKind::Paren(inner) = attr.arg.kind.as_ref() {
                match inner.kind.as_ref() {
                    ExprKind::Path(path) => match Target::from_str(path.name.name.as_ref()) {
                        Ok(t) => t == target,
                        Err(_) => true,
                    },
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
