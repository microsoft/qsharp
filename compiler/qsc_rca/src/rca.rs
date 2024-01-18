// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::data_structures::{
    derive_callable_input_elements, derive_callable_input_params,
    initialize_callable_variables_map, CallableInputElement, CallableInputElementKind, InputParam,
};
use crate::fir_extensions::CallableDeclExt;
use crate::{ComputePropsLookup, ItemComputeProps, PackageStoreComputeProps, PatComputeProps};
use qsc_fir::fir::{
    CallableDecl, Global, PackageId, PackageStore, PackageStoreLookup, StoreItemId, StoreStmtId,
};

pub fn analyze_package_and_update_compute_props(
    id: PackageId,
    package_store: &PackageStore,
    compute_props: &mut PackageStoreComputeProps,
) {
    // Clear current compute properties of the package to make sure we are performing a coherent analysis on it.
    let package_compute_props = compute_props
        .get_mut(id)
        .expect("package compute properties should exist");
    package_compute_props.clear();
    let package = package_store.get(id).expect("package should exist");

    // Analyze all items in the package.
    for (item_id, _) in &package.items {
        analyze_and_update_item_compute_props((id, item_id).into(), package_store, compute_props);
    }

    // Analyze all statements in the package.
    for (stmt_id, _) in &package.stmts {
        analyze_and_update_stmt_compute_props((id, stmt_id).into(), package_store, compute_props);
    }
}

fn analyze_and_update_callable_compute_props(
    id: StoreItemId,
    callable: &CallableDecl,
    package_store: &PackageStore,
    compute_props: &mut PackageStoreComputeProps,
) {
    // This function is only called when a callable has not already been analyzed.
    if compute_props.find_item(id).is_some() {
        panic!("callable is already analyzed");
    }

    // First, set the compute properties of all the patterns that are part of the callable input.
    let input_elements = derive_callable_input_elements(
        callable,
        &package_store
            .get(id.package)
            .expect("package should exist")
            .pats,
    );
    analyze_and_update_callable_input_elements(id, input_elements.iter(), compute_props);

    // Analyze the callable depending on its type.
    let input_params = derive_callable_input_params(input_elements.iter());
    if callable.is_intrinsic() {
        analyze_and_update_intrinsic_callable_compute_props(
            id,
            callable,
            input_params.iter(),
            package_store,
            compute_props,
        );
    } else {
        analyze_and_update_non_intrinsic_callable_compute_props(
            id,
            callable,
            input_params.iter(),
            package_store,
            compute_props,
        );
    }
}

fn analyze_and_update_callable_input_elements<'a>(
    callable_id: StoreItemId,
    input_elements: impl Iterator<Item = &'a CallableInputElement>,
    compute_props: &mut PackageStoreComputeProps,
) {
    // This function is only called when a callable has not already been analyzed.
    if compute_props.find_item(callable_id).is_some() {
        panic!("callable is already analyzed");
    }

    for element in input_elements {
        match &element.kind {
            CallableInputElementKind::Discard => compute_props.insert_pat(
                (callable_id.package, element.pat).into(),
                PatComputeProps::InputParamDiscard,
            ),
            CallableInputElementKind::Node(node_id) => compute_props.insert_pat(
                (callable_id.package, element.pat).into(),
                PatComputeProps::InputParamNode(*node_id),
            ),
            CallableInputElementKind::Tuple(tuple_pats) => compute_props.insert_pat(
                (callable_id.package, element.pat).into(),
                PatComputeProps::InputParamTuple(tuple_pats.clone()),
            ),
        }
    }
}

fn analyze_and_update_intrinsic_callable_compute_props<'a>(
    id: StoreItemId,
    _callable: &CallableDecl,
    _input_params: impl Iterator<Item = &'a InputParam>,
    _package_store: &PackageStore,
    compute_props: &mut PackageStoreComputeProps,
) {
    // This function is only called when a callable has not already been analyzed.
    if compute_props.find_item(id).is_some() {
        panic!("callable is already analyzed");
    }

    // TODO (cesarzc): Implement.
}

fn analyze_and_update_item_compute_props(
    id: StoreItemId,
    package_store: &PackageStore,
    compute_props: &mut PackageStoreComputeProps,
) {
    // If the item has already been analyzed, there's nothing left to do.
    if compute_props.find_item(id).is_some() {
        return;
    }

    if let Some(Global::Callable(callable)) = package_store.get_global(id) {
        analyze_and_update_callable_compute_props(id, callable, package_store, compute_props);
    } else {
        compute_props.insert_item(id, ItemComputeProps::NonCallable);
    }
}

fn analyze_and_update_non_intrinsic_callable_compute_props<'a>(
    id: StoreItemId,
    _callable: &CallableDecl,
    input_params: impl Iterator<Item = &'a InputParam>,
    _package_store: &PackageStore,
    compute_props: &mut PackageStoreComputeProps,
) {
    // This function is only called when a callable has not already been analyzed.
    if compute_props.find_item(id).is_some() {
        panic!("callable is already analyzed");
    }

    let _var_map = initialize_callable_variables_map(input_params);
    // TODO (cesarzc): Implement.
}

fn analyze_and_update_stmt_compute_props(
    id: StoreStmtId,
    package_store: &PackageStore,
    compute_props: &mut PackageStoreComputeProps,
) {
    // If the item has already been analyzed, there's nothing left to do.
    if compute_props.find_stmt(id).is_some() {
        return;
    }

    let _stmt = package_store.get_stmt(id);
    // TODO (cesarzc): Implement.
}
