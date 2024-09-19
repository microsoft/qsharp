// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::compilation::{Compilation, CompilationKind};
use crate::protocol::{CompletionItem, CompletionItemKind, CompletionList, TextEdit};
use crate::qsc_utils::into_range;

use log::{log_enabled, trace, Level::Trace};
use qsc::ast::visit::{self, Visitor};
use qsc::display::{CodeDisplay, Lookup};

use qsc::hir::{ItemKind, Package, PackageId, Visibility};
use qsc::line_column::{Encoding, Position, Range};
use qsc::{
    resolve::{Local, LocalKind},
    PRELUDE,
};
use rustc_hash::FxHashSet;
use std::rc::Rc;
use std::sync::Arc;

type SortPriority = u32;

#[derive(Debug)]
/// Used to represent pre-existing imports in the completion context
struct ImportItem {
    path: Vec<Rc<str>>,
    alias: Option<Rc<str>>,
    is_glob: bool,
}

impl ImportItem {
    fn from_import_or_export_item(decl: &qsc::ast::ImportOrExportDecl) -> Vec<Self> {
        if decl.is_export() {
            return vec![];
        };
        let mut buf = Vec::with_capacity(decl.items.len());
        for item in &decl.items {
            let alias = item.alias.as_ref().map(|x| x.name.clone());
            let is_glob = item.is_glob;
            let path: qsc::ast::Idents = item.path.clone().into();
            let path = path.into_iter().map(|x| x.name.clone()).collect();

            buf.push(ImportItem {
                path,
                alias,
                is_glob,
            });
        }
        buf
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn get_completions(
    compilation: &Compilation,
    source_name: &str,
    position: Position,
    position_encoding: Encoding,
) -> CompletionList {
    let offset =
        compilation.source_position_to_package_offset(source_name, position, position_encoding);
    let user_ast_package = &compilation.user_unit().ast.package;

    if log_enabled!(Trace) {
        let last_char = compilation
            .user_unit()
            .sources
            .find_by_offset(offset)
            .map(|s| {
                let offset = offset - s.offset;
                if offset > 0 {
                    s.contents[(offset as usize - 1)..].chars().next()
                } else {
                    None
                }
            });
        trace!("the character before the cursor is: {last_char:?}");
    }

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
        start_of_namespace: None,
        current_namespace_name: None,
        imports: vec![],
    };
    context_finder.visit_package(user_ast_package);

    let insert_open_at = match compilation.kind {
        CompilationKind::OpenProject { .. } => context_finder.start_of_namespace,
        // Since notebooks don't typically contain namespace declarations,
        // open statements should just get before the first non-whitespace
        // character (i.e. at the top of the cell)
        CompilationKind::Notebook { .. } => {
            Some(get_first_non_whitespace_in_source(compilation, offset))
        }
    };

    let insert_open_range = insert_open_at.map(|o| {
        into_range(
            position_encoding,
            qsc::Span { lo: o, hi: o },
            &compilation.user_unit().sources,
        )
    });

    let indent = match insert_open_at {
        Some(start) => get_indent(compilation, start),
        None => String::new(),
    };

    let mut prelude_ns_ids: Vec<ImportItem> = PRELUDE
        .iter()
        .map(|ns| ImportItem {
            path: ns.iter().map(|x| Rc::from(*x)).collect(),
            alias: None,
            is_glob: true,
        })
        .collect();

    // The PRELUDE namespaces are always implicitly opened.
    context_finder.imports.append(&mut prelude_ns_ids);

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
                &context_finder.imports,
                insert_open_range,
                &context_finder.current_namespace_name,
                &indent,
            );
        }

        Context::CallableSignature => {
            builder.push_types();
            builder.push_locals(compilation, offset, false, true);
        }
        Context::Block => {
            // Pretty much anything goes in a block
            builder.push_locals(compilation, offset, true, true);
            builder.push_stmt_keywords();
            builder.push_expr_keywords();
            builder.push_types();
            builder.push_globals(
                compilation,
                &context_finder.imports,
                insert_open_range,
                &context_finder.current_namespace_name,
                &indent,
            );

            // Item decl keywords last, unlike in a namespace
            builder.push_item_decl_keywords();
        }
        Context::NoCompilation | Context::TopLevel => match compilation.kind {
            CompilationKind::OpenProject { .. } => builder.push_namespace_keyword(),
            CompilationKind::Notebook { .. } => {
                // For notebooks, the top-level allows for
                // more syntax.

                builder.push_locals(compilation, offset, true, true);

                // Item declarations
                builder.push_item_decl_keywords();

                // Things that go in a block
                builder.push_stmt_keywords();
                builder.push_expr_keywords();
                builder.push_types();
                builder.push_globals(
                    compilation,
                    &context_finder.imports,
                    insert_open_range,
                    &context_finder.current_namespace_name,
                    &indent,
                );

                // Namespace declarations - least likely to be used, so last
                builder.push_namespace_keyword();
            }
        },
    };

    CompletionList {
        items: builder.into_items(),
    }
}

