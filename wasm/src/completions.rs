// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::ls_utils::{get_compilation, span_contains};
use crate::{CompletionItem, CompletionList};
use enum_iterator::all;
use qsc::hir::ItemKind;
use qsc_frontend::parse::Keyword;
use qsc_hir::hir::Package;
use qsc_hir::{
    hir::{Block, Item},
    visit::Visitor,
};
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

// These definitions match the values expected by VS Code and Monaco.
enum CompletionKind {
    Function = 1,
    Module = 8,
    Keyword = 13,
    Issue = 26,
}

pub(crate) fn get_completions(
    source_path: &str,
    code: &str,
    offset: u32,
) -> Result<JsValue, JsValue> {
    let (std_package, package, _, no_compilation, errors) = get_compilation(source_path, code);

    // TODO: I don't like thread locals
    thread_local! {
        static KEYWORDS: Vec<CompletionItem> = {
            all::<Keyword>().map(|k| CompletionItem {
                label: k.to_string(),
                kind: CompletionKind::Keyword as i32,
            }).collect::<Vec<_>>()
        }
    }

    // Determine context
    let mut context_builder = ContextFinder {
        offset,
        context: if no_compilation {
            Context::NoCompilation
        } else {
            Context::TopLevel
        },
    };
    context_builder.visit_package(&package);
    let context = context_builder.context;

    // Collect namespaces
    let mut namespace_collector = NamespaceCollector {
        namespaces: HashSet::new(),
    };
    namespace_collector.visit_package(&package);
    namespace_collector.visit_package(&std_package);

    // Add debug items for convenience
    let mut debug_items = Vec::new();
    debug_items.push(CompletionItem {
        label: format!("__DEBUG__ context: {:?}", context),
        kind: CompletionKind::Issue as i32,
    });
    debug_items.push(CompletionItem {
        label: format!("__DEBUG__ errors: {:?}", errors),
        kind: CompletionKind::Issue as i32,
    });

    let mut res = CompletionList { items: Vec::new() };

    // Callables from the current code
    let mut current_callables = callable_names_from_package(&package);

    // All callables from std package
    let mut std_callables = callable_names_from_package(&std_package);

    // All keywords
    let mut keywords = KEYWORDS.with(|kws| kws.to_vec());

    // All namespaces
    let mut namespaces = namespace_collector
        .namespaces
        .drain()
        .map(|ns| CompletionItem {
            label: ns,
            kind: CompletionKind::Module as i32,
        })
        .collect::<Vec<_>>();

    match context {
        Context::Namespace => {
            res.items.push(CompletionItem {
                label: "open".to_string(),
                kind: CompletionKind::Keyword as i32,
            });
            res.items.append(&mut namespaces);
        }
        Context::Block => {
            res.items.append(&mut keywords);
            res.items.append(&mut std_callables);
            res.items.append(&mut current_callables);
            res.items.append(&mut namespaces);
        }
        Context::TopLevel | Context::NotSignificant => res.items.push(CompletionItem {
            label: "namespace".to_string(),
            kind: CompletionKind::Keyword as i32,
        }),
        Context::NoCompilation => {
            // Add everything we know of.
            // Of course, what's the point in determining context
            // if we're going to do this in most cases?
            res.items.append(&mut keywords);
            res.items.append(&mut std_callables);
            res.items.append(&mut current_callables);
            res.items.append(&mut namespaces);

            debug_items.push(CompletionItem {
                label: "__DEBUG__ NO COMPILATION".to_string(),
                kind: CompletionKind::Issue as i32,
            });
        }
    }

    res.items.append(&mut debug_items);
    Ok(serde_wasm_bindgen::to_value(&res)?)
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
        qsc_hir::visit::walk_item(self, item)
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

        qsc_hir::visit::walk_item(self, item)
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
                kind: CompletionKind::Function as i32,
            }),
            _ => None,
        })
        .collect::<Vec<_>>()
}
