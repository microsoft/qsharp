// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::display::increase_header_level;
use crate::{compilation::Compilation, display::CodeDisplay};
use qsc::hir::hir::{Item, ItemKind, Package, Visibility};
use rustc_hash::FxHashMap;
use std::fs;
use std::{fmt::Display, rc::Rc};

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

fn with_doc(doc: &str, code: impl Display) -> String {
    if doc.is_empty() {
        format!("# {code}\n")
    } else {
        let doc = increase_header_level(doc);
        format!("# {code}\n\n{doc}\n")
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
            // fs::write(
            //     format!("{ns_dir}/{k}.md"),
            //     v.iter()
            //         .map(|i| self.item_to_content(i))
            //         .collect::<Vec<_>>()
            //         .join("\n&nbsp;\n\n---\n\n&nbsp;\n\n"),
            // )
            // .expect("Unable to write file");
        }
    }

    fn item_to_content(&self, item: &Item) -> (String, String) {
        match &item.kind {
            ItemKind::Callable(decl) => (
                decl.name.name.to_string(),
                with_doc(&item.doc, self.display.hir_callable_decl(decl)),
            ),
            ItemKind::Ty(name, udt) => (
                name.name.to_string(),
                with_doc(&item.doc, self.display.hir_udt(udt)),
            ),
            ItemKind::Namespace(_, _) => {
                unreachable!("Namespace items should have been filtered out")
            }
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
