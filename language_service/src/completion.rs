// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::rc::Rc;

use crate::compilation::{Compilation, CompilationKind};
use crate::display::CodeDisplay;
use crate::protocol::{self, CompletionItem, CompletionItemKind, CompletionList};
use crate::qsc_utils::span_contains;
use qsc::ast::visit::{self, Visitor};
use qsc::hir::{ItemKind, Package, PackageId};

const PRELUDE: [&str; 3] = [
    "Microsoft.Quantum.Canon",
    "Microsoft.Quantum.Core",
    "Microsoft.Quantum.Intrinsic",
];

pub(crate) fn get_completions(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> CompletionList {
    let offset = compilation.source_offset_to_package_offset(source_name, offset);
    let user_ast_package = &compilation.user_unit().ast.package;

    // Determine context for the offset
    let mut context_finder = ContextFinder {
        offset,
        context: if user_ast_package.nodes.is_empty() {
            // The parser failed entirely, no context to go on
            Context::NoCompilation
        } else {
            // Starting context is top-level (i.e. outside a namespace block)
            Context::TopLevel
        },
        opens: vec![],
        start_of_namespace: None,
        current_namespace_name: None,
    };
    context_finder.visit_package(user_ast_package);

    // The PRELUDE namespaces are always implicitly opened.
    context_finder
        .opens
        .extend(PRELUDE.into_iter().map(|ns| (Rc::from(ns), None)));

    // We don't attempt to be comprehensive or accurate when determining completions,
    // since that's not really possible without more sophisticated error recovery
    // in the parser or the ability for the resolver to gather all
    // appropriate names for a scope. These are not done at the moment.

    // So the following is an attempt to get "good enough" completions, tuned
    // based on the user experience of typing out a few samples in the editor.

    let mut builder = CompletionListBuilder::new();
    match context_finder.context {
        Context::Namespace => {
            // Include "open", "operation", etc
            builder.push_item_decl_keywords();
            builder.push_attributes();

            // Typing into a callable decl sometimes breaks the
            // parser and the context appears to be a namespace block,
            // so just include everything that may be relevant
            builder.push_stmt_keywords();
            builder.push_expr_keywords();
            builder.push_types();
            builder.push_globals(
                compilation,
                &context_finder.opens,
                context_finder.start_of_namespace,
                &context_finder.current_namespace_name,
            );
        }

        Context::CallableSignature => {
            builder.push_types();
        }
        Context::Block => {
            // Pretty much anything goes in a block
            builder.push_stmt_keywords();
            builder.push_expr_keywords();
            builder.push_types();
            builder.push_globals(
                compilation,
                &context_finder.opens,
                context_finder.start_of_namespace,
                &context_finder.current_namespace_name,
            );

            // Item decl keywords last, unlike in a namespace
            builder.push_item_decl_keywords();
        }
        Context::NoCompilation | Context::TopLevel => match compilation.kind {
            CompilationKind::OpenDocument => builder.push_namespace_keyword(),
            CompilationKind::Notebook => {
                // For notebooks, the top-level allows for
                // more syntax.

                // Item declarations
                builder.push_item_decl_keywords();

                // Things that go in a block
                builder.push_stmt_keywords();
                builder.push_expr_keywords();
                builder.push_types();
                builder.push_globals(
                    compilation,
                    &context_finder.opens,
                    context_finder.start_of_namespace,
                    &context_finder.current_namespace_name,
                );

                // Namespace declarations - least likely to be used, so last
                builder.push_namespace_keyword();
            }
        },
    }

    CompletionList {
        items: builder.into_items(),
    }
}

struct CompletionListBuilder {
    current_sort_group: u32,
    items: Vec<CompletionItem>,
}

impl CompletionListBuilder {
    fn new() -> Self {
        CompletionListBuilder {
            current_sort_group: 1,
            items: Vec::new(),
        }
    }

    fn into_items(self) -> Vec<CompletionItem> {
        self.items
    }

    fn push_item_decl_keywords(&mut self) {
        static ITEM_KEYWORDS: [&str; 5] = ["operation", "open", "internal", "function", "newtype"];

        self.push_completions(
            ITEM_KEYWORDS
                .map(|key| CompletionItem::new(key.to_string(), CompletionItemKind::Keyword))
                .into_iter(),
        );
    }

    fn push_attributes(&mut self) {
        static ATTRIBUTES: [&str; 2] = ["@EntryPoint()", "@Config()"];

        self.push_completions(
            ATTRIBUTES
                .map(|key| CompletionItem::new(key.to_string(), CompletionItemKind::Property))
                .into_iter(),
        );
    }

    fn push_namespace_keyword(&mut self) {
        self.push_completions(
            [CompletionItem::new(
                "namespace".to_string(),
                CompletionItemKind::Keyword,
            )]
            .into_iter(),
        );
    }

    fn push_types(&mut self) {
        static PRIMITIVE_TYPES: [&str; 10] = [
            "Qubit", "Int", "Unit", "Result", "Bool", "BigInt", "Double", "Pauli", "Range",
            "String",
        ];
        static FUNCTOR_KEYWORDS: [&str; 3] = ["Adj", "Ctl", "is"];

        self.push_completions(
            PRIMITIVE_TYPES
                .map(|key| CompletionItem::new(key.to_string(), CompletionItemKind::Interface))
                .into_iter(),
        );

        self.push_completions(
            FUNCTOR_KEYWORDS
                .map(|key| CompletionItem::new(key.to_string(), CompletionItemKind::Keyword))
                .into_iter(),
        );
    }

    fn push_globals(
        &mut self,
        compilation: &Compilation,
        opens: &[(Rc<str>, Option<Rc<str>>)],
        start_of_namespace: Option<u32>,
        current_namespace_name: &Option<Rc<str>>,
    ) {
        let core = &compilation
            .package_store
            .get(PackageId::CORE)
            .expect("expected to find core package")
            .package;

        let mut all_except_core = compilation
            .package_store
            .iter()
            .filter(|p| p.0 != PackageId::CORE)
            .collect::<Vec<_>>();

        // Reverse to collect symbols starting at the current package backwards
        all_except_core.reverse();

        for (package_id, _) in &all_except_core {
            self.push_sorted_completions(Self::get_callables(
                compilation,
                *package_id,
                opens,
                start_of_namespace,
                current_namespace_name.clone(),
            ));
        }

        self.push_sorted_completions(Self::get_core_callables(compilation, core));

        for (_, unit) in &all_except_core {
            self.push_completions(Self::get_namespaces(&unit.package));
        }

        self.push_completions(Self::get_namespaces(core));
    }

    fn push_stmt_keywords(&mut self) {
        static STMT_KEYWORDS: [&str; 5] = ["let", "return", "use", "mutable", "borrow"];

        self.push_completions(
            STMT_KEYWORDS
                .map(|key| CompletionItem::new(key.to_string(), CompletionItemKind::Keyword))
                .into_iter(),
        );
    }

    fn push_expr_keywords(&mut self) {
        static EXPR_KEYWORDS: [&str; 11] = [
            "if", "for", "in", "within", "apply", "repeat", "until", "fixup", "set", "while",
            "fail",
        ];

        self.push_completions(
            EXPR_KEYWORDS
                .map(|key| CompletionItem::new(key.to_string(), CompletionItemKind::Keyword))
                .into_iter(),
        );
    }

    /// Each invocation of this function increments the sort group so that
    /// in the eventual completion list, the groups of items show up in the
    /// order they were added.
    /// The items are then sorted according to the input list order (not alphabetical)
    fn push_completions(&mut self, iter: impl Iterator<Item = CompletionItem>) {
        let mut current_sort_prefix = 0;

        self.items.extend(iter.map(|item| CompletionItem {
            sort_text: {
                current_sort_prefix += 1;
                Some(format!(
                    "{:02}{:02}{}",
                    self.current_sort_group, current_sort_prefix, item.label
                ))
            },
            ..item
        }));

        self.current_sort_group += 1;
    }

    /// Push a group of completions that are themselves sorted into subgroups
    fn push_sorted_completions(&mut self, iter: impl Iterator<Item = (CompletionItem, u32)>) {
        self.items
            .extend(iter.map(|(item, item_sort_group)| CompletionItem {
                sort_text: Some(format!(
                    "{:02}{:02}{}",
                    self.current_sort_group, item_sort_group, item.label
                )),
                ..item
            }));

        self.current_sort_group += 1;
    }

    fn get_callables<'a>(
        compilation: &'a Compilation,
        package_id: PackageId,
        opens: &'a [(Rc<str>, Option<Rc<str>>)],
        start_of_namespace: Option<u32>,
        current_namespace_name: Option<Rc<str>>,
    ) -> impl Iterator<Item = (CompletionItem, u32)> + 'a {
        let package = &compilation
            .package_store
            .get(package_id)
            .expect("package id should exist")
            .package;
        let display = CodeDisplay { compilation };

        package.items.values().filter_map(move |i| {
            // We only want items whose parents are namespaces
            if let Some(item_id) = i.parent {
                if let Some(parent) = package.items.get(item_id) {
                    if let ItemKind::Namespace(namespace, _) = &parent.kind {
                        return match &i.kind {
                            ItemKind::Callable(callable_decl) => {
                                let name = callable_decl.name.name.as_ref();
                                let detail = Some(
                                    display
                                        .hir_callable_decl(package_id, callable_decl)
                                        .to_string(),
                                );
                                // Everything that starts with a __ goes last in the list
                                let sort_group = u32::from(name.starts_with("__"));
                                let mut additional_edits = vec![];
                                let mut qualification: Option<Rc<str>> = None;
                                match &current_namespace_name {
                                    Some(curr_ns) if *curr_ns == namespace.name => {}
                                    _ => {
                                        // open is an option of option of Rc<str>
                                        // the first option tells if it found an open with the namespace name
                                        // the second, nested option tells if that open has an alias
                                        let open = opens.iter().find_map(|(name, alias)| {
                                            if *name == namespace.name {
                                                Some(alias)
                                            } else {
                                                None
                                            }
                                        });
                                        qualification = match open {
                                            Some(alias) => alias.as_ref().cloned(),
                                            None => match start_of_namespace {
                                                Some(start) => {
                                                    additional_edits.push((
                                                        protocol::Span { start, end: start },
                                                        format!(
                                                            "open {};\n    ",
                                                            namespace.name.clone()
                                                        ),
                                                    ));
                                                    None
                                                }
                                                None => Some(namespace.name.clone()),
                                            },
                                        }
                                    }
                                }

                                let additional_text_edits = if additional_edits.is_empty() {
                                    None
                                } else {
                                    Some(additional_edits)
                                };

                                let label = if let Some(qualification) = qualification {
                                    format!("{qualification}.{name}")
                                } else {
                                    name.to_owned()
                                };
                                Some((
                                    CompletionItem {
                                        label,
                                        kind: CompletionItemKind::Function,
                                        sort_text: None, // This will get filled in during `push_sorted_completions`
                                        detail,
                                        additional_text_edits,
                                    },
                                    sort_group,
                                ))
                            }
                            _ => None,
                        };
                    }
                }
            }
            None
        })
    }

    fn get_core_callables<'a>(
        compilation: &'a Compilation,
        package: &'a Package,
    ) -> impl Iterator<Item = (CompletionItem, u32)> + 'a {
        let display = CodeDisplay { compilation };

        package.items.values().filter_map(move |i| match &i.kind {
            ItemKind::Callable(callable_decl) => {
                let name = callable_decl.name.name.as_ref();
                let detail = Some(
                    display
                        .hir_callable_decl(PackageId::CORE, callable_decl)
                        .to_string(),
                );
                // Everything that starts with a __ goes last in the list
                let sort_group = u32::from(name.starts_with("__"));
                Some((
                    CompletionItem {
                        label: name.to_string(),
                        kind: CompletionItemKind::Function,
                        sort_text: None, // This will get filled in during `push_sorted_completions`
                        detail,
                        additional_text_edits: None,
                    },
                    sort_group,
                ))
            }
            _ => None,
        })
    }

    fn get_namespaces(package: &'_ Package) -> impl Iterator<Item = CompletionItem> + '_ {
        package.items.values().filter_map(|i| match &i.kind {
            ItemKind::Namespace(namespace, _) => Some(CompletionItem::new(
                namespace.name.to_string(),
                CompletionItemKind::Module,
            )),
            _ => None,
        })
    }
}

