// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::display::{increase_header_level, parse_doc_for_summary};
use crate::display::{CodeDisplay, Lookup};
use qsc_ast::ast;
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_data_structures::target::TargetCapabilityFlags;
use qsc_frontend::compile::{self, compile, Dependencies, PackageStore, SourceMap};
use qsc_frontend::resolve;
use qsc_hir::hir::{CallableKind, Item, ItemKind, Package, PackageId, Visibility};
use qsc_hir::{hir, ty};
use rustc_hash::FxHashMap;
use std::fmt::{Display, Formatter, Result};
use std::rc::Rc;
use std::sync::Arc;

type Files = Vec<(Arc<str>, Arc<str>, Arc<str>)>;

/// Represents an immutable compilation state.
#[derive(Debug)]
struct Compilation {
    /// Package store, containing the current package and all its dependencies.
    package_store: PackageStore,
}

impl Compilation {
    /// Creates a new `Compilation` by compiling standard library
    /// and additional sources.
    pub(crate) fn new(
        additional_program: Option<(PackageStore, &Dependencies, SourceMap)>,
        capabilities: Option<TargetCapabilityFlags>,
        language_features: Option<LanguageFeatures>,
    ) -> Self {
        let actual_capabilities = capabilities.unwrap_or_default();
        let actual_language_features = language_features.unwrap_or_default();

        let package_store =
            if let Some((mut package_store, dependencies, sources)) = additional_program {
                let unit = compile(
                    &package_store,
                    dependencies,
                    sources,
                    actual_capabilities,
                    actual_language_features,
                );
                // We ignore errors here (unit.errors vector) and use whatever
                // documentation we can produce. In future we may consider
                // displaying the fact of error presence on documentation page.

                package_store.insert(unit);
                package_store
            } else {
                let mut package_store = PackageStore::new(compile::core());
                let std_unit = compile::std(&package_store, actual_capabilities);
                package_store.insert(std_unit);
                package_store
            };

        Self { package_store }
    }
}

impl Lookup for Compilation {
    fn get_ty(&self, _: ast::NodeId) -> Option<&ty::Ty> {
        unimplemented!("Not needed for docs generation")
    }

    fn get_res(&self, _: ast::NodeId) -> Option<&resolve::Res> {
        unimplemented!("Not needed for docs generation")
    }

    fn resolve_item_relative_to_user_package(
        &self,
        _: &hir::ItemId,
    ) -> (&hir::Item, &hir::Package, hir::ItemId) {
        unimplemented!("Not needed for docs generation")
    }

    /// Returns the hir `Item` node referred to by `res`.
    /// `Res`s can resolve to external packages, and the references
    /// are relative, so here we also need the
    /// local `PackageId` that the `res` itself came from.
    fn resolve_item_res(
        &self,
        local_package_id: PackageId,
        res: &hir::Res,
    ) -> (&hir::Item, hir::ItemId) {
        match res {
            hir::Res::Item(item_id) => {
                let (item, _, resolved_item_id) = self.resolve_item(local_package_id, item_id);
                (item, resolved_item_id)
            }
            _ => panic!("expected to find item"),
        }
    }

    /// Returns the hir `Item` node referred to by `item_id`.
    /// `ItemId`s can refer to external packages, and the references
    /// are relative, so here we also need the local `PackageId`
    /// that the `ItemId` originates from.
    fn resolve_item(
        &self,
        local_package_id: PackageId,
        item_id: &hir::ItemId,
    ) -> (&hir::Item, &hir::Package, hir::ItemId) {
        // If the `ItemId` contains a package id, use that.
        // Lack of a package id means the item is in the
        // same package as the one this `ItemId` reference
        // came from. So use the local package id passed in.
        let package_id = item_id.package.unwrap_or(local_package_id);
        let package = &self
            .package_store
            .get(package_id)
            .expect("package should exist in store")
            .package;
        (
            package
                .items
                .get(item_id.item)
                .expect("item id should exist"),
            package,
            hir::ItemId {
                package: Some(package_id),
                item: item_id.item,
            },
        )
    }
}

/// Generates and returns documentation files for the standard library
/// and additional sources (if specified.)
#[must_use]
pub fn generate_docs(
    additional_sources: Option<(PackageStore, &Dependencies, SourceMap)>,
    capabilities: Option<TargetCapabilityFlags>,
    language_features: Option<LanguageFeatures>,
) -> Files {
    let compilation = Compilation::new(additional_sources, capabilities, language_features);
    let mut files: Files = vec![];

    let display = &CodeDisplay {
        compilation: &compilation,
    };

    let mut toc: FxHashMap<Rc<str>, Vec<String>> = FxHashMap::default();
    for (_, unit) in &compilation.package_store {
        let package = &unit.package;
        for (_, item) in &package.items {
            if let Some((ns, line)) = generate_doc_for_item(package, item, display, &mut files) {
                toc.entry(ns).or_default().push(line);
            }
        }
    }

    generate_toc(&mut toc, &mut files);

    files
}

