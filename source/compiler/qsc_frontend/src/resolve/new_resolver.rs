// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::resolve::{Error, Resolver, ScopeKind};
use qsc_ast::{
    ast::{self, Ident, Idents, ImportOrExportItem, Item, ItemKind, Path, PathKind},
    visit::{self as ast_visit, Visitor as AstVisitor},
};
use qsc_data_structures::span::Span;

/// An individual item within an [`ImportOrExportDecl`]. This can be a path or a path with an alias.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ValidImportOrExportItem {
    /// The span of the import path including the glob and alias, if any.
    pub span: Span,
    /// The path to the item being exported.
    pub path: Box<Path>,
    pub kind: ImportOrExportItemKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ImportOrExportItemKind {
    /// A glob import
    GlobImport,
    /// A direct import/export
    Direct {
        /// An optional alias for the item being imported.
        alias: Option<Ident>,
        /// Export or import?
        is_export: bool,
    },
}

impl ValidImportOrExportItem {
    pub(super) fn valid_item(
        item: &ImportOrExportItem,
        is_export: bool,
        errors: &mut Vec<Error>,
    ) -> Option<Self> {
        let PathKind::Ok(path) = &item.path else {
            return None;
        };

        if item.is_glob {
            if is_export {
                errors.push(Error::GlobExportNotSupported(path.span));
                return None;
            }

            if let Some(alias) = &item.alias {
                errors.push(Error::GlobImportAliasNotSupported {
                    span: item.span,
                    namespace_name: path.full_name().to_string(),
                    alias: alias.name.to_string(),
                });
                return None;
            }
            Some(ValidImportOrExportItem {
                span: item.span,
                path: path.clone(),
                kind: ImportOrExportItemKind::GlobImport,
            })
        } else {
            Some(ValidImportOrExportItem {
                span: item.span,
                path: path.clone(),
                kind: ImportOrExportItemKind::Direct {
                    alias: item.alias.clone(),
                    is_export,
                },
            })
        }
    }

    pub fn name(&self) -> &Ident {
        let alias = match &self.kind {
            ImportOrExportItemKind::GlobImport => None,
            ImportOrExportItemKind::Direct { alias, .. } => alias.as_ref(),
        };

        match alias {
            Some(alias) => alias,
            None => &self.path.name,
        }
    }

    pub fn is_name_only_export(&self) -> bool {
        matches!(
            self.kind,
            ImportOrExportItemKind::Direct {
                alias: None,
                is_export: true,
            }
        )
    }

    pub fn _is_export(&self) -> bool {
        !matches!(
            self.kind,
            ImportOrExportItemKind::GlobImport
                | ImportOrExportItemKind::Direct {
                    is_export: false,
                    ..
                }
        )
    }
}

pub(super) struct ImportBinder<'a> {
    pub(super) resolver: &'a mut Resolver,
}

impl ImportBinder<'_> {
    /// Given a function, apply that function to `Self` within a scope. In other words, this
    /// function automatically pushes a scope before `f` and pops it after.
    fn with_scope(&mut self, span: Span, kind: ScopeKind, f: impl FnOnce(&mut Self)) {
        self.resolver.push_scope(span, kind);
        f(self);
        self.resolver.pop_scope();
    }
}

impl AstVisitor<'_> for ImportBinder<'_> {
    fn visit_item(&mut self, item: &Item) {
        item.attrs.iter().for_each(|a| self.visit_attr(a));
        match &*item.kind {
            ItemKind::ImportOrExport(decl) if decl.is_export() => self
                .resolver
                .errors
                .push(Error::ExportFromLocalScope(item.span)),
            ItemKind::Open(..) => (),
            _ => ast_visit::walk_item(self, item),
        }
    }

    fn visit_namespace(&mut self, namespace: &ast::Namespace) {
        let ns = self
            .resolver
            .globals
            .find_namespace(namespace.name.str_iter())
            .expect("namespace should exist by this point");
        let root_id = self.resolver.globals.namespaces.root_id();
        let kind = ScopeKind::Namespace(ns);
        self.with_scope(namespace.span, kind, |visitor| {
            // the below line ensures that this namespace opens itself, in case
            // we are re-declaring a namespace. This is important, as without this,
            // a re-declared namespace would only have knowledge of the items declared within this declaration.
            // TODO: why? is this important after I remove the resolution from here?
            visitor.resolver.add_open(&namespace.name, None, root_id);
            for item in &namespace.items {
                match &*item.kind {
                    ItemKind::ImportOrExport(decl) => {
                        // new strategy: will only bind the import here, the imports will be resolved
                        // in the next pass.
                        visitor
                            .resolver
                            .bind_import_or_export_2(decl, Some((ns, &namespace.name)));
                    }
                    ItemKind::Open(PathKind::Ok(path), alias) => {
                        // we only need to bind opens that are in top-level namespaces, outside of callables.
                        // this is because this is for the intermediate export-binding pass
                        // and in the export-binding pass, we only need to know what symbols are available
                        // in the scope of the exports. Exports are only allowed from namespaces scopes.
                        // Put another way,
                        // ```
                        // // this is allowed, so we need to bind the "open B" for the export to work
                        // namespace A { open B; export { SomethingFromB }; }
                        //
                        // // this is disallowed, so we don't need to bind the "open B" for the export
                        // namespace A { callable foo() { open B; export { SomethingFromB }; } }
                        //                                        ^^^^^^ export from non-namespace scope is not allowed
                        // ```
                        visitor
                            .resolver
                            .add_open(path.as_ref(), alias.as_deref(), ns);
                    }
                    _ => ast_visit::walk_item(visitor, item),
                }
            }
        });
    }
}
