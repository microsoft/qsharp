// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::display::increase_header_level;
use crate::{compilation::Compilation, display::CodeDisplay};
use qsc::hir::hir::{Item, ItemKind, Package, Visibility};
use qsc::hir::PackageId;
use rustc_hash::FxHashMap;
use std::fs;
use std::{fmt::Display, rc::Rc};

pub(crate) fn generate_docs(compilation: &Compilation) {
    let display = &CodeDisplay { compilation };

    for (_, unit) in &compilation.package_store {
        GenDocs {
            package: &unit.package,
            display,
        }
        .generate_docs_for_package()
    }

    // GenDocs {
    //     package: &compilation
    //         .package_store
    //         .get(PackageId::CORE)
    //         .unwrap()
    //         .package,
    //     display: CodeDisplay { compilation },
    // }
    // .generate_docs_for_package();
}

struct GenDocs<'a> {
    package: &'a Package,
    display: &'a CodeDisplay<'a>,
}

fn with_doc(doc: &str, code: impl Display) -> String {
    if doc.is_empty() {
        code.to_string()
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
        let mut temp: FxHashMap<Rc<str>, Vec<&Item>> = FxHashMap::default();
        for (k, v) in items_to_gen {
            temp.entry(k).or_default().push(v);
        }
        for (k, v) in temp {
            fs::write(
                format!("../generated_docs/{k}.md"),
                v.iter()
                    .map(|i| self.item_to_content(i))
                    .collect::<Vec<_>>()
                    .join("\n&nbsp;\n\n---\n\n&nbsp;\n\n"),
            )
            .expect("Unable to write file");
        }
    }

    fn item_to_content(&self, item: &Item) -> String {
        match &item.kind {
            ItemKind::Callable(decl) => with_doc(&item.doc, self.display.hir_callable_decl(decl)),
            ItemKind::Ty(_, udt) => with_doc(&item.doc, self.display.hir_udt(udt)),
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
                    .expect("could not resolve parent item id");
                match &parent.kind {
                    ItemKind::Namespace(name, _) => Some(name.name.clone()),
                    _ => None,
                }
            }
            None => None,
        }
    }
}