fn get_first_non_whitespace_in_source(compilation: &Compilation, package_offset: u32) -> u32 {
    const QSHARP_MAGIC: &str = "//qsharp";
    let source = compilation
        .user_unit()
        .sources
        .find_by_offset(package_offset)
        .expect("source should exist in the user source map");

    // Skip the //qsharp magic if it exists (notebook cells)
    let start = if let Some(qsharp_magic_start) = source.contents.find(QSHARP_MAGIC) {
        qsharp_magic_start + QSHARP_MAGIC.len()
    } else {
        0
    };

    let source_after_magic = &source.contents[start..];

    let first = start
        + source_after_magic
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(source_after_magic.len());

    let first = u32::try_from(first).expect("source length should fit into u32");

    source.offset + first
}

fn get_indent(compilation: &Compilation, package_offset: u32) -> String {
    let source = compilation
        .user_unit()
        .sources
        .find_by_offset(package_offset)
        .expect("source should exist in the user source map");
    let source_offset = (package_offset - source.offset)
        .try_into()
        .expect("offset can't be converted to uszie");
    let before_offset = &source.contents[..source_offset];
    let mut indent = match before_offset.rfind(|c| c == '{' || c == '\n') {
        Some(begin) => {
            let indent = &before_offset[begin..];
            indent.strip_prefix('{').unwrap_or(indent)
        }
        None => before_offset,
    }
    .to_string();
    if !indent.starts_with('\n') {
        indent.insert(0, '\n');
    }
    indent
}

struct CompletionListBuilder {
    current_sort_group: SortPriority,
    items: FxHashSet<CompletionItem>,
}

impl CompletionListBuilder {
    fn new() -> Self {
        CompletionListBuilder {
            current_sort_group: 1,
            items: FxHashSet::default(),
        }
    }

    fn into_items(self) -> Vec<CompletionItem> {
        self.items.into_iter().collect()
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

    /// Populates self's completion list with globals
    fn push_globals(
        &mut self,
        compilation: &Compilation,
        imports: &[ImportItem],
        insert_open_range: Option<Range>,
        current_namespace_name: &Option<Vec<Rc<str>>>,
        indent: &String,
    ) {
        for (package_id, _) in compilation.package_store.iter().rev() {
            self.push_sorted_completions(Self::get_callables(
                compilation,
                package_id,
                imports,
                insert_open_range,
                current_namespace_name.as_deref(),
                indent,
            ));
        }

        for (id, unit) in compilation.package_store.iter().rev() {
            let alias = compilation.dependencies.get(&id).cloned().flatten();
            self.push_completions(Self::get_namespaces(&unit.package, alias));
        }
    }

    fn push_locals(
        &mut self,
        compilation: &Compilation,
        offset: u32,
        include_terms: bool,
        include_tys: bool,
    ) {
        self.push_sorted_completions(
            compilation
                .user_unit()
                .ast
                .locals
                .get_all_at_offset(offset)
                .iter()
                .filter_map(|candidate| {
                    local_completion(candidate, compilation, include_terms, include_tys)
                })
                .map(|item| (item, 0)),
        );
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
    fn push_sorted_completions(
        &mut self,
        iter: impl Iterator<Item = (CompletionItem, SortPriority)>,
    ) {
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

    #[allow(clippy::too_many_lines)]
    /// Get all callables in a package and return them as completion items, with a sort priority.
    fn get_callables<'a>(
        compilation: &'a Compilation,
        package_id: PackageId,
        // name and alias
        imports: &'a [ImportItem],
        // The range at which to insert an open statement if one is needed
        insert_open_at: Option<Range>,
        // The name of the current namespace, if any --
        current_namespace_name: Option<&'a [Rc<str>]>,
        indent: &'a String,
    ) -> impl Iterator<Item = (CompletionItem, SortPriority)> + 'a {
        let package = &compilation
            .package_store
            .get(package_id)
            .expect("package id should exist")
            .package;

        // if an alias exists for this dependency from the manifest,
        // this is used to prefix any access to items from this package with the alias
        let package_alias_from_manifest =
            compilation.dependencies.get(&package_id).cloned().flatten();

        let display = CodeDisplay { compilation };

        let is_user_package = compilation.user_package_id == package_id;

        // Given the package, get all completion items by iterating over its items
        // and converting any that would be valid as completions into completions
        package.items.values().filter_map(move |i| {
            package_item_to_completion_item(
                i,
                package,
                is_user_package,
                current_namespace_name,
                &display,
                &package_alias_from_manifest,
                imports,
                insert_open_at,
                indent,
            )
        })
    }

    fn get_namespaces(
        package: &'_ Package,
        package_alias: Option<Arc<str>>,
    ) -> impl Iterator<Item = CompletionItem> + '_ {
        package.items.values().filter_map(move |i| match &i.kind {
            ItemKind::Namespace(namespace, _) => {
                let qualification = namespace
                    .str_iter()
                    .into_iter()
                    .map(Rc::from)
                    .collect::<Vec<_>>();
                let label = format_external_name(&package_alias, &qualification[..], None);
                Some(CompletionItem::new(label, CompletionItemKind::Module))
            }
            _ => None,
        })
    }
}

