// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{ComputePropsLookup, ItemComputeProps, PackageStoreComputeProps};
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
    _id: StoreItemId,
    _callable: &CallableDecl,
    _package_store: &impl PackageStoreLookup,
    _compute_props: &mut PackageStoreComputeProps,
) {
}

fn analyze_and_update_item_compute_props(
    id: StoreItemId,
    package_store: &impl PackageStoreLookup,
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
    _package_store: &impl PackageStoreLookup,
    _compute_props: &mut PackageStoreComputeProps,
) {
}
