// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::nodes::{FlattenedInputParamsElmnts, InputParamElmnt};
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

    // First, set the compute properties of all the patterns that make the input parameters.
    let input_params_elmnts = FlattenedInputParamsElmnts::from_callable(
        callable,
        &package_store
            .get(id.package)
            .expect("package should exist")
            .pats,
    );
    analyze_and_update_callable_input_params(id, &input_params_elmnts, compute_props);
}

fn analyze_and_update_callable_input_params(
    callable_id: StoreItemId,
    input_params_elmnts: &FlattenedInputParamsElmnts,
    compute_props: &mut PackageStoreComputeProps,
) {
    for elmnt in input_params_elmnts.iter() {
        match elmnt {
            InputParamElmnt::Discard(pat_id) => compute_props.insert_pat(
                (callable_id.package, *pat_id).into(),
                PatComputeProps::InputParamDiscard,
            ),
            InputParamElmnt::Node(pat_id, node_id) => compute_props.insert_pat(
                (callable_id.package, *pat_id).into(),
                PatComputeProps::InputParamNode(*node_id),
            ),
            InputParamElmnt::Tuple(pat_id, tuple_pats) => compute_props.insert_pat(
                (callable_id.package, *pat_id).into(),
                PatComputeProps::InputParamTuple(tuple_pats.clone()),
            ),
        }
    }
}

fn analyze_and_update_item_compute_props(
    id: StoreItemId,
    package_store: &PackageStore,
    compute_props: &mut PackageStoreComputeProps,
) {
    // If the item has already been analyzed, there's nothing else left to do.
    if compute_props.find_item(id).is_some() {
        return;
    }

    if let Some(Global::Callable(callable)) = package_store.get_global(id) {
        analyze_and_update_callable_compute_props(id, callable, package_store, compute_props);
    } else {
        compute_props.insert_item(id, ItemComputeProps::NonCallable);
    }
}

fn analyze_and_update_stmt_compute_props(
    _id: StoreStmtId,
    _package_store: &PackageStore,
    _compute_props: &mut PackageStoreComputeProps,
) {
}