/// Format an external fully qualified name
/// This will prepend the package alias and remove `Main` if it is the first namespace
fn format_external_name(
    package_alias_from_manifest: &Option<Arc<str>>,
    qualification: &[Rc<str>],
    name: Option<&str>,
) -> String {
    let mut fully_qualified_name: Vec<Rc<str>> = if let Some(alias) = package_alias_from_manifest {
        vec![Rc::from(&*alias.clone())]
    } else {
        vec![]
    };

    // if this comes from an external project's Main, then the path does not include Main
    let item_comes_from_main_of_external_project = package_alias_from_manifest.is_some()
        && qualification.len() == 1
        && qualification.first() == Some(&"Main".into());

    // So, if it is _not_ from an external project's `Main`, we include the namespace in the fully
    // qualified name.
    if !(item_comes_from_main_of_external_project) {
        fully_qualified_name.append(&mut qualification.to_vec());
    };

    if let Some(name) = name {
        fully_qualified_name.push(name.into());
    }

    fully_qualified_name.join(".")
}

/// Convert a local into a completion item
fn local_completion(
    candidate: &Local,
    compilation: &Compilation,
    include_terms: bool,
    include_tys: bool,
) -> Option<CompletionItem> {
    let display = CodeDisplay { compilation };
    let (kind, detail) = match &candidate.kind {
        LocalKind::Item(item_id) => {
            let item = compilation.resolve_item_relative_to_user_package(item_id);
            let (detail, kind) = match &item.0.kind {
                ItemKind::Callable(decl) => {
                    if !include_terms {
                        return None;
                    }
                    (
                        Some(display.hir_callable_decl(decl).to_string()),
                        CompletionItemKind::Function,
                    )
                }
                ItemKind::Namespace(_, _) => {
                    panic!("did not expect local namespace item")
                }
                ItemKind::Ty(_, udt) => {
                    if !include_terms && !include_tys {
                        return None;
                    }
                    (
                        Some(display.hir_udt(udt).to_string()),
                        CompletionItemKind::Interface,
                    )
                }
                // We don't want completions for items exported from the local scope
                ItemKind::Export(_, _) => return None,
            };
            (kind, detail)
        }
        LocalKind::Var(node_id) => {
            if !include_terms {
                return None;
            }
            let detail = Some(display.name_ty_id(&candidate.name, *node_id).to_string());
            (CompletionItemKind::Variable, detail)
        }
        LocalKind::TyParam(_) => {
            if !include_tys {
                return None;
            }
            (CompletionItemKind::TypeParameter, None)
        }
    };

    Some(CompletionItem {
        label: candidate.name.to_string(),
        kind,
        sort_text: None,
        detail,
        additional_text_edits: None,
    })
}

