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
    // since that's not really possible without access to the AST, more sophisticated
    // error recovery in the parser, and the ability for the resolver to gather all
    // appropriate names for a scope. These are not done at the moment.

    // So the following is an attempt to get "good enough" completions, tuned
    // based on the user experience of typing out a few samples in the editor.

    let mut builder = CompletionListBuilder::new();
    match context_finder.context {
        Context::Namespace => {
            // We're in a namespace block, so include open, operation, etc
            builder.push_item_decl_keywords();

            // Typing the signature to a callable decl sometimes breaks the
            // parser and the context appears to be a namespace block,
            // so just include types which would be relevant in that context too
            builder.push_types();
        }
        Context::CallableSignature => {
            builder.push_types();
        }
        Context::Block => {
            // Anything goes in a block
            builder.push_all(compilation);
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
    items: Vec<CompletionItem>,
}

impl CompletionListBuilder {
    fn new() -> Self {
        CompletionListBuilder { items: Vec::new() }
    }

    fn into_items(self) -> Vec<CompletionItem> {
        self.items
    }

    fn push_item_decl_keywords(&mut self) {
        static ITEM_KEYWORDS: [&str; 4] = ["function", "newtype", "open", "operation"];

        self.push_completions(ITEM_KEYWORDS.into_iter(), CompletionItemKind::Keyword);
    }

    fn push_namespace_keyword(&mut self) {
        self.push_completions(["namespace"].into_iter(), CompletionItemKind::Keyword);
    }

    fn push_types(&mut self) {
        static PRIMITIVE_TYPES: [&str; 10] = [
            "BigInt", "Bool", "Double", "Int", "Pauli", "Qubit", "Range", "Result", "String",
            "Unit",
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

        self.push_completions(Self::get_callables(current), CompletionItemKind::Function);
        self.push_completions(Self::get_callables(std), CompletionItemKind::Function);
        self.push_completions(Self::get_callables(core), CompletionItemKind::Function);
        self.push_completions(Self::get_namespaces(current), CompletionItemKind::Module);
        self.push_completions(Self::get_namespaces(std), CompletionItemKind::Module);
        self.push_completions(Self::get_namespaces(core), CompletionItemKind::Module);
    }

    fn push_all(&mut self, compilation: &Compilation) {
        static STMT_KEYWORDS: [&str; 5] = ["let", "mutable", "use", "borrow", "return"];
        static EXPR_KEYWORDS: [&str; 11] = [
            "for", "in", "if", "repeat", "until", "fixup", "set", "while", "within", "apply",
            "fail",
        ];
        self.push_types();
        self.push_completions(STMT_KEYWORDS.into_iter(), CompletionItemKind::Keyword);
        self.push_completions(EXPR_KEYWORDS.into_iter(), CompletionItemKind::Keyword);
        self.push_globals(compilation);
        self.push_item_decl_keywords();
    }

    fn push_completions<'a, I>(&mut self, iter: I, kind: CompletionItemKind)
    where
        I: Iterator<Item = &'a str>,
    {
        self.items.extend(iter.map(|name| CompletionItem {
            label: name.to_string(),
            kind,
        }));
    }

    fn get_callables(package: &Package) -> impl Iterator<Item = &str> {
        package.items.values().filter_map(|i| match &i.kind {
            ItemKind::Callable(callable_decl) => Some(callable_decl.name.name.as_ref()),
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
