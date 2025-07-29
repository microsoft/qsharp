// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::ast::{Ident, ImportKind, ImportOrExportDecl, Path, PathKind};
use qsc_data_structures::span::Span;

/// Iterates over the items in an import or export declaration, yielding only those
/// that have a well-formed path.
pub(crate) fn iter_valid_items(
    decl: &ImportOrExportDecl,
) -> impl Iterator<Item = ValidImportOrExportItem<'_>> + '_ {
    decl.items.iter().filter_map(move |item| {
        if let PathKind::Ok(path) = &item.path {
            Some(ValidImportOrExportItem {
                span: item.span,
                path,
                kind: &item.kind,
                is_export: decl.is_export(),
            })
        } else {
            None
        }
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValidImportOrExportItem<'a> {
    pub span: Span,
    pub path: &'a Path,
    pub kind: &'a ImportKind,
    pub is_export: bool,
}

impl ValidImportOrExportItem<'_> {
    pub fn name(&self) -> &Ident {
        let alias = match &self.kind {
            ImportKind::Wildcard => None,
            ImportKind::Direct { alias, .. } => alias.as_ref(),
        };

        match alias {
            Some(alias) => alias,
            None => &self.path.name,
        }
    }
}
