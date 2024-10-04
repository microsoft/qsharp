// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod global_items;
mod locals;
mod path_context;
#[cfg(test)]
mod tests;
mod text_edits;

use crate::{
    compilation::{Compilation, CompilationKind},
    protocol::{CompletionItem, CompletionItemKind, CompletionList, TextEdit},
};
use global_items::Globals;
use locals::Locals;
use log::{log_enabled, trace, Level::Trace};
use path_context::IncompletePath;
use qsc::{
    line_column::{Encoding, Position},
    parse::completion::{
        possible_words_at_offset_in_fragments, possible_words_at_offset_in_source,
        HardcodedIdentKind, NameKind, PathKind, WordKinds,
    },
    LanguageFeatures,
};
use rustc_hash::FxHashSet;
use std::iter::once;
use text_edits::TextEditRange;

type SortPriority = u32;

pub(crate) fn get_completions(
    compilation: &Compilation,
    source_name: &str,
    position: Position,
    position_encoding: Encoding,
) -> CompletionList {
    let package_offset =
        compilation.source_position_to_package_offset(source_name, position, position_encoding);
    let source = compilation
        .user_unit()
        .sources
        .find_by_offset(package_offset)
        .expect("source should exist");
    let source_offset: u32 = package_offset - source.offset;

    // The parser uses the relative source name to figure out the implicit namespace.
    let source_name_relative = compilation
        .user_unit()
        .sources
        .relative_sources()
        .find(|s| s.offset == source.offset)
        .expect("source should exist in the user source map")
        .name;

    if log_enabled!(Trace) {
        let last_char = if source_offset > 0 {
            source.contents[(package_offset as usize - 1)..]
                .chars()
                .next()
        } else {
            None
        };
        trace!("the character before the cursor is: {last_char:?}");
    }

    // What kinds of words are expected at the cursor location?
    let expected_words_at_cursor = expected_word_kinds(
        compilation,
        &source_name_relative,
        &source.contents,
        source_offset,
    );

    // Now that we have the information from the parser about what kinds of
    // words are expected, gather the actual words (identifiers, keywords, etc) for each kind.

    // Keywords and other hardcoded words
    let hardcoded_completions = collect_hardcoded_words(expected_words_at_cursor);

    // The tricky bit: globals, locals, names we need to gather from the compilation.
    let name_completions = collect_names(
        expected_words_at_cursor,
        package_offset,
        compilation,
        position_encoding,
    );

    // We have all the data, put everything into a completion list.
    into_completion_list(once(hardcoded_completions).chain(name_completions))
}

/// Invokes the parser to determine what kinds of words are expected at the cursor location.
fn expected_word_kinds(
    compilation: &Compilation,
    source_name_relative: &str,
    source_contents: &str,
    cursor_offset: u32,
) -> WordKinds {
    match &compilation.kind {
        CompilationKind::OpenProject {
            package_graph_sources,
        } => possible_words_at_offset_in_source(
            source_contents,
            Some(source_name_relative),
            package_graph_sources.root.language_features,
            cursor_offset,
        ),
        CompilationKind::Notebook { project } => possible_words_at_offset_in_fragments(
            source_contents,
            project.as_ref().map_or(LanguageFeatures::default(), |p| {
                p.package_graph_sources.root.language_features
            }),
            cursor_offset,
        ),
    }
}

/// Collects hardcoded completions from the given set of parser predictions.
///
/// Hardcoded words are actual keywords (`let`, etc) as well as other words that are
/// hardcoded into the language (`Qubit`, `EntryPoint`, etc)
fn collect_hardcoded_words(expected: WordKinds) -> Vec<Completion> {
    let mut completions = Vec::new();
    for word_kind in expected.iter_hardcoded_ident_kinds() {
        match word_kind {
            HardcodedIdentKind::Qubit => {
                completions.push(Completion::new(
                    "Qubit".to_string(),
                    CompletionItemKind::Interface,
                ));
            }
            HardcodedIdentKind::Attr => {
                completions.extend([
                    Completion::new("EntryPoint".to_string(), CompletionItemKind::Interface),
                    Completion::new("Config".to_string(), CompletionItemKind::Interface),
                ]);
            }
        }
    }

    for keyword in expected.iter_keywords() {
        let keyword = keyword.to_string();
        // Skip adding the underscore keyword to the list, it's more confusing than helpful.
        if keyword != "_" {
            completions.push(Completion::new(keyword, CompletionItemKind::Keyword));
        }
    }

    completions
}

/// Collects names from the compilation that match the expected word kinds.
fn collect_names(
    expected: WordKinds,
    cursor_offset: u32,
    compilation: &Compilation,
    position_encoding: Encoding,
) -> Vec<Vec<Completion>> {
    let mut groups = Vec::new();

    for name_kind in expected.iter_name_kinds() {
        match name_kind {
            NameKind::Path(path_kind) => {
                let globals = Globals::init(cursor_offset, compilation);
                let edit_range = TextEditRange::init(cursor_offset, compilation, position_encoding);
                let locals = Locals::new(cursor_offset, compilation);

                groups.extend(collect_paths(path_kind, &globals, &locals, &edit_range));
            }
            NameKind::PathSegment => {
                let globals = Globals::init(cursor_offset, compilation);
                let path =
                    IncompletePath::init(cursor_offset, &compilation.user_unit().ast.package);

                groups.extend(collect_path_segments(&globals, &path));
            }
            NameKind::TyParam => {
                let locals = Locals::new(cursor_offset, compilation);
                groups.push(locals.type_names());
            }
            NameKind::Field => {
                // Not yet implemented
            }
        };
    }
    groups
}