struct ContextFinder {
    offset: u32,
    context: Context,
    opens: Vec<(Rc<str>, Option<Rc<str>>)>,
    start_of_namespace: Option<u32>,
    current_namespace_name: Option<Rc<str>>,
}

#[derive(Debug, PartialEq)]
enum Context {
    NoCompilation,
    TopLevel,
    Namespace,
    CallableSignature,
    Block,
}

impl Visitor<'_> for ContextFinder {
    fn visit_namespace(&mut self, namespace: &'_ qsc::ast::Namespace) {
        if span_contains(namespace.span, self.offset) {
            self.current_namespace_name = Some(namespace.name.name.clone());
            self.context = Context::Namespace;
            self.opens = vec![];
            self.start_of_namespace = None;
            visit::walk_namespace(self, namespace);
        }
    }

    fn visit_item(&mut self, item: &'_ qsc::ast::Item) {
        if self.start_of_namespace.is_none() {
            self.start_of_namespace = Some(item.span.lo);
        }

        if let qsc::ast::ItemKind::Open(name, alias) = &*item.kind {
            self.opens.push((
                name.name.clone(),
                alias.as_ref().map(|alias| alias.name.clone()),
            ));
        }

        if span_contains(item.span, self.offset) {
            visit::walk_item(self, item);
        }
    }

    fn visit_callable_decl(&mut self, decl: &'_ qsc::ast::CallableDecl) {
        if span_contains(decl.span, self.offset) {
            // This span covers the body too, but the
            // context will get overwritten by visit_block
            // if the offset is inside the actual body
            self.context = Context::CallableSignature;
            visit::walk_callable_decl(self, decl);
        }
    }

    fn visit_block(&mut self, block: &'_ qsc::ast::Block) {
        if span_contains(block.span, self.offset) {
            self.context = Context::Block;
        }
    }
}
