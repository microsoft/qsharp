// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::qsc_utils::{map_offset, span_contains, Compilation};
use qsc::ast::visit::{self, Visitor};
use qsc::hir::{ItemKind, Package, PackageId};

#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub enum CompletionItemKind {
    // It would have been nice to match these enum values to the ones used by
    // VS Code and Monaco, but unfortunately those two disagree on the values.
    // So we define our own unique enum here to reduce confusion.
    Function,
    Interface,
    Keyword,
    Module,
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
    pub sort_text: Option<String>,
    pub detail: Option<String>,
}

pub(crate) fn get_completions(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> CompletionList {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.unit.sources, source_name, offset);

    // Determine context for the offset
    let mut context_finder = ContextFinder {
        offset,
        context: if compilation.unit.ast.package.namespaces.as_ref().is_empty() {
            // The parser failed entirely, no context to go on
            Context::NoCompilation
        } else {
            // Starting context is top-level (i.e. outside a namespace block)
            Context::TopLevel
        },
    };
    context_finder.visit_package(&compilation.unit.ast.package);

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

            // Typing into a callable decl sometimes breaks the
            // parser and the context appears to be a namespace block,
            // so just include everything that may be relevant
            builder.push_stmt_keywords();
            builder.push_expr_keywords();
            builder.push_types();
            builder.push_globals(compilation);
        }

        Context::CallableSignature => {
            builder.push_types();
        }
        Context::Block => {
            // Pretty much anything goes in a block
            builder.push_stmt_keywords();
            builder.push_expr_keywords();
            builder.push_types();
            builder.push_globals(compilation);

            // Item decl keywords last, unlike in a namespace
            builder.push_item_decl_keywords();
        }
        Context::NoCompilation | Context::TopLevel => {
            builder.push_namespace_keyword();
        }
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

        self.push_completions(ITEM_KEYWORDS.into_iter(), CompletionItemKind::Keyword);
    }

    fn push_namespace_keyword(&mut self) {
        self.push_completions(["namespace"].into_iter(), CompletionItemKind::Keyword);
    }

    fn push_types(&mut self) {
        static PRIMITIVE_TYPES: [&str; 10] = [
            "Qubit", "Int", "Unit", "Result", "Bool", "BigInt", "Double", "Pauli", "Range",
            "String",
        ];
        static FUNCTOR_KEYWORDS: [&str; 3] = ["Adj", "Ctl", "is"];

        self.push_completions(PRIMITIVE_TYPES.into_iter(), CompletionItemKind::Interface);
        self.push_completions(FUNCTOR_KEYWORDS.into_iter(), CompletionItemKind::Keyword);
    }

    fn push_globals(&mut self, compilation: &Compilation) {
        let current = &compilation.unit.package;
        let std = &compilation
            .package_store
            .get(compilation.std_package_id)
            .expect("expected to find std package")
            .package;
        let core = &compilation
            .package_store
            .get(PackageId::CORE)
            .expect("expected to find core package")
            .package;

        self.push_sorted_completions(Self::get_callables(current), CompletionItemKind::Function);
        self.push_sorted_completions(Self::get_callables(std), CompletionItemKind::Function);
        self.push_sorted_completions(Self::get_callables(core), CompletionItemKind::Function);
        self.push_completions(Self::get_namespaces(current), CompletionItemKind::Module);
        self.push_completions(Self::get_namespaces(std), CompletionItemKind::Module);
        self.push_completions(Self::get_namespaces(core), CompletionItemKind::Module);
    }

    fn push_stmt_keywords(&mut self) {
        static STMT_KEYWORDS: [&str; 5] = ["let", "return", "use", "mutable", "borrow"];

        self.push_completions(STMT_KEYWORDS.into_iter(), CompletionItemKind::Keyword);
    }

    fn push_expr_keywords(&mut self) {
        static EXPR_KEYWORDS: [&str; 11] = [
            "if", "for", "in", "within", "apply", "repeat", "until", "fixup", "set", "while",
            "fail",
        ];

        self.push_completions(EXPR_KEYWORDS.into_iter(), CompletionItemKind::Keyword);
    }

    /// Each invocation of this function increments the sort group so that
    /// in the eventual completion list, the groups of items show up in the
    /// order they were added.
    /// The items are then sorted according to the input list order (not alphabetical)
    fn push_completions<'a>(
        &mut self,
        iter: impl Iterator<Item = &'a str>,
        kind: CompletionItemKind,
    ) {
        let mut current_sort_prefix = 0;

        self.items.extend(iter.map(|name| CompletionItem {
            label: name.to_string(),
            kind,
            sort_text: {
                current_sort_prefix += 1;
                Some(format!(
                    "{:02}{:02}{}",
                    self.current_sort_group, current_sort_prefix, name
                ))
            },
            detail: Some("fake detail at server level".to_owned()),
        }));

        self.current_sort_group += 1;
    }

    /// Push a group of completions that are themselves sorted into subgroups
    fn push_sorted_completions<'a>(
        &mut self,
        iter: impl Iterator<Item = (&'a str, u32)>,
        kind: CompletionItemKind,
    ) {
        self.items
            .extend(iter.map(|(name, item_sort_group)| CompletionItem {
                label: name.to_string(),
                kind,
                sort_text: Some(format!(
                    "{:02}{:02}{}",
                    self.current_sort_group, item_sort_group, name
                )),
                detail: Some("fake detail at server level".to_owned()),
            }));

        self.current_sort_group += 1;
    }

    fn get_callables(package: &Package) -> impl Iterator<Item = (&str, u32)> {
        package.items.values().filter_map(|i| match &i.kind {
            ItemKind::Callable(callable_decl) => Some({
                let name = callable_decl.name.name.as_ref();
                // Everything that starts with a __ goes last in the list
                let sort_group = u32::from(name.starts_with("__"));
                (name, sort_group)
            }),
            _ => None,
        })
    }

    fn get_namespaces(package: &Package) -> impl Iterator<Item = &str> {
        package.items.values().filter_map(|i| match &i.kind {
            ItemKind::Namespace(namespace, _) => Some(namespace.name.as_ref()),
            _ => None,
        })
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
    CallableSignature,
    Block,
}

impl Visitor<'_> for ContextFinder {
    fn visit_namespace(&mut self, namespace: &'_ qsc::ast::Namespace) {
        if span_contains(namespace.span, self.offset) {
            self.context = Context::Namespace;
        }

        visit::walk_namespace(self, namespace);
    }

    fn visit_callable_decl(&mut self, decl: &'_ qsc::ast::CallableDecl) {
        if span_contains(decl.span, self.offset) {
            // This span covers the body too, but the
            // context will get overwritten by visit_block
            // if the offset is inside the actual body
            self.context = Context::CallableSignature;
        }

        visit::walk_callable_decl(self, decl);
    }

    fn visit_block(&mut self, block: &'_ qsc::ast::Block) {
        if span_contains(block.span, self.offset) {
            self.context = Context::Block;
        }
    }
}
