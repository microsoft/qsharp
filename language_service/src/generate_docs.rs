// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::display::increase_header_level;
use crate::{compilation::Compilation, display::CodeDisplay};
use qsc::hir::hir::{Item, ItemKind, Package, Visibility};
use qsc::hir::ty::Udt;
use qsc::hir::{CallableDecl, Ident};
use rustc_hash::FxHashMap;
use std::fs;
use std::rc::Rc;

// Warning: this path gets deleted on each run. Use carefully!
const GENERATED_DOCS_PATH: &str = "../generated_docs";

pub(crate) fn generate_docs(compilation: &Compilation) {
    let display = &CodeDisplay { compilation };

    delete_existing_docs();
    fs::create_dir(GENERATED_DOCS_PATH).expect("Unable to create directory for generated docs");

    for (_, unit) in &compilation.package_store {
        GenDocs {
            package: &unit.package,
            display,
        }
        .generate_docs_for_package();
    }
}

struct GenDocs<'a> {
    package: &'a Package,
    display: &'a CodeDisplay<'a>,
}

fn delete_existing_docs() {
    // Checks if the generated docs exist
    if fs::metadata(GENERATED_DOCS_PATH).is_ok() {
        // Warning, use this carefully!
        fs::remove_dir_all(GENERATED_DOCS_PATH).expect("Unable to remove existing docs");
    }
}

impl<'a> GenDocs<'a> {
    fn generate_docs_for_package(self) {
        let package = self.package;

        let items_to_gen = package
            .items
            .iter()
            .filter_map(|(_, i)| self.filter_items(i));
        let mut namespace_to_content_map: FxHashMap<Rc<str>, Vec<&Item>> = FxHashMap::default();
        for (k, v) in items_to_gen {
            namespace_to_content_map.entry(k).or_default().push(v);
        }
        for (k, v) in namespace_to_content_map {
            let ns_dir = format!("{GENERATED_DOCS_PATH}/{k}");
            if fs::metadata(&ns_dir).is_err() {
                fs::create_dir(&ns_dir).expect("Unable to create directory for namespace");
            }
            for (name, contents) in v.iter().map(|i| self.item_to_content(i)) {
                fs::write(format!("{ns_dir}/{name}.md"), contents).expect("Unable to write file");
            }
        }
    }

    fn item_to_content(&self, item: &Item) -> (String, String) {
        match &item.kind {
            ItemKind::Callable(decl) => (
                decl.name.name.to_string(),
                self.callable_to_content(decl, &item.doc),
            ),
            ItemKind::Ty(name, udt) => (
                name.name.to_string(),
                self.udt_to_content(name, udt, &item.doc),
            ),
            ItemKind::Namespace(_, _) => {
                unreachable!("Namespace items should have been filtered out")
            }
        }
    }

    fn callable_to_content(&self, decl: &CallableDecl, doc: &str) -> String {
        if doc.is_empty() {
            format!(
                "# {} {}\n\n`{}`\n",
                decl.name.name,
                decl.kind,
                self.display.hir_callable_decl(decl)
            )
        } else {
            let doc = increase_header_level(doc);
            format!(
                "# {} {}\n\n`{}`\n\n{}\n",
                decl.name.name,
                decl.kind,
                self.display.hir_callable_decl(decl),
                doc
            )
        }
    }

    fn udt_to_content(&self, name: &Ident, udt: &Udt, doc: &str) -> String {
        if doc.is_empty() {
            format!(
                "# {} User-Defined Type\n\n`{}`\n",
                name.name,
                self.display.hir_udt(udt)
            )
        } else {
            let doc = increase_header_level(doc);
            format!(
                "# {} User-Defined Type\n\n`{}`\n\n{}\n",
                name.name,
                self.display.hir_udt(udt),
                doc
            )
        }
    }

    fn filter_items(&'a self, item: &'a Item) -> Option<(Rc<str>, &'a Item)> {
        if item.visibility == Visibility::Internal {
            return None;
        }

        match item.kind {
            ItemKind::Callable(_) | ItemKind::Ty(_, _) => {
                self.get_namespace(item).map(|ns| (ns, item))
            }
            ItemKind::Namespace(_, _) => None,
        }
    }

    fn get_namespace(&self, item: &Item) -> Option<Rc<str>> {
        match item.parent {
            Some(local_id) => {
                let parent = self
                    .package
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
}
