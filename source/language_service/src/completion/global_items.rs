// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Completion, text_edits::TextEditRange};
use crate::{
    compilation::Compilation,
    protocol::{CompletionItemKind, TextEdit},
};
use qsc::{
    PRELUDE,
    ast::{
        Idents as _, Package as AstPackage, PathKind,
        visit::{Visitor, walk_block, walk_callable_decl, walk_item, walk_namespace},
    },
    display::CodeDisplay,
    hir::{CallableDecl, Idents, Item, ItemKind, Package, PackageId, Res, Visibility, ty::Udt},
};
use rustc_hash::FxHashSet;
use std::{iter::once, rc::Rc};

/// Provides the globals that are visible or importable at the cursor offset.
pub(super) struct Globals<'a> {
    compilation: &'a Compilation,
    imports: Vec<ImportItem>,
}

struct NamespacesInPackage<'a> {
    package: &'a Package,
    is_user_package: bool,
    namespaces: Vec<Vec<Rc<str>>>,
}

impl<'a> Globals<'a> {
    pub fn init(offset: u32, compilation: &'a Compilation) -> Self {
        let import_finder = ImportFinder::init(offset, &compilation.user_unit().ast.package);

        Self {
            compilation,
            imports: import_finder.imports,
        }
    }

    /// Returns all names that are valid in an expression context,
    /// and available at the current offset,
    /// taking into account any imports that are in scope.
    ///
    /// If the item name is not in scope,
    /// includes the item with text edits (auto-imports, etc.) to bring it into scope.
    pub fn expr_names(&self, edit_range: &TextEditRange) -> Vec<Vec<Completion>> {
        // let mut completions = Vec::new();

        // include UDTs as well since they can be constructors
        let mut completions = self.items(
            true,  // include_callables
            true,  // include_udts
            false, // in_scope_only
            Some(edit_range),
        );

        completions.push(self.namespaces());

        completions
    }

    /// Returns all names that are valid in a type context,
    /// and available at the current offset,
    /// taking into account any imports that are in scope.
    ///
    /// If the item name is not in scope, and `in_scope_only` is false,
    /// includes the item with text edits (auto-imports, etc.) to bring it into scope.
    pub fn type_names(&self, edit_range: &TextEditRange) -> Vec<Vec<Completion>> {
        let mut completions = Vec::new();

        completions.extend(self.items(
            false, // include_callables
            true,  // include_udts
            false, // in_scope_only
            Some(edit_range),
        ));
        completions.push(self.namespaces());

        completions
    }

    /// Returns all names that are valid in an expression context,
    /// and available at the current offset,
    /// taking into account any imports that are in scope.
    ///
    /// Does not
    pub fn expr_names_in_scope_only(&self) -> Vec<Vec<Completion>> {
        // let mut completions = Vec::new();

        // include UDTs as well since they can be constructors
        let mut completions = self.items(
            true, // include_callables
            true, // include_udts
            true, // in_scope_only
            None,
        );

        completions.push(self.namespaces());

        completions
    }

    /// Returns all names that are valid in a type context,
    /// and available at the current offset,
    /// taking into account any imports that are in scope.
    ///
    /// If the item name is not in scope, and `in_scope_only` is false,
    /// includes the item with text edits (auto-imports, etc.) to bring the item into scope.
    pub fn type_names_in_scope_only(&self) -> Vec<Vec<Completion>> {
        let mut completions = Vec::new();

        completions.extend(self.items(
            false, // include_callables
            true,  // include_udts
            true,  // in_scope_only
            None,
        ));
        completions.push(self.namespaces());

        completions
    }

    /// Returns all namespaces in the compilation.
    pub fn namespaces(&self) -> Vec<Completion> {
        let mut completions = self.namespaces_in(&[]);

        let mut c = Vec::new();

        // Also add package aliases for dependencies
        for (is_user_package, package_alias, _package) in self.iter_all_packages() {
            if !is_user_package {
                if let Some(package_alias) = package_alias {
                    c.push(Completion::new(
                        (*package_alias).into(),
                        CompletionItemKind::Module,
                    ));
                }
            }
        }

        completions.push(c);

        completions.into_iter().flatten().collect()
    }