fn generate_doc_for_item<'a>(
    package: &'a Package,
    item: &'a Item,
    display: &'a CodeDisplay,
    files: &mut Files,
) -> Option<(Rc<str>, String)> {
    // Filter items
    if item.visibility == Visibility::Internal || matches!(item.kind, ItemKind::Namespace(_, _)) {
        return None;
    }

    // Get namespace for item
    let ns = get_namespace(package, item)?;

    // Add file
    let (metadata, content) = generate_file(&ns, item, display)?;
    let file_name: Arc<str> = Arc::from(format!("{ns}/{}.md", metadata.name).as_str());
    let file_metadata: Arc<str> = Arc::from(metadata.to_string().as_str());
    let file_content: Arc<str> = Arc::from(content.as_str());
    files.push((file_name, file_metadata, file_content));

    // Create toc line
    let line = format!("  - {{name: {}, uid: {}}}", metadata.name, metadata.uid);

    // Return (ns, line)
    Some((ns.clone(), line))
}

fn get_namespace(package: &Package, item: &Item) -> Option<Rc<str>> {
    match item.parent {
        Some(local_id) => {
            let parent = package
                .items
                .get(local_id)
                .expect("Could not resolve parent item id");
            match &parent.kind {
                ItemKind::Namespace(name, _) => {
                    if name.starts_with("QIR") {
                        None // We ignore "QIR" namespaces
                    } else {
                        Some(name.name())
                    }
                }
                _ => None,
            }
        }
        None => None,
    }
}

fn generate_file(ns: &Rc<str>, item: &Item, display: &CodeDisplay) -> Option<(Metadata, String)> {
    let metadata = get_metadata(ns.clone(), item, display)?;

    let doc = increase_header_level(&item.doc);
    let title = &metadata.title;
    let sig = &metadata.signature;

    let content = format!(
        "# {title}

Namespace: {ns}

```qsharp
{sig}
```
"
    );

    let content = if doc.is_empty() {
        content
    } else {
        format!("{content}\n{doc}\n")
    };

    Some((metadata, content))
}

struct Metadata {
    uid: String,
    title: String,
    topic: String,
    kind: MetadataKind,
    namespace: Rc<str>,
    name: Rc<str>,
    summary: String,
    signature: String,
}

impl Display for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let kind = match &self.kind {
            MetadataKind::Function => "function",
            MetadataKind::Operation => "operation",
            MetadataKind::Udt => "udt",
        };
        write!(
            f,
            "---
uid: {}
title: {}
ms.date: {{TIMESTAMP}}
ms.topic: {}
qsharp.kind: {}
qsharp.namespace: {}
qsharp.name: {}
qsharp.summary: \"{}\"
---",
            self.uid, self.title, self.topic, kind, self.namespace, self.name, self.summary
        )
    }
}

enum MetadataKind {
    Function,
    Operation,
    Udt,
}

fn get_metadata(ns: Rc<str>, item: &Item, display: &CodeDisplay) -> Option<Metadata> {
    let (name, signature, kind) = match &item.kind {
        ItemKind::Callable(decl) => Some((
            decl.name.name.clone(),
            display.hir_callable_decl(decl).to_string(),
            match &decl.kind {
                CallableKind::Function => MetadataKind::Function,
                CallableKind::Operation => MetadataKind::Operation,
            },
        )),
        ItemKind::Ty(ident, udt) => Some((
            ident.name.clone(),
            display.hir_udt(udt).to_string(),
            MetadataKind::Udt,
        )),
        ItemKind::Namespace(_, _) => None,
    }?;

    let summary = parse_doc_for_summary(&item.doc)
        .replace("\r\n", " ")
        .replace('\n', " ");

    Some(Metadata {
        uid: format!("Qdk.{ns}.{name}"),
        title: match &kind {
            MetadataKind::Function => format!("{name} function"),
            MetadataKind::Operation => format!("{name} operation"),
            MetadataKind::Udt => format!("{name} user defined type"),
        },
        topic: "managed-reference".to_string(),
        kind,
        namespace: ns,
        name,
        summary,
        signature,
    })
}

/// Generates the Table of Contents file, toc.yml
fn generate_toc(map: &mut FxHashMap<Rc<str>, Vec<String>>, files: &mut Files) {
    let header = "
# This file is automatically generated.
# Please do not modify this file manually, or your changes will be lost when
# documentation is rebuilt.";
    let mut table = map
        .iter_mut()
        .map(|(namespace, lines)| {
            lines.sort_unstable();
            let items_str = lines.join("\n");
            let content =
                format!("- items:\n{items_str}\n  name: {namespace}\n  uid: Qdk.{namespace}");
            (namespace, content)
        })
        .collect::<Vec<_>>();

    table.sort_unstable_by_key(|(n, _)| *n);
    let table = table
        .into_iter()
        .map(|(_, c)| c)
        .collect::<Vec<_>>()
        .join("\n");
    let content = format!("{header}\n{table}");

    let file_name: Arc<str> = Arc::from("toc.yml");
    let file_metadata: Arc<str> = Arc::from("");
    let file_content: Arc<str> = Arc::from(content.as_str());
    files.push((file_name, file_metadata, file_content));
}
