// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_frontend::compile::PackageStore;

use qsc_eval::{debug::CallStack, val::GlobalId, Global, GlobalLookup};
use qsc_hir::hir::{Item, ItemKind};

#[must_use]
pub(crate) fn format_call_stack<'a>(
    store: &PackageStore,
    globals: &impl GlobalLookup<'a>,
    call_stack: &CallStack,
    error: &dyn std::error::Error,
) -> String {
    let mut trace = String::new();
    trace.push_str(&format!("Error: {error}\n"));
    trace.push_str("Call stack:\n");

    let mut frames = call_stack.clone().into_frames();
    frames.reverse();

    for frame in frames {
        let Some(Global::Callable(call)) = globals.get(frame.id) else { panic!("missing global"); };

        trace.push_str("    at ");
        if frame.functor.adjoint {
            trace.push_str("Adjoint ");
        }
        if frame.functor.controlled > 0 {
            trace.push_str(&format!("Controlled({}) ", frame.functor.controlled));
        }
        if let Some(item) = get_item_parent(store, frame.id) {
            if let Some(ns) = get_ns_name(&item) {
                trace.push_str(&format!("{ns}."));
            }
        }
        trace.push_str(&format!("{}", call.name.name));

        let name = get_item_file_name(store, frame.id);
        trace.push_str(&format!(
            " in {}",
            name.unwrap_or("<expression>".to_string())
        ));

        trace.push('\n');
    }
    trace
}

#[must_use]
fn get_item_parent(store: &PackageStore, id: GlobalId) -> Option<Item> {
    store.get(id.package).and_then(|unit| {
        let item = unit.package.items.get(id.item)?;
        if let Some(parent) = item.parent {
            let parent = unit.package.items.get(parent)?;
            Some(parent.clone())
        } else {
            None
        }
    })
}

#[must_use]
fn get_item_file_name(store: &PackageStore, id: GlobalId) -> Option<String> {
    store.get(id.package).and_then(|unit| {
        let item = unit.package.items.get(id.item)?;
        let source = unit.sources.find_by_offset(item.span.lo);
        source.map(|s| s.name.to_string())
    })
}

#[must_use]
fn get_ns_name(item: &Item) -> Option<String> {
    if let ItemKind::Namespace(ns, _) = &item.kind {
        Some(ns.name.to_string())
    } else {
        None
    }
}
