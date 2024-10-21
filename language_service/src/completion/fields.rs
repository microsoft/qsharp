// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{path_context::PathOrFieldAccess, Completion};
use crate::{compilation::Compilation, protocol::CompletionItemKind};
use qsc::{
    display::Lookup,
    hir::{
        ty::{Ty, UdtDefKind},
        ItemKind, Res,
    },
};

/// If there is an incomplete field access expression (e.g. `foo.bar.`) at the cursor offset,
/// provides the possible field names.
pub(super) struct Fields<'a> {
    compilation: &'a Compilation,
    path_context: &'a PathOrFieldAccess<'a>,
}

impl<'a> Fields<'a> {
    pub(crate) fn new(compilation: &'a Compilation, path_context: &'a PathOrFieldAccess) -> Self {
        Self {
            compilation,
            path_context,
        }
    }

    pub(crate) fn fields(&self) -> Vec<Completion> {
        let Some(id) = self.path_context.field_access_context() else {
            return vec![];
        };

        let mut completions = vec![];
        let ty = self.compilation.get_ty(id);
        if let Some(Ty::Udt(_, Res::Item(item_id))) = ty {
            let (item, _, _) = self
                .compilation
                .resolve_item_relative_to_user_package(item_id);
            if let ItemKind::Ty(_, udt) = &item.kind {
                collect_fields(&mut completions, &udt.definition.kind);
            }
        }
        completions
    }
}

fn collect_fields(completions: &mut Vec<Completion>, field: &UdtDefKind) {
    match field {
        UdtDefKind::Field(field) => {
            if let Some(name) = &field.name {
                let detail = field.ty.display();
                completions.push(Completion::with_detail(
                    name.to_string(),
                    CompletionItemKind::Field,
                    Some(detail),
                ));
            }
        }
        UdtDefKind::Tuple(vec) => {
            for f in vec {
                collect_fields(completions, &f.kind);
            }
        }
    }
}
