// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    compilation::{Compilation, CompilationKind},
    protocol::{CodeLens, CodeLensCommand, OperationInfo},
    qsc_utils::{into_range, span_contains},
};
use qsc::{
    circuit::qubit_param_info,
    hir::{Attr, ItemKind, Visibility},
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

    // Get callables in the current source file.
    let callables = user_unit.package.items.values().filter_map(|item| {
        if span_contains(source_span, item.span.lo) {
            // We don't support any commands for internal operations.
            if matches!(item.visibility, Visibility::Internal) {
                return None;
            }

            if let ItemKind::Callable(decl) = &item.kind {
                if let Some(ItemKind::Namespace(ns, _)) = item
                    .parent
                    .and_then(|parent_id| user_unit.package.items.get(parent_id))
                    .map(|parent| &parent.kind)
                {
                    let namespace = ns.name.to_string();
                    let range = into_range(position_encoding, decl.span, &user_unit.sources);
                    let name = decl.name.name.clone();

                    if item.attrs.iter().any(|a| a == &Attr::EntryPoint) {
                        // If there is more than one entrypoint, not our problem, we'll go ahead
                        // and return code lenses for all. The duplicate entrypoint diagnostic
                        // will be reported from elsewhere.
                        return Some((item, range, namespace, name, true));
                    }

                    return Some((item, range, namespace, name, false));
                }
            }
        }
        None
    });

    callables
        .flat_map(|(item, range, namespace, name, is_entry_point)| {
            if is_entry_point {
                vec![
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
                    CodeLens {
                        range,
                        command: CodeLensCommand::Circuit(None),
                    },
                ]
            } else {
                if let Some((_, total_num_qubits)) = qubit_param_info(item) {
                    return vec![CodeLens {
                        range,
                        command: CodeLensCommand::Circuit(Some(OperationInfo {
                            operation: format!("{namespace}.{name}"),
                            total_num_qubits,
                        })),
                    }];
                }
                vec![]
            }
        })
        .collect()
}
