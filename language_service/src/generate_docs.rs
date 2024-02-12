// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::display::{increase_header_level, parse_doc_for_summary};
use crate::{compilation::Compilation, display::CodeDisplay};
use chrono;
use qsc::hir::hir::{Item, ItemKind, Package, Visibility};
use rustc_hash::FxHashMap;
use std::fmt::{Display, Formatter, Result};
use std::fs;
use std::rc::Rc;

// Warning: this path gets deleted on each run. Use carefully!
const GENERATED_DOCS_PATH: &str = "../generated_docs";

pub(crate) fn generate_docs(compilation: &Compilation) {
    let display = &CodeDisplay { compilation };

    delete_existing_docs();
    fs::create_dir(GENERATED_DOCS_PATH).expect("Unable to create directory for generated docs");

    let mut toc: FxHashMap<Rc<str>, Vec<String>> = FxHashMap::default();
    for (_, unit) in &compilation.package_store {
        let package = &unit.package;
        for (_, item) in &package.items {
            if let Some((ns, line)) = generate_doc_for_item(package, item, display) {
                toc.entry(ns).or_default().push(line);
            }
        }
    }

    generate_toc(&toc);
}

fn delete_existing_docs() {
    // Checks if the generated docs exist
    if fs::metadata(GENERATED_DOCS_PATH).is_ok() {
        // Warning, use this carefully!
        fs::remove_dir_all(GENERATED_DOCS_PATH).expect("Unable to remove existing docs");
    }
}

fn generate_doc_for_item<'a>(
    package: &'a Package,
    item: &'a Item,
    display: &'a CodeDisplay,
) -> Option<(Rc<str>, String)> {
    // Filter items
    if item.visibility == Visibility::Internal || matches!(item.kind, ItemKind::Namespace(_, _)) {
        return None;
    }

    // Get namespace for item
    let ns = get_namespace(package, item)?;

    // Create ns folder, if it doesn't exist
    let ns_dir = format!("{GENERATED_DOCS_PATH}/{ns}");
    if fs::metadata(&ns_dir).is_err() {
        fs::create_dir(&ns_dir).expect("Unable to create directory for namespace");
    }

    // Get Date
    let date = chrono::Utc::now()
        .date_naive()
        .format("%m/%d/%Y")
        .to_string();

    // Print file
    let (title, content) = generate_file(&ns, item, display, date)?;
    fs::write(format!("{ns_dir}/{title}.md"), content).expect("Unable to write file");

    // Create toc line
    let line = format!("  - {{name: {title}, uid: {ns}.{title}}}");

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
                    if name.name.starts_with("QIR") {
                        None // We ignore "QIR" namespaces
                    } else {
                        Some(name.name.clone())
                    }
                }
                _ => None,
            }
        }
        None => None,
    }
}

fn generate_file(
    ns: &Rc<str>,
    item: &Item,
    display: &CodeDisplay,
    date: String,
) -> Option<(Rc<str>, String)> {
    let metadata = get_metadata(ns.clone(), item, display, date)?;

    let doc = increase_header_level(&item.doc);
    let title = &metadata.title;
    let sig = &metadata.signature;

    let content = format!(
        "{metadata}

# {title}

Namespace: [{ns}](xref:{ns})

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

    Some((metadata.name.clone(), content))
}

struct Metadata {
    uid: String,
    title: String,
    date: String,
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
            MetadataKind::Operation => "opeartion",
            MetadataKind::Udt => "udt",
        };
        write!(
            f,
            "---
uid {}
title: {}
ms.date: {}
ms.topic: {}
qsharp.kind: {}
qsharp.namespace: {}
qsharp.name: {}
qsharp.summary: {}
---",
            self.uid,
            self.title,
            self.date,
            self.topic,
            kind,
            self.namespace,
            self.name,
            self.summary
        )
    }
}

enum MetadataKind {
    Function,
    Operation,
    Udt,
}

fn get_metadata(ns: Rc<str>, item: &Item, display: &CodeDisplay, date: String) -> Option<Metadata> {
    let (name, signature, kind) = match &item.kind {
        ItemKind::Callable(decl) => Some((
            decl.name.name.clone(),
            display.hir_callable_decl(decl).to_string(),
            match &decl.kind {
                qsc::hir::CallableKind::Function => MetadataKind::Function,
                qsc::hir::CallableKind::Operation => MetadataKind::Operation,
            },
        )),
        ItemKind::Ty(ident, udt) => Some((
            ident.name.clone(),
            display.hir_udt(udt).to_string(),
            MetadataKind::Udt,
        )),
        ItemKind::Namespace(_, _) => None,
    }?;

    Some(Metadata {
        uid: format!("{ns}.{name}"),
        title: match &kind {
            MetadataKind::Function => format!("{name} function"),
            MetadataKind::Operation => format!("{name} operation"),
            MetadataKind::Udt => format!("{name} user defined type"),
        },
        date,
        topic: "managed-reference".to_string(),
        kind,
        namespace: ns,
        name,
        summary: parse_doc_for_summary(&item.doc),
        signature,
    })
}

fn generate_toc(map: &FxHashMap<Rc<str>, Vec<String>>) {
    let header = "
# This file is automatically generated.
# Please do not modify this file manually, or your changes may be lost when
# documentation is rebuilt.";
    let table = map
        .iter()
        .map(|(namespace, lines)| {
            let items_str = lines.join("\n");
            format!("- items:\n{items_str}\n  name: {namespace}\n  uid: {namespace}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let contents = format!("{header}\n{table}");

    fs::write(format!("{GENERATED_DOCS_PATH}/toc.yml"), contents)
        .expect("Unable to create table of contents file");
}
