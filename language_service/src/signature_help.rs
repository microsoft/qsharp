// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// #[cfg(test)]
// mod tests;

use crate::{
    protocol::{ParameterInformation, SignatureHelp, SignatureInformation, Span},
    qsc_utils::{map_offset, Compilation},
};

pub(crate) fn get_signature_help(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Option<SignatureHelp> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.unit.sources, source_name, offset);
    let package = &compilation.unit.ast.package;

    let temp = true;

    if temp {
        Some(SignatureHelp {
            signatures: vec![SignatureInformation {
                label: "operation Foo(a: Int, b: Double, c: String) : Unit".to_string(),
                documentation: None,
                parameters: vec![
                    ParameterInformation {
                        label: Span { start: 14, end: 20 },
                        documentation: Some("The parameter `a`".to_string()),
                    },
                    ParameterInformation {
                        label: Span { start: 22, end: 31 },
                        documentation: Some("The parameter `b`".to_string()),
                    },
                    ParameterInformation {
                        label: Span { start: 33, end: 42 },
                        documentation: Some("The parameter `c`".to_string()),
                    },
                ],
            }],
            active_signature: 0,
            active_parameter: 2,
        })
    } else {
        None
    }
}
