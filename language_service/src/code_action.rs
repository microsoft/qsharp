use miette::Diagnostic;
use qsc::{
    compile::ErrorKind,
    error::WithSource,
    line_column::{Encoding, Range},
};

use crate::{
    compilation::Compilation,
    protocol::{CodeAction, CodeActionKind, TextEdit, WorkspaceEdit},
};

pub(crate) fn get_code_actions(
    compilation: &Compilation,
    source_name: &str,
    range: &Range,
    _position_encoding: Encoding,
) -> Vec<CodeAction> {
    let mut code_actions = Vec::new();

    // get relevant diagnostics
    let diagnostics = compilation
        .errors
        .iter()
        .filter(|error| is_error_relevant(error, source_name, range));

    for diagnostic in diagnostics {
        code_actions.push(CodeAction {
            title: diagnostic.to_string(),
            edit: Some(WorkspaceEdit {
                changes: vec![(
                    source_name.to_string(),
                    vec![TextEdit {
                        new_text: r#""CodeActions from rust!!""#.to_string(),
                        range: resolve_range(diagnostic),
                    }],
                )],
            }),
            kind: Some(CodeActionKind::QuickFix),
            is_preferred: None,
        });
    }

    code_actions
}

/// Returns true if the error:
///  - is in the file named `source_name`
///  - has a `Range` and it overlaps with the `code_action`'s range
fn is_error_relevant(error: &WithSource<ErrorKind>, source_name: &str, range: &Range) -> bool {
    let uri = error
        .sources()
        .first()
        .map(|source| source.name.to_string())
        .unwrap_or_default();

    let error_range = resolve_range(error);
    uri == source_name && range.intersection(&error_range).is_some()
}

fn resolve_range(e: &WithSource<ErrorKind>) -> Range {
    e.labels()
        .into_iter()
        .flatten()
        .map(|labeled_span| {
            let (source, span) = e.resolve_span(labeled_span.inner());
            let start = u32::try_from(span.offset()).expect("offset should fit in u32");
            let len = u32::try_from(span.len()).expect("length should fit in u32");
            qsc::line_column::Range::from_span(
                qsc::line_column::Encoding::Utf16,
                &source.contents,
                &qsc::Span {
                    lo: start,
                    hi: start + len,
                },
            )
        })
        .next()
        .expect("range should exist")
}
