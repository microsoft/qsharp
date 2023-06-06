// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::qsc_utils::{span_contains, Compilation};
use enum_iterator::all;
use qsc::hir::ItemKind;
use qsc_frontend::parse::Keyword;
use qsc_hir::hir::Package;
use qsc_hir::{
    hir::{Block, Item},
    visit::Visitor,
};
use std::collections::HashSet;

// It would have been nice to match these enum values to the ones used by
// VS Code and Monaco, but unfortunately those two disagree in the exact values.
// So we define our own unique enum here to reduce confusion.
#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
    let compile_unit = &compilation.compile_unit;
    // Map the file offset into a SourceMap offset
    let offset = compile_unit.sources.map_offset(source_name, offset);
    let no_compilation = compile_unit.package.items.values().next().is_none();
    let package = &compile_unit.package;
    let std_package = &compilation
        .package_store
        .get(compilation.std_package_id)
        .expect("expected to find std package")
        .package;

    // Determine context
    let mut context_builder = ContextFinder {
        offset,
        context: if no_compilation {
            Context::NoCompilation
        } else {
            Context::TopLevel
        },
    };
    context_builder.visit_package(package);
    let context = context_builder.context;

    // Collect namespaces
    let mut namespace_collector = NamespaceCollector {
        namespaces: HashSet::new(),
    };
    namespace_collector.visit_package(package);
    namespace_collector.visit_package(std_package);

    let mut res = CompletionList { items: Vec::new() };

    // Callables from the current document
    let mut current_callables = callable_names_from_package(package);

    // All callables from std package
    let mut std_callables = callable_names_from_package(std_package);

    // All keywords
    let mut keywords = all::<Keyword>()
        .map(|k| CompletionItem {
            label: k.to_string(),
            kind: CompletionItemKind::Keyword,
        })
        .collect::<Vec<_>>();

    // All namespaces
    let mut namespaces = namespace_collector
        .namespaces
        .drain()
        .map(|ns| CompletionItem {
            label: ns,
            kind: CompletionItemKind::Module,
        })
        .collect::<Vec<_>>();

    match context {
        Context::Namespace => {
            res.items.push(CompletionItem {
                label: "open".to_string(),
                kind: CompletionItemKind::Keyword,
            });
            res.items.append(&mut namespaces);
        }
        Context::Block | Context::NoCompilation => {
            // Add everything we know of.
            res.items.append(&mut keywords);
            res.items.append(&mut std_callables);
            res.items.append(&mut current_callables);
            res.items.append(&mut namespaces);
        }
        Context::TopLevel | Context::NotSignificant => res.items.push(CompletionItem {
            label: "namespace".to_string(),
            kind: CompletionItemKind::Keyword,
        }),
    }
    res
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
        qsc_hir::visit::walk_item(self, item);
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

        qsc_hir::visit::walk_item(self, item);
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
