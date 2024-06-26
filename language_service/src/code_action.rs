// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use qsc::{
    compile::ErrorKind,
    error::WithSource,
    line_column::{Encoding, Range},
    Span,
};
use qsc_linter::{AstLint, HirLint};

use crate::{
    compilation::Compilation,
    protocol::{CodeAction, CodeActionKind, TextEdit, WorkspaceEdit},
};

pub(crate) fn get_code_actions(
    compilation: &Compilation,
    source_name: &str,
    range: Range,
    position_encoding: Encoding,
) -> Vec<CodeAction> {
    // Compute quick_fixes and other code_actions, and then merge them together
    let span = compilation.source_range_to_package_span(source_name, range, position_encoding);
    quick_fixes(compilation, source_name, span, position_encoding)
}

fn quick_fixes(
    compilation: &Compilation,
    source_name: &str,
    span: Span,
    encoding: Encoding,
) -> Vec<CodeAction> {
    let mut code_actions = Vec::new();

    // get relevant diagnostics
    let diagnostics = compilation
        .errors
        .iter()
        .filter(|error| is_error_relevant(error, span));

    // An example of what quickfixes could look like if they were generated here.
    // The other option I considered was generating the quickfixes when the errors
    // are initially issued. But that has two problems:
    //  1. It does unnecessary computations at compile time, that would go to waste if using the CLI compiler.
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
                                // We want to remove the redundant semicolons, so the
                                // replacement text is just an empty string.
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
                                // Same source code without the first and last characters
                                // which should correspond to the redundant parentheses.
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
                LintKind::Hir(HirLint::DeprecatedWithOperator) => code_actions.push(CodeAction {
                    title: diagnostic.to_string(),
                    edit: Some(WorkspaceEdit {
                        changes: vec![(
                            source_name.to_string(),
                            vec![TextEdit {
                                // Use the text from the lint to replace the deprecated operator.
                                new_text: lint.code_action_edits[0].0.clone(),
                                range: resolve_range(diagnostic, encoding)
                                    .expect("range should exist"),
                            }],
                        )],
                    }),
                    kind: Some(CodeActionKind::QuickFix),
                    is_preferred: None,
                }),
                LintKind::Hir(HirLint::DeprecatedDoubleColonOperator) => {
                    let text_edits = code_action_edits_to_text_edits(
                        &lint.code_action_edits,
                        compilation,
                        source_name,
                        encoding,
                    );
                    code_actions.push(CodeAction {
                        title: diagnostic.to_string(),
                        edit: Some(WorkspaceEdit {
                            changes: vec![(source_name.to_string(), text_edits)],
                        }),
                        kind: Some(CodeActionKind::QuickFix),
                        is_preferred: None,
                    });
                }
                _ => (),
            }
        }
    }

    code_actions
}

fn code_action_edits_to_text_edits(
    code_action_edits: &[(String, Span)],
    compilation: &Compilation,
    source_name: &str,
    encoding: Encoding,
) -> Vec<TextEdit> {
    let source = compilation
        .user_unit()
        .sources
        .find_by_name(source_name)
        .expect("source should exist");
    code_action_edits
        .iter()
        .map(|(new_text, span)| TextEdit {
            new_text: new_text.clone(),
            range: qsc::line_column::Range::from_span(encoding, &source.contents, span),
        })
        .collect()
}

/// Returns true if the error has a `Range` and it overlaps
/// with the code action's range.
fn is_error_relevant(error: &WithSource<ErrorKind>, span: Span) -> bool {
    let Some(error_span) = resolve_span(error) else {
        return false;
    };
    span.intersection(&error_span).is_some()
}

/// Extracts the uri and `Span` from an error.
fn resolve_span(e: &WithSource<ErrorKind>) -> Option<Span> {
    e.labels()
        .into_iter()
        .flatten()
        .map(|labeled_span| {
            let start = u32::try_from(labeled_span.offset()).expect("offset should fit in u32");
            let len = u32::try_from(labeled_span.len()).expect("length should fit in u32");
            qsc::Span {
                lo: start,
                hi: start + len,
            }
        })
        .next()
}

/// Extracts the `Range` from an error.
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

/// Returns a substring of the user code's `SourceMap` in the range `lo..hi`.
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
