// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    compilation::{Compilation, CompilationKind},
    protocol::{CodeLens, CodeLensCommand},
    qsc_utils::{into_range, span_contains},
};
use qsc::{
    hir::{Attr, ItemKind},
    line_column::Encoding,
};

pub(crate) fn get_code_lenses(
    compilation: &Compilation,
    source_name: &str,
    position_encoding: Encoding,
) -> Vec<CodeLens> {
    if matches!(compilation.kind, CompilationKind::Notebook) {
        // entrypoint actions don't work in notebooks
        return vec![];
    }

    let user_unit = compilation.user_unit();
    let source_span = compilation.package_span_of_source(source_name);

    // Get callables in the current source file with the @EntryPoint() attribute.
    // If there is more than one entrypoint, not our problem, we'll go ahead
    // and return code lenses for all. The duplicate entrypoint diagnostic
    // will be reported from elsewhere.
    let entry_point_decls = user_unit.package.items.values().filter_map(|item| {
        if span_contains(source_span, item.span.lo) {
            if let ItemKind::Callable(decl) = &item.kind {
                if item.attrs.iter().any(|a| a == &Attr::EntryPoint) {
                    return Some(decl);
                }
            }
        }
        None
    });

    entry_point_decls
        .flat_map(|decl| {
            let range = into_range(position_encoding, decl.span, &user_unit.sources);

            [
                CodeLens {
                    range,
                    command: CodeLensCommand::Run,
                },
                CodeLens {
                    range,
                    command: CodeLensCommand::Histogram,
                },
                CodeLens {
                    range,
                    command: CodeLensCommand::Estimate,
                },
                CodeLens {
                    range,
                    command: CodeLensCommand::Debug,
                },
            ]
        })
        .collect()
}