struct ContextFinder {
    offset: u32,
    context: Context,
    imports: Vec<ImportItem>,
    start_of_namespace: Option<u32>,
    current_namespace_name: Option<Vec<Rc<str>>>,
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
        if namespace.span.contains(self.offset) {
            self.current_namespace_name = Some(namespace.name.clone().into());
            self.context = Context::Namespace;
            self.imports = vec![];
            self.start_of_namespace = None;
            visit::walk_namespace(self, namespace);
        }
    }

    fn visit_item(&mut self, item: &'_ qsc::ast::Item) {
        if self.start_of_namespace.is_none() {
            self.start_of_namespace = Some(item.span.lo);
        }

        match &*item.kind {
            qsc::ast::ItemKind::Open(name, alias) => {
                let open_as_import = ImportItem {
                    path: name.clone().into(),
                    alias: alias.as_ref().map(|x| x.name.clone()),
                    is_glob: true,
                };
                self.imports.push(open_as_import);
            }
            qsc::ast::ItemKind::ImportOrExport(decl) => {
                // if this is an import, populate self.imports
                if decl.is_import() {
                    self.imports
                        .append(&mut ImportItem::from_import_or_export_item(decl));
                }
            }
            _ => (),
        }

        if item.span.contains(self.offset) {
            visit::walk_item(self, item);
        }
    }

    fn visit_callable_decl(&mut self, decl: &'_ qsc::ast::CallableDecl) {
        if decl.span.contains(self.offset) {
            // This span covers the body too, but the
            // context will get overwritten by visit_block
            // if the offset is inside the actual body
            self.context = Context::CallableSignature;
            visit::walk_callable_decl(self, decl);
        }
    }

    fn visit_block(&mut self, block: &'_ qsc::ast::Block) {
        if block.span.contains(self.offset) {
            self.context = Context::Block;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn package_item_to_completion_item(
    item: &qsc::hir::Item,
    package: &qsc::hir::Package,
    is_user_package: bool,
    current_namespace_name: Option<&[Rc<str>]>,
    display: &CodeDisplay,
    package_alias_from_manifest: &Option<Arc<str>>,
    imports: &[ImportItem],
    insert_open_at: Option<Range>,
    indent: &String,
) -> Option<(CompletionItem, SortPriority)> {
    // We only want items whose parents are namespaces
    if let Some(item_id) = item.parent {
        if let Some(parent) = package.items.get(item_id) {
            if let ItemKind::Namespace(callable_namespace, _) = &parent.kind {
                // filter out internal packages that are not from the user's
                // compilation
                if matches!(item.visibility, Visibility::Internal) && !is_user_package {
                    return None; // ignore item if not in the user's package
                }

                match &item.kind {
                    ItemKind::Callable(callable_decl) => {
                        return Some(callable_decl_to_completion_item(
                            callable_decl,
                            current_namespace_name,
                            display,
                            package_alias_from_manifest,
                            callable_namespace,
                            imports,
                            insert_open_at,
                            indent,
                        ))
                    }
                    _ => return None,
                }
            }
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn callable_decl_to_completion_item(
    callable_decl: &qsc::hir::CallableDecl,
    current_namespace_name: Option<&[Rc<str>]>,
    display: &CodeDisplay,
    package_alias_from_manifest: &Option<Arc<str>>,
    callable_namespace: &qsc::hir::Idents,
    imports: &[ImportItem],
    insert_import_at: Option<Range>,
    indent: &String,
) -> (CompletionItem, SortPriority) {
    let name = callable_decl.name.name.as_ref();
    // details used when rendering the completion item
    let detail = Some(display.hir_callable_decl(callable_decl).to_string());
    // Everything that starts with a __ goes last in the list
    let sort_group = u32::from(name.starts_with("__"));

    let namespace_as_strs = Into::<Vec<_>>::into(callable_namespace);

    // Now, we calculate the qualification that goes before the import
    // item.
    // if an exact import already exists, or if that namespace
    // is glob imported, then there is no qualification.
    // If there is no matching import or glob import, then the
    // qualification is the full namespace name.
    //
    // An exact import is an import that matches the namespace
    // and item name exactly
    let preexisting_exact_import = imports.iter().any(|import_item| {
        let import_item_namespace = &import_item.path[..import_item.path.len() - 1];
        let import_item_name = import_item.path.last().map(|x| &**x);
        *import_item_namespace == namespace_as_strs[..] && import_item_name == Some(name)
    });

    let preexisting_glob_import = imports
        .iter()
        .any(|import_item| import_item.path == namespace_as_strs[..] && import_item.is_glob);

    let preexisting_namespace_alias = imports.iter().find_map(|import_item| {
        if import_item.path == namespace_as_strs[..] {
            import_item.alias.as_ref().map(|x| vec![x.clone()])
        } else {
            None
        }
    });

    // this conditional is kind of gnarly, but it boils down to:
    // do we need to add an import statement for this item, or is it already accessible?
    // If so, what edit?
    // The first condition is if we are in the same namespace,
    // the second is if there is already an exact import or glob import of this item. In these
    // cases, we do not need to add an import statement.
    // The third condition is if there is no existing imports, and no existing package alias (a
    // form of import), then we need to add an import statement.
    let additional_text_edit =
        // first condition
        if current_namespace_name == Some(namespace_as_strs.as_slice()) ||
        // second condition
        (preexisting_exact_import || preexisting_glob_import) { None }
        // third condition
        else if preexisting_namespace_alias.is_none() {
            // if there is no place to insert an import, then we can't add an import.
            if let Some(range) = insert_import_at {
                let import_text = format_external_name(
                    package_alias_from_manifest,
                    &Into::<Vec<_>>::into(callable_namespace),
                    Some(name),
                );
                Some(TextEdit {
                    new_text: format!("import {import_text};{indent}",),
                    range,
                })
            } else { None }
        } else {
            None
        };

    let label = if let Some(qualification) = preexisting_namespace_alias {
        format_external_name(package_alias_from_manifest, &qualification, Some(name))
    } else {
        name.to_owned()
    };

    (
        CompletionItem {
            label,
            kind: CompletionItemKind::Function,
            sort_text: None, // This will get filled in during `push_sorted_completions`
            detail,
            additional_text_edits: additional_text_edit.map(|x| vec![x]),
        },
        sort_group,
    )
}
