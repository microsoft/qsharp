// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::qsc_utils::{map_offset, span_contains, Compilation};
use qsc::hir::{
    visit::{walk_item, Visitor},
    ItemKind, {Block, Item, Package},
};
use std::collections::HashSet;

// It would have been nice to match these enum values to the ones used by
// VS Code and Monaco, but unfortunately those two disagree on the values.
// So we define our own unique enum here to reduce confusion.
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub enum CompletionItemKind {
    Function,
    Module,
    Keyword,
    Issue,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct CompletionList {
    pub items: Vec<CompletionItem>,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
}

pub(crate) fn get_completions(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> CompletionList {
    todo!();
    // // Map the file offset into a SourceMap offset
    // let offset = map_offset(&compilation.source_map, source_name, offset);
    // let package = &compilation.package;
    // let std_package = &compilation
    //     .package_store
    //     .get(compilation.std_package_id)
    //     .expect("expected to find std package")
    //     .package;

    // // Collect namespaces
    // let mut namespace_collector = NamespaceCollector {
    //     namespaces: HashSet::new(),
    // };
    // namespace_collector.visit_package(package);
    // namespace_collector.visit_package(std_package);

    // // All namespaces
    // let mut namespaces = namespace_collector
    //     .namespaces
    //     .drain()
    //     .map(|ns| CompletionItem {
    //         label: ns,
    //         kind: CompletionItemKind::Module,
    //     })
    //     .collect::<Vec<_>>();

    // // Determine context for the offset
    // let mut context_builder = ContextFinder {
    //     offset,
    //     context: if compilation.package.items.values().next().is_none() {
    //         Context::NoCompilation
    //     } else {
    //         Context::TopLevel
    //     },
    // };
    // context_builder.visit_package(package);
    // let context = context_builder.context;

    // let mut items = Vec::new();
    // match context {
    //     Context::Namespace => {
    //         items.push(CompletionItem {
    //             label: "open".to_string(),
    //             kind: CompletionItemKind::Keyword,
    //         });
    //         items.append(&mut namespaces);
    //     }
    //     Context::Block | Context::NoCompilation => {
    //         // Add everything we know of.
    //         // All callables from std package
    //         items.append(&mut callable_names_from_package(std_package));
    //         // Callables from the current document
    //         items.append(&mut callable_names_from_package(package));
    //         items.append(&mut namespaces);
    //     }
    //     Context::TopLevel | Context::NotSignificant => items.push(CompletionItem {
    //         label: "namespace".to_string(),
    //         kind: CompletionItemKind::Keyword,
    //     }),
    // }
    // CompletionList { items }
}

struct NamespaceCollector {
    namespaces: HashSet<String>,
}

impl Visitor<'_> for NamespaceCollector {
    fn visit_item(&mut self, item: &Item) {
        if let ItemKind::Namespace(ident, _) = &item.kind {
            // Collect namespaces
            self.namespaces.insert(ident.name.to_string());
        }
        walk_item(self, item);
    }
}

struct ContextFinder {
    offset: u32,
    context: Context,
}

#[derive(Debug, PartialEq)]
enum Context {
    NoCompilation,
    TopLevel,
    Namespace,
    Block,
    NotSignificant,
}

impl Visitor<'_> for ContextFinder {
    fn visit_item(&mut self, item: &Item) {
        if span_contains(item.span, self.offset) {
            self.context = match &item.kind {
                ItemKind::Namespace(..) => Context::Namespace,
                _ => Context::NotSignificant,
            }
        }

        walk_item(self, item);
    }

    fn visit_block(&mut self, block: &Block) {
        if span_contains(block.span, self.offset) {
            self.context = Context::Block;
        }
    }
}

fn callable_names_from_package(package: &Package) -> Vec<CompletionItem> {
    package
        .items
        .values()
        .filter_map(|i| match &i.kind {
            ItemKind::Callable(callable_decl) => Some(CompletionItem {
                label: callable_decl.name.name.to_string(),
                kind: CompletionItemKind::Function,
            }),
            _ => None,
        })
        .collect::<Vec<_>>()
}