/// Collects paths that are applicable at the current cursor offset,
/// taking into account all the relevant name resolution context,
/// such as scopes and visibility at the cursor location.
///
/// Note that the list will not contain full paths to items. It will typically
/// only include leading qualifier, or the item name along with an auto-import edit.
/// For example, the item `Microsoft.Quantum.Diagnostics.DumpMachine` will contribute
/// two completion items: the leading qualifier (namespace) `Microsoft` and the
/// callable name `DumpMachine` with an auto-import edit to add `Microsoft.Quantum.Diagnostics`.
fn collect_paths(
    expected: PathKind,
    globals: &Globals,
    locals_at_cursor: &Locals,
    text_edit_range: &TextEditRange,
) -> Vec<Vec<Completion>> {
    let mut global_names = Vec::new();
    let mut locals_and_builtins = Vec::new();
    match expected {
        PathKind::Expr => {
            locals_and_builtins.push(locals_at_cursor.expr_names());
            global_names.extend(globals.expr_names(text_edit_range));
        }
        PathKind::Ty => {
            locals_and_builtins.push(locals_at_cursor.type_names());
            locals_and_builtins.push(
                [
                    "Qubit", "Int", "Unit", "Result", "Bool", "BigInt", "Double", "Pauli", "Range",
                    "String",
                ]
                .map(|s| Completion::new(s.to_string(), CompletionItemKind::Interface))
                .into(),
            );

            global_names.extend(globals.type_names(text_edit_range));
        }
        PathKind::Import => {
            global_names.extend(globals.expr_names_in_scope_only());
            global_names.extend(globals.type_names_in_scope_only());
            global_names.push(globals.namespaces());
        }
        PathKind::Struct => {
            global_names.extend(globals.type_names(text_edit_range));
        }
        PathKind::Namespace => {
            global_names.push(globals.namespaces());
        }
    }

    // This order ensures that locals and builtins come before globals
    // in the eventual completion list
    locals_and_builtins.extend(global_names);
    locals_and_builtins
}

/// Collects path segments that are applicable at the current cursor offset.
/// Assumes that the cursor is in the middle of a path, either immediately
/// following a `.` or in the middle of a path segment that follows a `.` .
///
/// Narrows down the list based on the qualifier before the `.` as well
/// as the name kind expected at that syntax node. For example,
/// `let x : Microsoft.Quantum.Math.↘`  should include `Complex` (a type) while
/// `let x = Microsoft.Quantum.Math.↘` should include `PI` (a callable).
fn collect_path_segments(globals: &Globals, path_context: &IncompletePath) -> Vec<Vec<Completion>> {
    let (path_kind, qualifier) = path_context.context();

    match path_kind {
        PathKind::Namespace => globals.namespaces_in(&qualifier),
        PathKind::Expr => globals.expr_names_in(&qualifier),
        PathKind::Ty | PathKind::Struct => globals.type_names_in(&qualifier),
        PathKind::Import => [
            globals.expr_names_in(&qualifier),
            globals.type_names_in(&qualifier),
            globals.namespaces_in(&qualifier),
        ]
        .into_iter()
        .flatten()
        .collect(),
    }
}

/// Builds the `CompletionList` from the ordered groups of completion items.
fn into_completion_list(groups: impl Iterator<Item = Vec<Completion>>) -> CompletionList {
    // The HashSet serves to eliminate duplicates.
    let mut items = FxHashSet::default();

    // Build list one group at a time. The sort order
    // is determined by the order in which the groups are pushed.
    // Within each group, items are sorted by sort_priority.
    for (current_sort_group, group) in groups.enumerate() {
        items.extend(group.into_iter().map(
            |Completion {
                 item,
                 sort_priority,
             }| CompletionItem {
                // The sort_text is what the editor will ultimately use to
                // sort the items, so we're using the sort priority as a prefix.
                sort_text: Some(format!(
                    "{:02}{:02}{}",
                    current_sort_group, sort_priority, item.label
                )),
                ..item
            },
        ));
    }

    CompletionList {
        items: items.into_iter().collect(),
    }
}

struct Completion {
    item: CompletionItem,
    sort_priority: SortPriority,
}

impl Completion {
    fn new(label: String, kind: CompletionItemKind) -> Self {
        Self::with_detail(label, kind, None)
    }

    fn with_detail(label: String, kind: CompletionItemKind, detail: Option<String>) -> Self {
        Self::with_text_edits(label, kind, detail, None, 0)
    }

    fn with_text_edits(
        label: String,
        kind: CompletionItemKind,
        detail: Option<String>,
        additional_text_edits: Option<Vec<TextEdit>>,
        sort_priority: u32,
    ) -> Completion {
        Completion {
            item: CompletionItem {
                label,
                kind,
                // This will be populated from sort_priority when the list gets built
                sort_text: None,
                detail,
                additional_text_edits,
            },
            sort_priority,
        }
    }
}
