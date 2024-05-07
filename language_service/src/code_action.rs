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
                        new_text: r#""Hello from rust!!""#.to_string(),
                        range: *range,
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
fn is_error_relevant(error: &WithSource<ErrorKind>, source_name: &str, _range: &Range) -> bool {
    let uri = error
        .sources()
        .first()
        .map(|source| source.name.to_string())
        .unwrap_or_default();

    log::error!("{uri} ==? {source_name}");

    uri == source_name
}
