// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{CompletionItem, CompletionList};
use enum_iterator::all;
use qsc::{compile, hir::ItemKind, hir::PackageId, PackageStore, SourceMap};
use qsc_frontend::{compile::CompileUnit, parse::Keyword};
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

pub fn get_completions(code: &str, offset: u32) -> Result<JsValue, JsValue> {
    // TODO: I don't like thread locals
    thread_local! {
        static STORE_STD: (PackageStore, PackageId) = {
            let mut store = PackageStore::new(compile::core());
            let std = store.insert(compile::std(&store));
            (store, std)
        };
    }

    thread_local! {
        static KEYWORDS: Vec<CompletionItem> = {
            all::<Keyword>().map(|k| CompletionItem {
                label: k.to_string(),
                kind: CompletionKind::Keyword as i32,
            }).collect::<Vec<_>>()
        }
    }

    STORE_STD.with(|(store, std)| {
        let sources = SourceMap::new([("code".into(), code.into())], None);
        let (compile_unit, errors) = compile::compile(store, &[*std], sources);
        let no_compilation = compile_unit.package.items.values().next().is_none();

        // Determine context
        let mut context_builder = ContextFinder {
            offset,
            context: if no_compilation {
                Context::NoCompilation
            } else {
                Context::TopLevel
            },
        };
        context_builder.visit_package(&compile_unit.package);
        let context = context_builder.context;

        // Collect namespaces
        let mut namespace_collector = NamespaceCollector {
            namespaces: HashSet::new(),
        };
        namespace_collector.visit_package(&compile_unit.package);
        namespace_collector.visit_package(
            &store
                .get(*std)
                .expect("expected to find std package")
                .package,
        );

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
        let mut current_callables = callable_names_from_compile_unit(&compile_unit);

        // All callables from std package
        let mut std_callables = callable_names_from_compile_unit(
            store.get(*std).expect("expected to find std package"),
        );

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
    })
}

fn span_contains(span: qsc_data_structures::span::Span, offset: u32) -> bool {
    offset >= span.lo && offset < span.hi
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

fn callable_names_from_compile_unit(compile_unit: &CompileUnit) -> Vec<CompletionItem> {
    compile_unit
        .package
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