    /// Returns all namespace parts that are valid completions at the current offset,
    /// for the given qualifier, taking into account any imports that are in scope.
    pub fn namespaces_in(&'a self, qualifier: &[Rc<str>]) -> Vec<Vec<Completion>> {
        let namespaces_in_packages = self.matching_namespaces_in_packages(qualifier);

        let mut groups = Vec::new();
        for NamespacesInPackage {
            package,
            is_user_package,
            namespaces,
        } in &namespaces_in_packages
        {
            // Collect all items from all relevant namespaces in this package
            let mut all_items = Vec::new();

            for namespace in namespaces {
                let mut items = Self::namespaces_in_namespace(package, namespace, *is_user_package);
                all_items.append(&mut items);
            }

            // Apply the same deduplication logic as unqualified completion
            // Sort to ensure Export items (if any) come last for deduplication purposes
            all_items.dedup();

            let completions = all_items
                .into_iter()
                .map(|ns_part| Completion::new(ns_part.to_string(), CompletionItemKind::Module))
                .collect();

            groups.push(completions);
        }

        groups
    }

    /// Returns all names that are valid completions at the current offset,
    /// in an expression context, for the given qualifier,
    /// taking into account any imports that are in scope.
    pub fn expr_names_in(&self, qualifier: &[Rc<str>]) -> Vec<Vec<Completion>> {
        let mut groups = self.items_in(
            qualifier, true, // include_callables
            true, // include_udts
        );

        let namespace_groups = self.namespaces_in(qualifier);

        groups.extend(namespace_groups);
        groups
    }

    /// Returns all names that are valid completions at the current offset,
    /// in a type context, for the given qualifier,
    /// taking into account any imports that are in scope.
    pub fn type_names_in(&self, qualifier: &[Rc<str>]) -> Vec<Vec<Completion>> {
        let mut groups = self.items_in(
            qualifier, false, // include_callables
            true,  // include_udts
        );

        groups.extend(self.namespaces_in(qualifier));
        groups
    }

    /// Returns all item names that are available at the current offset,
    /// taking into account any imports that are in scope.
    ///
    /// If the item name is not in scope, and `in_scope_only` is false,
    /// includes the item with text edits (auto-imports, etc.) to bring the item into scope.
    fn items(
        &self,
        include_callables: bool,
        include_udts: bool,
        in_scope_only: bool,
        edit_range: Option<&TextEditRange>,
    ) -> Vec<Vec<Completion>> {
        let mut groups = Vec::new();

        for (is_user_package, package_alias, package) in self.iter_all_packages() {
            // Given the package, get all completion items by iterating over its items
            // and converting any that would be valid as completions into completions
            let mut all_items: Vec<_> = package
                .items
                .values()
                .filter_map(|item| {
                    Self::is_item_relevant(
                        package,
                        item,
                        include_callables,
                        include_udts,
                        is_user_package,
                    )
                })
                .collect();

            // Sort to ensure Export items (if any) come last for deduplication purposes
            all_items.sort_by(|a, b| {
                use std::cmp::Ordering;
                match a.name.cmp(&b.name) {
                    Ordering::Equal => {
                        // For same name, prefer non-Export items first so Export items can override them
                        a.is_export.cmp(&b.is_export)
                    }
                    other => other,
                }
            });

            // Deduplicate by name, preferring the last item (Export items due to sorting)
            let mut seen_names = FxHashSet::default();
            let mut deduplicated_items = Vec::new();

            for item in all_items.into_iter().rev() {
                if !seen_names.contains(&item.name) {
                    seen_names.insert(item.name.clone());
                    deduplicated_items.push(item);
                }
            }
            deduplicated_items.reverse(); // Restore original order

            let completions = deduplicated_items
                .into_iter()
                .filter_map(|item| {
                    let import_info = self.import_info(&item, package_alias);
                    if in_scope_only && !matches!(import_info, ImportInfo::InScope) {
                        return None;
                    }
                    Some(self.to_completion(&item, import_info, edit_range))
                })
                .collect();
            groups.push(completions);
        }
        groups
    }

