// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::PackageStoreComputeProps;
use qsc_fir::fir::{PackageId, PackageStore, PackageStoreLookup, StoreItemId, StoreStmtId};

pub fn analyze_package_and_update_compute_props(
    id: PackageId,
    package_store: &PackageStore,
    compute_props: &mut PackageStoreComputeProps,
) {
    // Clear current compute properties of the package to make sure we are performing a coherent analysis on it.
    let package_compute_props = compute_props.get_mut(id);
    package_compute_props.clear();
    let package = package_store.get(id).expect("package does not exist");

    // Analyze all items in the package.
    for (item_id, _) in &package.items {
        analyze_item_and_update_compute_props(
            StoreItemId::from(id, item_id),
            package_store,
            compute_props,
        );
    }

    // Analyze all statements in the package.
    for (stmt_id, _) in &package.stmts {
        analyze_stmt_and_update_compute_props(
            StoreStmtId::from(id, stmt_id),
            package_store,
            compute_props,
        );
    }
}

fn analyze_item_and_update_compute_props(
    _id: StoreItemId,
    _package_store: &impl PackageStoreLookup,
    _compute_props: &mut PackageStoreComputeProps,
) {
}

fn analyze_stmt_and_update_compute_props(
    _id: StoreStmtId,
    _package_store: &impl PackageStoreLookup,
    _compute_props: &mut PackageStoreComputeProps,
) {
}
