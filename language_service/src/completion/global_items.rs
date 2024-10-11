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
    display::{CodeDisplay, Lookup},
    hir::{ty::Udt, CallableDecl, ItemId, ItemKind, Package, PackageId, Visibility},
    resolve::Res,
    PRELUDE,
};
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
    /// Does not add any auto-import edits.
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
    /// Does not add any auto-import edits.
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
        let mut completions = Vec::new();

        completions.extend(self.namespaces_in(&[]).into_iter().flatten());
        completions
    }

    /// Returns all namespaces that are valid completions at the current offset,
    /// for the given qualifier.
    pub fn namespaces_in(&self, qualifier: &[Rc<str>]) -> Vec<Vec<Completion>> {
        let namespaces = self.matching_namespaces(qualifier);
        let mut completions = Vec::new();

        for parent_ns in namespaces {
            let global_scope = &self.compilation.user_unit().ast.globals;

            let ns_id = global_scope
                .namespaces
                .get_namespace_id(parent_ns.iter().map(AsRef::as_ref));

            if let Some(ns_id) = ns_id {
                let (_, node) = global_scope.namespaces.find_namespace_by_id(&ns_id);
                for child_name in node.borrow().children().keys() {
                    completions.push(Completion::new(
                        child_name.to_string(),
                        CompletionItemKind::Module,
                    ));
                }
            }
        }

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

        groups.extend(self.namespaces_in(qualifier));
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

        for (is_user_package, package_alias, package, package_id) in self.iter_all_packages() {
            // Given the package, get all completion items by iterating over its items
            // and converting any that would be valid as completions into completions
            let completions = package
                .items
                .values()
                .filter_map(|item| {
                    self.is_item_relevant(
                        package_id,
                        package,
                        item,
                        include_callables,
                        include_udts,
                        is_user_package,
                    )
                })
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
        let namespaces = self.matching_namespaces(qualifier);
        let mut completions = Vec::new();

        for parent_ns in namespaces {
            let global_scope = &self.compilation.user_unit().ast.globals;

            let ns_id = global_scope
                .namespaces
                .get_namespace_id(parent_ns.iter().map(AsRef::as_ref));

            if let Some(ns_id) = ns_id {
                // if include_udts {
                //     let _tys = global_scope.tys.get(ns_id).unwrap_or(&FxHashMap::default());
                // }

                // if include_callables {
                let callables = global_scope.terms.get(ns_id);
                let Some(callables) = callables else { continue };

                for (name, res) in callables {
                    let Res::Item(item_id, _item_status) = res else {
                        panic!("expected res to be an item")
                    };

                    let (item, _package, _item_id) = self
                        .compilation
                        .resolve_item_relative_to_user_package(item_id);

                    let relevant_item = match &item.kind {
                        ItemKind::Callable(callable) => RelevantItem {
                            name: name.clone(),
                            kind: RelevantItemKind::Callable(callable),
                            namespace: parent_ns.clone(),
                        },
                        ItemKind::Ty(_, udt) => RelevantItem {
                            name: name.clone(),
                            kind: RelevantItemKind::Udt(udt),
                            namespace: parent_ns.clone(),
                        },
                        _ => panic!("only expect callables and UDTs in the terms table"),
                    };

                    if include_callables
                        && matches!(relevant_item.kind, RelevantItemKind::Callable(_))
                        || include_udts && matches!(relevant_item.kind, RelevantItemKind::Udt(_))
                    {
                        completions.push(self.to_completion(
                            &relevant_item,
                            ImportInfo::InScope,
                            None,
                        ));
                    }
                }
                // }

                // let (_, node) = global_scope.namespaces.find_namespace_by_id(&ns_id);

                // for (child_name, child_node) in node.borrow().children() {
                //     completions.push(Completion::new(
                //         child_name.to_string(),
                //         CompletionItemKind::Module,
                //     ));
                // }
            }
        }

        vec![completions]
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
    /// Returns the tuple of `(is_user_package, alias, package, package_id)`
    fn iter_all_packages(
        &self,
    ) -> impl Iterator<Item = (bool, Option<&'a str>, &'a Package, PackageId)> + 'a {
        let packages = self
            .compilation
            .package_store
            .iter()
            .rev()
            .filter_map(|(id, unit)| {
                if self.compilation.user_package_id == id {
                    return Some((true, None, &unit.package, id));
                }
                self.compilation
                    .dependencies
                    .get(&id)
                    .map(|alias| (false, alias.as_ref().map(AsRef::as_ref), &unit.package, id))
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
            PackageId::CORE,
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
        &self,
        package_id: PackageId,
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

                    return match &item.kind {
                        ItemKind::Callable(callable_decl) if include_callables => {
                            Some(RelevantItem {
                                name: callable_decl.name.name.clone(),
                                namespace: namespace.into(),
                                kind: RelevantItemKind::Callable(callable_decl),
                            })
                        }
                        ItemKind::Ty(_, udt) if include_udts => Some(RelevantItem {
                            name: udt.name.clone(),
                            namespace: namespace.into(),
                            kind: RelevantItemKind::Udt(udt),
                        }),
                        ItemKind::Export(name, item_id) => {
                            let (
                                item,
                                package,
                                ItemId {
                                    package: package_id,
                                    item: _,
                                },
                            ) = self.compilation.resolve_item(package_id, item_id);
                            return self
                                .is_item_relevant(
                                    package_id.expect("package id should have been resolved"),
                                    package,
                                    item,
                                    include_callables,
                                    include_udts,
                                    is_user_package,
                                )
                                .map(|item| {
                                    // Use the name and namespace from the export, but the
                                    // hir node from the original item.
                                    RelevantItem {
                                        name: name.name.clone(),
                                        namespace: namespace.into(),
                                        kind: item.kind,
                                    }
                                });
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
        let namespace_without_pkg_alias = item.namespace.clone();
        let mut namespace = namespace_without_pkg_alias.clone();
        if let Some(package_alias) = package_alias {
            if namespace
                .first()
                .expect("namespace should have at least one part")
                .as_ref()
                == "Main"
            {
                namespace.remove(0);
            }
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
#[derive(Debug)]
enum RelevantItemKind<'a> {
    Callable(&'a CallableDecl),
    Udt(&'a Udt),
}

/// A callable or UDT that's visible to the user package.
#[derive(Debug)]
struct RelevantItem<'a> {
    name: Rc<str>,
    kind: RelevantItemKind<'a>,
    namespace: Vec<Rc<str>>,
}

/// Format an external fully qualified name.
/// This will prepend the package alias and remove `Main` if it is the first namespace.
// TODO: I feel like this can be fixed up before we call here
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
    };

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
        };
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