    /// Returns all item names that are valid completions at the current offset,
    /// for the given qualifier, taking into account any imports that are in scope.
    fn items_in(
        &'a self,
        qualifier: &[Rc<str>],
        include_callables: bool,
        include_udts: bool,
    ) -> Vec<Vec<Completion>> {
        let namespaces_in_packages = self.matching_namespaces_in_packages(qualifier);

        let mut groups = Vec::new();
        for NamespacesInPackage {
            package,
            is_user_package,
            namespaces,
        } in &namespaces_in_packages
        {
            // Collect all items from all relevant namespaces in this package
            let mut all_items = Vec::new();

            for namespace in namespaces {
                let mut items = Self::items_in_namespace(
                    package,
                    namespace,
                    include_callables,
                    include_udts,
                    *is_user_package,
                );
                all_items.append(&mut items);
            }

            // Apply the same deduplication logic as unqualified completion
            // Sort to ensure Export items (if any) come last for deduplication purposes
            all_items.sort_by(|a, b| {
                use std::cmp::Ordering;
                match a.name.cmp(&b.name) {
                    Ordering::Equal => {
                        // For same name, prefer non-Export items first so Export items can override them
                        a.is_export.cmp(&b.is_export)
                    }
                    other => other,
                }
            });

            // Deduplicate by name, preferring the last item (Export items due to sorting)
            let mut seen_names = FxHashSet::default();
            let mut deduplicated_items = Vec::new();

            for item in all_items.into_iter().rev() {
                if !seen_names.contains(&item.name) {
                    seen_names.insert(item.name.clone());
                    deduplicated_items.push(item);
                }
            }
            deduplicated_items.reverse(); // Restore original order

            let completions = deduplicated_items
                .into_iter()
                .map(|item| self.to_completion(&item, ImportInfo::InScope, None))
                .collect();

            groups.push(completions);
        }

        groups
    }

