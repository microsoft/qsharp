// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use qsc::{
    compile::ErrorKind,
    error::WithSource,
    line_column::{Encoding, Range},
};
use qsc_linter::{AstLint, HirLint};

use crate::{
    compilation::Compilation,
    protocol::{CodeAction, CodeActionKind, TextEdit, WorkspaceEdit},
};

pub(crate) fn get_code_actions(
    compilation: &Compilation,
    source_name: &str,
    range: &Range,
    encoding: Encoding,
) -> Vec<CodeAction> {
    // Compute quick_fixes and other code_actions, and then merge them together
    quick_fixes(compilation, source_name, range, encoding)
}

fn quick_fixes(
    compilation: &Compilation,
    source_name: &str,
    range: &Range,
    encoding: Encoding,
) -> Vec<CodeAction> {
    let mut code_actions = Vec::new();

    // get relevant diagnostics
    let diagnostics = compilation
        .errors
        .iter()
        .filter(|error| is_error_relevant(error, source_name, range, encoding));

    // An example of what quickfixes could look like if they were generated here.
    // The other option I considered was generating the quickfixes when the errors
    // are initially issued. But that has two problems:
    //  1. It does unnecesary computations at compile time, that would go to waste if using the CLI compiler.
    //  2. The quickfix logic would be spread across many crates in the compiler.
    for diagnostic in diagnostics {
        if let ErrorKind::Lint(lint) = diagnostic.error() {
            use qsc::linter::LintKind;
            match lint.kind {
                LintKind::Ast(AstLint::RedundantSemicolons) => code_actions.push(CodeAction {
                    title: diagnostic.to_string(),
                    edit: Some(WorkspaceEdit {
                        changes: vec![(
                            source_name.to_string(),
                            vec![TextEdit {
                                new_text: String::new(),
                                range: resolve_range(diagnostic, encoding)
                                    .expect("range should exist"),
                            }],
                        )],
                    }),
                    kind: Some(CodeActionKind::QuickFix),
                    is_preferred: None,
                }),
                LintKind::Ast(AstLint::NeedlessParens) => code_actions.push(CodeAction {
                    title: diagnostic.to_string(),
                    edit: Some(WorkspaceEdit {
                        changes: vec![(
                            source_name.to_string(),
                            vec![TextEdit {
                                new_text: get_source_code(
                                    compilation,
                                    lint.span.lo + 1,
                                    lint.span.hi - 1,
                                ),
                                range: resolve_range(diagnostic, encoding)
                                    .expect("range should exist"),
                            }],
                        )],
                    }),
                    kind: Some(CodeActionKind::QuickFix),
                    is_preferred: None,
                }),
                LintKind::Ast(AstLint::DivisionByZero) | LintKind::Hir(HirLint::Placeholder) => (),
            }
        }
    }

    code_actions
}

/// Returns true if the error:
///  - is in the file named `source_name`
///  - has a `Range` and it overlaps with the `code_action`'s range
fn is_error_relevant(
    error: &WithSource<ErrorKind>,
    source_name: &str,
    range: &Range,
    encoding: Encoding,
) -> bool {
    let uri = error
        .sources()
        .first()
        .map(|source| source.name.to_string())
        .unwrap_or_default();

    let Some(error_range) = resolve_range(error, encoding) else {
        return false;
    };
    uri == source_name && range.intersection(&error_range).is_some()
}

/// Extracts the `Range` from an error
fn resolve_range(e: &WithSource<ErrorKind>, encoding: Encoding) -> Option<Range> {
    e.labels()
        .into_iter()
        .flatten()
        .map(|labeled_span| {
            let (source, span) = e.resolve_span(labeled_span.inner());
            let start = u32::try_from(span.offset()).expect("offset should fit in u32");
            let len = u32::try_from(span.len()).expect("length should fit in u32");
            qsc::line_column::Range::from_span(
                encoding,
                &source.contents,
                &qsc::Span {
                    lo: start,
                    hi: start + len,
                },
            )
        })
        .next()
}

/// Returns a substring of the user code's `SourceMap` in the range lo..hi
fn get_source_code(compilation: &Compilation, lo: u32, hi: u32) -> String {
    let unit = compilation
        .package_store
        .get(compilation.user_package_id)
        .expect("user package should exist");

    let source = unit
        .sources
        .find_by_offset(lo)
        .expect("source should exist");

    let lo = (lo - source.offset) as usize;
    let hi = (hi - source.offset) as usize;
    source.contents[lo..hi].to_string()
}
