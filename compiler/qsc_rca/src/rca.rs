// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::data_structures::{
    derive_callable_input_elements, derive_callable_input_params,
    initialize_callable_variables_map, CallableInputElement, CallableInputElementKind, InputParam,
};
use crate::{
    ApplicationsTable, CallableComputeProperties, ComputeProperties, ComputePropertiesLookup,
    ItemComputeProperties, PackageStoreComputeProperties, PatComputeProperties, QuantumSource,
};
use qsc_fir::{
    fir::{
        CallableDecl, CallableImpl, CallableKind, Global, PackageId, PackageStore,
        PackageStoreLookup, StoreItemId, StoreStmtId,
    },
    ty::{Prim, Ty},
};
use qsc_frontend::compile::RuntimeCapabilityFlags;

pub fn analyze_package(
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
        analyze_item(
            (id, item_id).into(),
            package_store,
            package_store_compute_properties,
        );
    }

    // Analyze all statements in the package.
    for (stmt_id, _) in &package.stmts {
        analyze_statement(
            (id, stmt_id).into(),
            package_store,
            package_store_compute_properties,
        );
    }
}

fn analyze_callable(
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
    match callable.implementation {
        CallableImpl::Intrinsic => analyze_intrinsic_callable_compute_properties(
            id,
            callable,
            input_params.iter(),
            package_store,
            package_store_compute_properties,
        ),
        CallableImpl::Spec(_) => analyze_non_intrinsic_callable_compute_properties(
            id,
            callable,
            input_params.iter(),
            package_store,
            package_store_compute_properties,
        ),
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
    callable: &CallableDecl,
    input_params: impl Iterator<Item = &'a InputParam>,
    _package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // This function is only called when a callable has not already been analyzed.
    if package_store_compute_properties.find_item(id).is_some() {
        panic!("callable is already analyzed");
    }

    let callable_compute_properties =
        create_intrinsic_callable_compute_properties(callable, input_params);
    package_store_compute_properties.insert_item(
        id,
        ItemComputeProperties::Callable(callable_compute_properties),
    );
}

fn analyze_item(
    id: StoreItemId,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // If the item has already been analyzed, there's nothing left to do.
    if package_store_compute_properties.find_item(id).is_some() {
        return;
    }

    if let Some(Global::Callable(callable)) = package_store.get_global(id) {
        analyze_callable(
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

fn analyze_statement(
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

fn create_intrinsic_callable_compute_properties<'a>(
    callable: &CallableDecl,
    input_params: impl Iterator<Item = &'a InputParam>,
) -> CallableComputeProperties {
    assert!(matches!(callable.implementation, CallableImpl::Intrinsic));
    match callable.kind {
        CallableKind::Function => {
            create_intrinsic_function_compute_properties(callable, input_params)
        }
        CallableKind::Operation => {
            create_instrinsic_operation_compute_properties(callable, input_params)
        }
    }
}

fn create_intrinsic_function_compute_properties<'a>(
    callable: &CallableDecl,
    input_params: impl Iterator<Item = &'a InputParam>,
) -> CallableComputeProperties {
    assert!(matches!(callable.kind, CallableKind::Function));

    // Functions are purely classical, so no runtime capabilities are needed and cannot be an inherent quantum
    // source.
    let inherent_properties = ComputeProperties {
        runtime_capabilities: RuntimeCapabilityFlags::empty(),
        quantum_source: None,
    };

    // Calculate the properties for all parameters.
    let mut dynamic_params_properties = Vec::new();
    for param in input_params {
        // For each parameter, its properties when it is used as a dynamic argument in a particular application depend
        // on the parameter type.
        let param_runtime_capabilities = derive_runtime_capabilities_from_type(&param.ty);

        // For intrinsic functions, we assume any parameter can contribute to the output, so if any parameter is dynamic
        // the output of the function is dynamic. Therefore, this function becomes a quantum source for all dynamic
        // params if its output is non-unit.
        let quantum_source = if callable.output == Ty::UNIT {
            None
        } else {
            Some(QuantumSource::Intrinsic)
        };
        let param_compute_properties = ComputeProperties {
            runtime_capabilities: param_runtime_capabilities,
            quantum_source,
        };
        dynamic_params_properties.push(param_compute_properties);
    }

    // Construct the callable compute properties.
    let body = ApplicationsTable {
        inherent_properties,
        dynamic_params_properties,
    };
    CallableComputeProperties {
        body,
        adj: None,
        ctl: None,
        ctl_adj: None,
    }
}

fn create_instrinsic_operation_compute_properties<'a>(
    callable: &CallableDecl,
    input_params: impl Iterator<Item = &'a InputParam>,
) -> CallableComputeProperties {
    assert!(matches!(callable.kind, CallableKind::Operation));

    // For intrinsic operations, they inherently do not require any runtime capabilities and they are a quantum source
    // if their output is not qubit nor unit.
    let quantum_source = if callable.output == Ty::Prim(Prim::Qubit) || callable.output == Ty::UNIT
    {
        None
    } else {
        Some(QuantumSource::Intrinsic)
    };
    let inherent_properties = ComputeProperties {
        runtime_capabilities: RuntimeCapabilityFlags::empty(),
        quantum_source,
    };

    // Calculate the properties for all parameters.
    let mut dynamic_params_properties = Vec::new();
    for param in input_params {
        // For each parameter, its properties when it is used as a dynamic argument in a particular application depend
        // on the parameter type.
        let param_runtime_capabilities = derive_runtime_capabilities_from_type(&param.ty);

        // For intrinsic operations, we assume any parameter can contribute to the output, so if any parameter is
        // dynamic the output of the operation is dynamic. Therefore, this operation becomes a quantum source for all
        // dynamic params if its output is non-unit.
        let quantum_source = if callable.output == Ty::UNIT {
            None
        } else {
            Some(QuantumSource::Intrinsic)
        };
        let param_compute_properties = ComputeProperties {
            runtime_capabilities: param_runtime_capabilities,
            quantum_source,
        };
        dynamic_params_properties.push(param_compute_properties);
    }

    // Construct the callable compute properties.
    let body = ApplicationsTable {
        inherent_properties,
        dynamic_params_properties,
    };
    CallableComputeProperties {
        body,
        adj: None,
        ctl: None,
        ctl_adj: None,
    }
}

fn derive_runtime_capabilities_from_type(ty: &Ty) -> RuntimeCapabilityFlags {
    fn derive_runtime_capabilities_from_primitive(prim: &Prim) -> RuntimeCapabilityFlags {
        match prim {
            Prim::BigInt => RuntimeCapabilityFlags::HigherLevelConstructs,
            Prim::Bool => RuntimeCapabilityFlags::ForwardBranching,
            Prim::Double => RuntimeCapabilityFlags::FloatingPointComputations,
            Prim::Int => RuntimeCapabilityFlags::IntegerComputations,
            Prim::Pauli => RuntimeCapabilityFlags::IntegerComputations,
            Prim::Qubit => RuntimeCapabilityFlags::HigherLevelConstructs,
            Prim::Range | Prim::RangeFrom | Prim::RangeTo | Prim::RangeFull => {
                RuntimeCapabilityFlags::IntegerComputations
            }
            Prim::Result => RuntimeCapabilityFlags::ForwardBranching,
            Prim::String => RuntimeCapabilityFlags::HigherLevelConstructs,
        }
    }

    fn derive_runtime_capabilities_from_tuple(tuple: &Vec<Ty>) -> RuntimeCapabilityFlags {
        let mut runtime_capabilities = RuntimeCapabilityFlags::empty();
        for item_type in tuple {
            let item_runtime_capabilities = derive_runtime_capabilities_from_type(item_type);
            runtime_capabilities |= item_runtime_capabilities;
        }
        runtime_capabilities
    }

    match ty {
        // N.B. Derived array runtime capabilities can be more nuanced by taking into account the contained type.
        Ty::Array(_) => RuntimeCapabilityFlags::HigherLevelConstructs,
        // N.B. Derived array runtime capabilities can be more nuanced by taking into account the input and output
        // types.
        Ty::Arrow(_) => RuntimeCapabilityFlags::HigherLevelConstructs,
        Ty::Prim(prim) => derive_runtime_capabilities_from_primitive(prim),
        Ty::Tuple(tuple) => derive_runtime_capabilities_from_tuple(tuple),
        // N.B. Derived UDT runtime capabilities can be more nuanced by taking into account the type of each UDT
        // item.
        Ty::Udt(_) => RuntimeCapabilityFlags::HigherLevelConstructs,
        _ => panic!("Unexpected type"),
    }
}
