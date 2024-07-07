// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    compilation::{Compilation, CompilationKind},
    protocol::{CodeLens, CodeLensCommand, OperationInfo},
    qsc_utils::into_range,
};
use qsc::{
    circuit::qubit_param_info,
    hir::{Expr, ExprKind, ItemId, ItemKind, LocalItemId, Package, Res, Visibility},
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
    let package = &user_unit.package;
    let source_span = compilation.package_span_of_source(source_name);

    let entry_item_id = entry_callable(package);

    // Get callables in the current source file.
    let callables = package.items.iter().filter_map(|(item_id, item)| {
        if source_span.contains(item.span.lo) {
            // We don't support any commands for internal operations.
            if matches!(item.visibility, Visibility::Internal) {
                return None;
            }

            if let ItemKind::Callable(decl) = &item.kind {
                if let Some(ItemKind::Namespace(ns, _)) = item
                    .parent
                    .and_then(|parent_id| package.items.get(parent_id))
                    .map(|parent| &parent.kind)
                {
                    let namespace = ns.name();
                    let range = into_range(position_encoding, decl.span, &user_unit.sources);
                    let name = decl.name.name.clone();

                    return Some((item, range, namespace, name, Some(item_id) == entry_item_id));
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

/// Uses the entry expression in the package to find the
/// entrypoint callable item id. The entry expression has to
/// be a call to a parameterless operation or function. This is the
/// specific pattern generated by the "generate entry expression" pass.
/// In practice, this callable will be the one tagged with the
/// `@EntryPoint()` attribute or named "Main".
fn entry_callable(package: &Package) -> Option<LocalItemId> {
    if let Some(Expr {
        kind: ExprKind::Call(callee, args),
        ..
    }) = &package.entry
    {
        if let ExprKind::Tuple(args) = &args.kind {
            if args.is_empty() {
                if let ExprKind::Var(
                    Res::Item(ItemId {
                        package: None,
                        item,
                    }),
                    ..,
                ) = callee.kind
                {
                    return Some(item);
                }
            }
        }
    }
    None
}