    /// For a given package, returns all items that are in the given namespace.
    fn items_in_namespace(
        package: &'a Package,
        namespace: &[Rc<str>],
        include_callables: bool,
        include_udts: bool,
        is_user_package: bool,
    ) -> Vec<RelevantItem<'a>> {
        let matching_exported_namespaces = namespace_exports(package)
            .filter_map(|(export_to, export_from)| {
                if export_to == namespace {
                    Some(export_from)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let ns_items = package.items.values().find_map(move |i| {
            if let ItemKind::Namespace(candidate_ns, items) = &i.kind {
                let candidate_ns: Vec<Rc<str>> = candidate_ns.into();

                // If the namespace matches exactly, include the items.
                if candidate_ns == namespace || matching_exported_namespaces.contains(&candidate_ns)
                {
                    return Some(items);
                }

                // If we're being asked for the the top-level namespace in a dependency package,
                // include items from the `Main` namespace.
                if namespace.is_empty() && candidate_ns == ["Main".into()] && !is_user_package {
                    return Some(items);
                }
            }
            None
        });

        ns_items
            .into_iter()
            .flatten()
            .filter_map(|item_id| {
                let item = package
                    .items
                    .get(*item_id)
                    .expect("item id should exist in package");

                Self::is_item_relevant(
                    package,
                    item,
                    include_callables,
                    include_udts,
                    is_user_package,
                )
            })
            .chain(
                // Also include Export items that have this namespace as their parent
                // but might not be in the namespace's item list
                package.items.values().filter_map(|item| {
                    if let Some(parent_id) = item.parent {
                        if let Some(parent) = package.items.get(parent_id) {
                            if let ItemKind::Namespace(name, ..) = &parent.kind {
                                let parent_ns: Vec<Rc<str>> = name.into();

                                // Check if this item's parent namespace matches our target
                                let matches_target = if namespace.is_empty()
                                    && parent_ns == ["Main".into()]
                                    && !is_user_package
                                {
                                    true // Main namespace items for dependencies
                                } else {
                                    parent_ns == namespace
                                };

                                if matches_target && matches!(item.kind, ItemKind::Export(_, _)) {
                                    return Self::is_item_relevant(
                                        package,
                                        item,
                                        include_callables,
                                        include_udts,
                                        is_user_package,
                                    );
                                }
                            }
                        }
                    }
                    None
                }),
            )
            .collect()
    }

    /// For a given package, returns all namespace names that are direct
    /// children of the given namespace prefix.
    ///
    /// E.g. if the package contains `Foo.Bar.Baz` and `Foo.Qux`, and
    /// the given prefix is `Foo`, this will return `Bar` and `Qux`.
    fn namespaces_in_namespace(
        package: &'a Package,
        namespace_prefix: &[Rc<str>],
        is_user_package: bool,
    ) -> Vec<Rc<str>> {
        let ns_next_parts = package.items.values().filter_map(move |i| {
            let mut candidate_ns = vec![];
            let next_part = None;
            if let ItemKind::Export(ident, Res::Item(item_id)) = &i.kind {
                let is_namespace_export = item_id.package.is_none()
                    && package
                        .items
                        .get(item_id.item)
                        .is_some_and(|i| matches!(&i.kind, ItemKind::Namespace(..)));
                if is_namespace_export {
                    let parent = i.parent;
                    if let Some(parent_id) = parent {
                        if let Some(parent_ns) = package.items.get(parent_id) {
                            if let ItemKind::Namespace(name, ..) = &parent_ns.kind {
                                let parent_ns_parts: Vec<Rc<str>> = name.into();
                                candidate_ns.extend(parent_ns_parts);
                            }
                        }
                    }

                    candidate_ns.push(ident.name.clone());
                }
            }

            if let ItemKind::Namespace(name, ..) = &i.kind {
                candidate_ns = name.into();
            }
            if !candidate_ns.is_empty() {
                // If this is under the "Main" namespace in a dependent package,
                // skip the "Main" part
                if !is_user_package
                    && !candidate_ns.is_empty()
                    && candidate_ns[0].as_ref() == "Main"
                {
                    candidate_ns.remove(0);
                }

                // If the namespace matches exactly, include the items.
                if candidate_ns.starts_with(namespace_prefix)
                    && candidate_ns.len() > namespace_prefix.len()
                {
                    #[allow(clippy::unnecessary_literal_unwrap)]
                    let next_part =
                        next_part.unwrap_or_else(|| candidate_ns[namespace_prefix.len()].clone());
                    return Some(next_part);
                }

                // If we're being asked for the the top-level namespace in a dependency package,
                // include items from the `Main` namespace.
                // if namespace_prefix.is_empty()
                //     && candidate_ns.len() > 1
                //     && candidate_ns[0].as_ref() == "Main"
                //     && !is_user_package
                // {
                //     return Some(candidate_ns[1].clone());
                // }
            }
            None
        });

        ns_next_parts.collect()
    }

    /// Given a qualifier, and any imports that are in scope,
    /// produces a list of namespaces, grouped by package,
    /// that this qualifier matches. The namespaces don't have to exist.
    fn matching_namespaces_in_packages(&self, qualifier: &[Rc<str>]) -> Vec<NamespacesInPackage> {
        let namespaces = self.matching_namespaces(qualifier);

        let mut packages_and_namespaces = Vec::new();

        for (is_user_package, package_alias, package) in self.iter_all_packages() {
            let mut namespaces_for_package = Vec::new();

            for namespace in &namespaces {
                if let Some(package_alias) = package_alias {
                    // Only include the namespace if it starts with this package's alias
                    if !namespace.is_empty() && *namespace[0] == *package_alias {
                        namespaces_for_package.push(namespace[1..].to_vec());
                    }
                } else {
                    // No package alias, always include the namespace
                    namespaces_for_package.push(namespace.clone());
                }
            }
            packages_and_namespaces.push(NamespacesInPackage {
                package,
                is_user_package,
                namespaces: namespaces_for_package,
            });
        }

        packages_and_namespaces
    }

    /// Given a qualifier, and any imports that are in scope,
    /// produces a list of potential namespaces this qualifier could match.
    /// The namespaces don't have to exist.
    fn matching_namespaces(&self, qualifier: &[Rc<str>]) -> Vec<Vec<Rc<str>>> {
        let mut namespaces: Vec<Vec<Rc<str>>> = Vec::new();
        // Add the qualifier as is
        namespaces.push(qualifier.to_vec());

        // Add qualifier prefixed with all opened namespaces
        let opened_namespaces = self
            .imports
            .iter()
            .filter_map(|import_item| {
                if import_item.is_glob {
                    Some(import_item.path.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for open_namespace in &opened_namespaces {
            namespaces.push([open_namespace, qualifier].concat());
        }

        // Does `qualifier` start with a namespace alias?
        let full_qualifier_for_aliased_ns = self
            .imports
            .iter()
            .find_map(|import_item| {
                if !qualifier.is_empty()
                    && import_item
                        .alias
                        .as_ref()
                        .is_some_and(|s| **s == *qualifier[0])
                {
                    Some(&import_item.path)
                } else {
                    None
                }
            })
            .map(|full_ns_for_alias| {
                let rest = &qualifier[1..];
                [full_ns_for_alias, rest].concat()
            });

        // Add any aliased namespaces with an alias that matches the qualifier
        if let Some(full_qualifier_for_aliased_ns) = &full_qualifier_for_aliased_ns {
            namespaces.push(full_qualifier_for_aliased_ns.clone());
        }
        namespaces
    }

    /// Returns the core package, then the dependencies in reverse order,
    /// then finally the user package.
    ///
    /// Iterating in this order ensures that the user package items appear
    /// first, and indirect dependencies are lower on the completion list.
    ///
    /// Returns the tuple of `(is_user_package, alias, package)`
    fn iter_all_packages(&self) -> impl Iterator<Item = (bool, Option<&'a str>, &'a Package)> + 'a {
        let packages = self
            .compilation
            .package_store
            .iter()
            .rev()
            .filter_map(|(id, unit)| {
                if self.compilation.user_package_id == id {
                    return Some((true, None, &unit.package));
                }
                self.compilation
                    .dependencies
                    .get(&id)
                    .map(|alias| (false, alias.as_ref().map(AsRef::as_ref), &unit.package))
            });
        once((
            false,
            None,
            &self
                .compilation
                .package_store
                .get(PackageId::CORE)
                .expect("core package must exist")
                .package,
        ))
        .chain(packages)
    }

    /// Whether an exact import exists for the given path, along with its alias if one exists.
    fn exact_import_exists(&self, path: &[Rc<str>]) -> (bool, Option<Rc<str>>) {
        let exact_import = self.imports.iter().find_map(|import_item| {
            if import_item.is_glob {
                return None;
            }

            if import_item.path == path {
                Some(import_item.alias.clone())
            } else {
                None
            }
        });
        (exact_import.is_some(), exact_import.unwrap_or_default())
    }

    /// An item is "relevant" if it's a callable or UDT that's visible to the user package.
    fn is_item_relevant(
        package: &'a qsc::hir::Package,
        item: &'a qsc::hir::Item,
        include_callables: bool,
        include_udts: bool,
        is_user_package: bool,
    ) -> Option<RelevantItem<'a>> {
        // We only want items whose parents are namespaces
        if let Some(item_id) = item.parent {
            if let Some(parent) = package.items.get(item_id) {
                if let ItemKind::Namespace(namespace, _) = &parent.kind {
                    // filter out internal packages that are not from the user's
                    // compilation
                    if matches!(item.visibility, Visibility::Internal) && !is_user_package {
                        return None; // ignore item if not in the user's package
                    }
                    if !is_user_package
                        && namespace.name().to_lowercase().starts_with("std.openqasm")
                    {
                        return None; // ignore item if in a QASM namespace
                    }
                    return match &item.kind {
                        ItemKind::Callable(callable_decl) if include_callables => {
                            Some(RelevantItem {
                                name: callable_decl.name.name.clone(),
                                namespace,
                                kind: RelevantItemKind::Callable(callable_decl),
                                is_export: false,
                            })
                        }
                        ItemKind::Ty(_, udt) if include_udts => Some(RelevantItem {
                            name: udt.name.clone(),
                            namespace,
                            kind: RelevantItemKind::Udt(udt),
                            is_export: false,
                        }),
                        ItemKind::Export(export_name, Res::Item(export_item_id)) => {
                            // For Export items, resolve to the underlying item for completion
                            if export_item_id.package.is_some() {
                                // Export references item from another package - skip for now
                                return None;
                            }

                            // Export references item from same package - resolve it
                            if let Some(underlying_item) = package.items.get(export_item_id.item) {
                                match &underlying_item.kind {
                                    ItemKind::Callable(callable_decl) if include_callables => {
                                        Some(RelevantItem {
                                            name: export_name.name.clone(), // Use export name (might be alias)
                                            namespace,
                                            kind: RelevantItemKind::Callable(callable_decl),
                                            is_export: true,
                                        })
                                    }
                                    ItemKind::Ty(_, udt) if include_udts => Some(RelevantItem {
                                        name: export_name.name.clone(), // Use export name (might be alias)
                                        namespace,
                                        kind: RelevantItemKind::Udt(udt),
                                        is_export: true,
                                    }),
                                    _ => {
                                        // TODO: Also check if this Export item references a namespace
                                        // and make that namespace available as a module
                                        None
                                    }
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                }
            }
        }
        None
    }

    /// For a given item, produces any auto-imports, prefixes or aliases that would
    /// make that item a valid completion in the current scope.
    fn import_info(&self, item: &RelevantItem<'a>, package_alias: Option<&str>) -> ImportInfo {
        let namespace_without_pkg_alias = Into::<Vec<_>>::into(item.namespace);
        let mut namespace = namespace_without_pkg_alias.clone();
        if let Some(package_alias) = package_alias {
            namespace.insert(0, package_alias.into());
        }

        // Is there a glob import for the namespace, i.e. is the name already in scope?
        let glob_import = self
            .imports
            .iter()
            .any(|import_item| import_item.path == namespace && import_item.is_glob);

        if glob_import {
            return ImportInfo::InScope;
        }

        // Special case: If this item is from a dependency's Main namespace,
        // check if there's a glob import for the package (without the Main part)
        if let Some(package_alias) = package_alias {
            if namespace_without_pkg_alias == ["Main".into()] {
                let package_glob_import = self.imports.iter().any(|import_item| {
                    import_item.path == [package_alias.into()] && import_item.is_glob
                });

                if package_glob_import {
                    return ImportInfo::InScope;
                }
            }
        }

        // An exact import is an import that matches the namespace and item name exactly
        let (exact_import, item_alias) =
            self.exact_import_exists(&[namespace.as_slice(), &[item.name.clone()]].concat());

        if exact_import {
            if let Some(alias) = item_alias {
                return ImportInfo::Alias(alias);
            }
            return ImportInfo::InScope;
        }

        // Does an alias for the namespace exist?
        let namespace_alias = self.exact_import_exists(&namespace).1;

        if let Some(namespace_alias) = namespace_alias {
            return ImportInfo::InAliasNamespace(namespace_alias);
        }

        // If there are no existing exact or glob imports of the item,
        // no open aliases for the namespace it's in,
        // and we are not in the same namespace as the item,
        // we need to add an import for it.
        ImportInfo::NeedAutoImport(fully_qualify_name(
            package_alias,
            &namespace_without_pkg_alias,
            Some(&item.name),
        ))
    }

    /// Creates a completion list entry for the given item, including
    /// any text edits that would bring the item into scope, if requested.
    fn to_completion(
        &self,
        item: &RelevantItem<'a>,
        import_info: ImportInfo,
        text_edits: Option<&TextEditRange>,
    ) -> Completion {
        let display = CodeDisplay {
            compilation: self.compilation,
        };
        let (kind, display) = match &item.kind {
            RelevantItemKind::Callable(callable_decl) => (
                CompletionItemKind::Function,
                display.hir_callable_decl(callable_decl).to_string(),
            ),
            RelevantItemKind::Udt(udt) => (
                CompletionItemKind::Interface,
                display.hir_udt(udt).to_string(),
            ),
        };

        // Deprioritize names starting with "__" in the completion list
        let mut sort_priority = u32::from(item.name.starts_with("__"));

        match import_info {
            ImportInfo::InScope => Completion::with_text_edits(
                item.name.to_string(),
                kind,
                Some(display),
                None,
                sort_priority,
            ),
            ImportInfo::NeedAutoImport(import_path) => {
                // Deprioritize auto-import items
                sort_priority += 1;

                let text_edits = text_edits.expect(
                    "a text edit range should have been provided if `in_scope_only` is false",
                );
                // if there is no place to insert an import, then we can't add an import.
                let edits = text_edits.insert_import_at.as_ref().map(|range| {
                    vec![TextEdit {
                        new_text: format!("import {};{}", import_path, &text_edits.indent),
                        range: *range,
                    }]
                });

                Completion::with_text_edits(
                    item.name.to_string(),
                    kind,
                    Some(display),
                    edits,
                    sort_priority,
                )
            }
            ImportInfo::InAliasNamespace(prefix) => Completion::with_text_edits(
                format!("{}.{}", prefix, item.name),
                kind,
                Some(display),
                None,
                sort_priority,
            ),
            ImportInfo::Alias(alias) => Completion::with_text_edits(
                alias.to_string(),
                kind,
                Some(display),
                None,
                sort_priority,
            ),
        }
    }
}

fn namespace_exports(package: &Package) -> impl Iterator<Item = (Vec<Rc<str>>, Vec<Rc<str>>)> {
    package.items.iter().filter_map(|(_, item)| {
        if let ItemKind::Export(ident, Res::Item(exported_item_id)) = &item.kind {
            if exported_item_id.package.is_none() {
                if let Some(exported_namespace_id) = package.items.iter().find_map(|(id, item)| {
                    if id == exported_item_id.item {
                        if let ItemKind::Namespace(name, ..) = &item.kind {
                            let exported_namespace: Vec<Rc<str>> = name.into();
                            return Some(exported_namespace);
                        }
                    }
                    None
                }) {
                    if let Some(parent_id) = &item.parent {
                        let parent = package.items.get(*parent_id);
                        if let Some(Item {
                            kind: ItemKind::Namespace(name, ..),
                            ..
                        }) = parent
                        {
                            let mut export_path: Vec<Rc<str>> = name.into();
                            export_path.push(ident.name.clone());
                            return Some((export_path, exported_namespace_id));
                        }
                    }
                }
            }
        }
        None
    })
}

enum ImportInfo {
    /// Item name is already in scope, no edits necessary.
    InScope,
    /// The path that we should add an auto-import for, if any.
    NeedAutoImport(String),
    /// The item name should be prefixed with the namespace alias.
    ///
    /// e.g. `Foo` will appear as `Bar.Foo` if it's under an open
    /// namespace that is aliased as `Bar`.
    InAliasNamespace(Rc<str>),
    /// The item was imported under an alias.
    Alias(Rc<str>),
}

/// A callable or UDT that's visible to the user package.
enum RelevantItemKind<'a> {
    Callable(&'a CallableDecl),
    Udt(&'a Udt),
}

/// A callable or UDT that's visible to the user package.
struct RelevantItem<'a> {
    name: Rc<str>,
    kind: RelevantItemKind<'a>,
    namespace: &'a Idents,
    is_export: bool, // True if this item comes from an Export HIR item
}

/// Format an external fully qualified name.
/// This will prepend the package alias and remove `Main` if it is the first namespace.
fn fully_qualify_name(
    package_alias: Option<&str>,
    namespace: &[Rc<str>],
    name: Option<&str>,
) -> String {
    let mut fully_qualified_name: Vec<Rc<str>> = if let Some(alias) = package_alias {
        vec![Rc::from(alias)]
    } else {
        vec![]
    };

    // if this comes from an external project's Main, then the path does not include Main
    let item_comes_from_main_of_external_project = package_alias.is_some()
        && namespace.len() == 1
        && namespace.first() == Some(&"Main".into());

    // So, if it is _not_ from an external project's `Main`, we include the namespace in the fully
    // qualified name.
    if !(item_comes_from_main_of_external_project) {
        fully_qualified_name.append(&mut namespace.to_vec());
    }

    if let Some(name) = name {
        fully_qualified_name.push(name.into());
    }

    fully_qualified_name.join(".")
}

#[derive(Default)]
struct ImportFinder {
    offset: u32,
    // The available imports at the current location
    imports: Vec<ImportItem>,
}

impl ImportFinder {
    fn init(offset: u32, package: &AstPackage) -> Self {
        let mut context = Self {
            offset,
            ..Self::default()
        };
        context.visit_package(package);

        let mut prelude_ns_ids: Vec<ImportItem> = PRELUDE
            .iter()
            .map(|ns| ImportItem {
                path: ns.iter().map(|x| Rc::from(*x)).collect(),
                alias: None,
                is_glob: true,
            })
            .collect();

        // The PRELUDE namespaces are always implicitly opened.
        context.imports.append(&mut prelude_ns_ids);

        context
    }
}

impl<'a> Visitor<'a> for ImportFinder {
    fn visit_namespace(&mut self, namespace: &'a qsc::ast::Namespace) {
        if namespace.span.contains(self.offset) {
            // the current namespace is implicitly opened.
            self.imports = vec![ImportItem {
                path: namespace.name.rc_str_iter().cloned().collect(),
                alias: None,
                is_glob: true,
            }];
            walk_namespace(self, namespace);
        }
    }

    fn visit_item(&mut self, item: &'a qsc::ast::Item) {
        match &*item.kind {
            qsc::ast::ItemKind::Open(PathKind::Ok(name), alias) => {
                let open_as_import = ImportItem {
                    path: name.rc_str_iter().cloned().collect(),
                    alias: alias.as_ref().map(|x| x.name.clone()),
                    is_glob: alias.is_none(),
                };
                self.imports.push(open_as_import);
            }
            qsc::ast::ItemKind::ImportOrExport(decl) => {
                // if this is an import, populate self.imports
                if decl.is_import() {
                    self.imports
                        .append(&mut ImportItem::from_import_or_export_item(decl));
                }
            }
            _ => (),
        }

        if item.span.contains(self.offset) {
            walk_item(self, item);
        }
    }

    fn visit_callable_decl(&mut self, decl: &'a qsc::ast::CallableDecl) {
        if decl.span.contains(self.offset) {
            // This span covers the body too, but the
            // context will get overwritten by visit_block
            // if the offset is inside the actual body
            walk_callable_decl(self, decl);
        }
    }

    fn visit_block(&mut self, block: &'a qsc::ast::Block) {
        if block.span.contains(self.offset) {
            walk_block(self, block);
        }
    }
}

#[derive(Debug)]
/// Used to represent pre-existing imports in the completion context
struct ImportItem {
    path: Vec<Rc<str>>,
    alias: Option<Rc<str>>,
    is_glob: bool,
}

impl ImportItem {
    fn from_import_or_export_item(decl: &qsc::ast::ImportOrExportDecl) -> Vec<Self> {
        if decl.is_export() {
            return vec![];
        }
        let mut buf = Vec::with_capacity(decl.items.len());
        for item in &decl.items {
            let PathKind::Ok(path) = &item.path else {
                continue;
            };
            let (alias, is_glob) = match &item.kind {
                qsc::ast::ImportKind::Wildcard => (None, true),
                qsc::ast::ImportKind::Direct { alias } => {
                    (alias.as_ref().map(|x| x.name.clone()), false)
                }
            };

            buf.push(ImportItem {
                path: path.rc_str_iter().cloned().collect(),
                alias,
                is_glob,
            });
        }
        buf
    }
}
