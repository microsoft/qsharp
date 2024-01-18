// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::data_structures::{
    derive_callable_input_elements, derive_callable_input_params,
    initialize_callable_variables_map, CallableInputElement, CallableInputElementKind, InputParam,
};
use crate::fir_extensions::CallableDeclExtension;
use crate::{
    ComputePropertiesLookup, ItemComputeProperties, PackageStoreComputeProperties,
    PatComputeProperties,
};
use qsc_fir::fir::{
    CallableDecl, Global, PackageId, PackageStore, PackageStoreLookup, StoreItemId, StoreStmtId,
};

pub fn analyze_package_compute_properties(
    id: PackageId,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // Clear current compute properties of the package to make sure we are performing a coherent analysis on it.
    let package_compute_properties = package_store_compute_properties
        .get_mut(id)
        .expect("package compute properties should exist");
    package_compute_properties.clear();
    let package = package_store.get(id).expect("package should exist");

    // Analyze all items in the package.
    for (item_id, _) in &package.items {
        analyze_item_compute_properties(
            (id, item_id).into(),
            package_store,
            package_store_compute_properties,
        );
    }

    // Analyze all statements in the package.
    for (stmt_id, _) in &package.stmts {
        analyze_statement_compute_properties(
            (id, stmt_id).into(),
            package_store,
            package_store_compute_properties,
        );
    }
}

fn analyze_callable_compute_properties(
    id: StoreItemId,
    callable: &CallableDecl,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // This function is only called when a callable has not already been analyzed.
    if package_store_compute_properties.find_item(id).is_some() {
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
    analyze_callable_input_elements(id, input_elements.iter(), package_store_compute_properties);

    // Analyze the callable depending on its type.
    let input_params = derive_callable_input_params(input_elements.iter());
    if callable.is_intrinsic() {
        analyze_intrinsic_callable_compute_properties(
            id,
            callable,
            input_params.iter(),
            package_store,
            package_store_compute_properties,
        );
    } else {
        analyze_non_intrinsic_callable_compute_properties(
            id,
            callable,
            input_params.iter(),
            package_store,
            package_store_compute_properties,
        );
    }
}

fn analyze_callable_input_elements<'a>(
    callable_id: StoreItemId,
    input_elements: impl Iterator<Item = &'a CallableInputElement>,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // This function is only called when a callable has not already been analyzed.
    if package_store_compute_properties
        .find_item(callable_id)
        .is_some()
    {
        panic!("callable is already analyzed");
    }

    for element in input_elements {
        match &element.kind {
            CallableInputElementKind::Discard => package_store_compute_properties.insert_pat(
                (callable_id.package, element.pat).into(),
                PatComputeProperties::InputParamDiscard,
            ),
            CallableInputElementKind::Node(node_id) => package_store_compute_properties.insert_pat(
                (callable_id.package, element.pat).into(),
                PatComputeProperties::InputParamNode(*node_id),
            ),
            CallableInputElementKind::Tuple(tuple_pats) => package_store_compute_properties
                .insert_pat(
                    (callable_id.package, element.pat).into(),
                    PatComputeProperties::InputParamTuple(tuple_pats.clone()),
                ),
        }
    }
}

fn analyze_intrinsic_callable_compute_properties<'a>(
    id: StoreItemId,
    _callable: &CallableDecl,
    _input_params: impl Iterator<Item = &'a InputParam>,
    _package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // This function is only called when a callable has not already been analyzed.
    if package_store_compute_properties.find_item(id).is_some() {
        panic!("callable is already analyzed");
    }

    // TODO (cesarzc): Implement.
}

fn analyze_item_compute_properties(
    id: StoreItemId,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // If the item has already been analyzed, there's nothing left to do.
    if package_store_compute_properties.find_item(id).is_some() {
        return;
    }

    if let Some(Global::Callable(callable)) = package_store.get_global(id) {
        analyze_callable_compute_properties(
            id,
            callable,
            package_store,
            package_store_compute_properties,
        );
    } else {
        package_store_compute_properties.insert_item(id, ItemComputeProperties::NonCallable);
    }
}

fn analyze_non_intrinsic_callable_compute_properties<'a>(
    id: StoreItemId,
    _callable: &CallableDecl,
    input_params: impl Iterator<Item = &'a InputParam>,
    _package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // This function is only called when a callable has not already been analyzed.
    if package_store_compute_properties.find_item(id).is_some() {
        panic!("callable is already analyzed");
    }

    let _var_map = initialize_callable_variables_map(input_params);
    // TODO (cesarzc): Implement.
}

fn analyze_statement_compute_properties(
    id: StoreStmtId,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // If the item has already been analyzed, there's nothing left to do.
    if package_store_compute_properties.find_stmt(id).is_some() {
        return;
    }

    let _stmt = package_store.get_stmt(id);
    // TODO (cesarzc): Implement.
}
