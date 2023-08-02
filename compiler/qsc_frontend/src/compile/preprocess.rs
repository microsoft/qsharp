// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::{
    ast::{Attr, Namespace},
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
                if item.attrs.is_empty() || matches_target(&item.attrs, self.target) {
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
    if attrs
        .iter()
        .any(|attr| attr.name.name.as_ref() == target.to_str())
    {
        true
    } else {
        !attrs
            .iter()
            .any(|attr| attr.name.name.starts_with("Target") && attr.name.name.ends_with("Profile"))
    }
}
