// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{text_edits::TextEditRange, Completion};
use crate::{
    compilation::Compilation,
    protocol::{CompletionItemKind, TextEdit},
};
use qsc::{
    ast::{
        visit::{walk_block, walk_callable_decl, walk_item, walk_namespace, Visitor},
        Idents as _, Package as AstPackage, PathKind,
    },
    display::CodeDisplay,
    hir::{ty::Udt, CallableDecl, Idents, ItemKind, Package, PackageId, Visibility},
    PRELUDE,
};
use rustc_hash::FxHashSet;
use std::{iter::once, rc::Rc};

/// Provides the globals that are visible or importable at the cursor offset.
pub(super) struct Globals<'a> {
    compilation: &'a Compilation,
    imports: Vec<ImportItem>,
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
        // Use the user package's namespace tree to find all top-level namespaces
        let user_package = &self.compilation.user_unit().package;
        let mut completions = Self::namespaces_in_user_package_tree(user_package, &[]);

        // Also add package aliases for dependencies
        for (is_user_package, package_alias, _package) in self.iter_all_packages() {
            if !is_user_package {
                if let Some(package_alias) = package_alias {
                    completions.push(Completion::new(
                        (*package_alias).into(),
                        CompletionItemKind::Module,
                    ));
                }
            }
        }

        completions
    }

    /// Returns all namespaces that are valid completions at the current offset,
    /// for the given qualifier.
    /// Uses only the user package's namespace tree since it should contain
    /// all resolved namespaces from dependencies.
    pub fn namespaces_in(&self, qualifier: &[Rc<str>]) -> Vec<Vec<Completion>> {
        // Get the user package
        let user_package = &self.compilation.user_unit().package;

        // Use the user package's namespace tree to find completions
        let completions = Self::namespaces_in_user_package_tree(user_package, qualifier);

        vec![completions]
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

        for (i, group) in namespace_groups.iter().enumerate() {
            eprintln!("DEBUG: namespace group {}: {} completions", i, group.len());
            for completion in group {
                eprintln!("DEBUG: - {}", completion.item.label);
            }
        }

        groups.extend(namespace_groups);
        eprintln!("DEBUG: expr_names_in - total groups: {}", groups.len());
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
        eprintln!("DEBUG: items_in called with qualifier: {qualifier:?}, include_callables: {include_callables}, include_udts: {include_udts}");
        let namespaces_in_packages = self.matching_namespaces_in_packages(qualifier);

        let mut groups = Vec::new();
        for (package, is_user_package, namespaces) in &namespaces_in_packages {
            eprintln!("DEBUG: processing package is_user_package: {is_user_package}, namespaces: {namespaces:?}");
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

    /// For a given package, returns all namespace names that are direct
    /// children of the given namespace prefix.
    ///
    /// E.g. if the package contains `Foo.Bar.Baz` and `Foo.Qux` , and
    /// the given prefix is `Foo` , this will return `Bar` and `Qux`.
    /// Get namespace completions from the user package's namespace tree.
    /// The user package's namespace tree should contain all resolved namespaces
    /// from dependencies, so we only need to search in one place.
    fn namespaces_in_user_package_tree(
        user_package: &Package,
        qualifier: &[Rc<str>],
    ) -> Vec<Completion> {
        // Get the namespace ID for the qualifier
        let namespace_id = if qualifier.is_empty() {
            // Root namespace
            Some(user_package.namespaces.root_id())
        } else {
            // Convert qualifier to &str slice for the API
            let qualifier_strs: Vec<&str> = qualifier.iter().map(AsRef::as_ref).collect();
            user_package.namespaces.get_namespace_id(qualifier_strs)
        };

        if let Some(ns_id) = namespace_id {
            // Get the namespace node and its children
            let (_, namespace_node) = user_package.namespaces.find_namespace_by_id(&ns_id);
            let borrowed_node = namespace_node.borrow();
            let children = borrowed_node.children();

            children
                .keys()
                .map(|child_name| {
                    Completion::new(child_name.to_string(), CompletionItemKind::Module)
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// For a given package, returns all items that are in the given namespace.
    fn items_in_namespace(
        package: &'a Package,
        namespace: &[Rc<str>],
        include_callables: bool,
        include_udts: bool,
        is_user_package: bool,
    ) -> Vec<RelevantItem<'a>> {
        let ns_items = package.items.values().find_map(move |i| {
            if let ItemKind::Namespace(candidate_ns, items) = &i.kind {
                let candidate_ns: Vec<Rc<str>> = candidate_ns.into();

                // If the namespace matches exactly, include the items.
                if candidate_ns == namespace {
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
                            if let ItemKind::Namespace(parent_ns, _) = &parent.kind {
                                let parent_ns: Vec<Rc<str>> = parent_ns.into();

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

    /// Given a qualifier, and any imports that are in scope,
    /// produces a list of `(package, is_user_package, namespaces)`
    /// tuples that this qualifier could match.
    #[allow(clippy::type_complexity)]
    fn matching_namespaces_in_packages(
        &self,
        qualifier: &[Rc<str>],
    ) -> Vec<(&Package, bool, Vec<Vec<Rc<str>>>)> {
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
            packages_and_namespaces.push((package, is_user_package, namespaces_for_package));
        }

        packages_and_namespaces
    }

    /// Given a qualifier, and any imports that are in scope,
    /// produces a list of potential namespaces this qualifier could match.
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
                        ItemKind::Export(export_name, export_item_id) => {
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
            let alias = item.alias.as_ref().map(|x| x.name.clone());
            let is_glob = item.is_glob;

            buf.push(ImportItem {
                path: path.rc_str_iter().cloned().collect(),
                alias,
                is_glob,
            });
        }
        buf
    }
}
