// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    cycle_detection::{detect_callables_with_cycles, CycledCallableInfo},
    data_structures::{
        derive_callable_input_elements, derive_callable_input_map, derive_callable_input_params,
        CallableInputElement, CallableInputElementKind, CallableVariable, CallableVariableKind,
        InputParam, InputParamIndex,
    },
    CallableElementComputeProperties,
    {
        ApplicationsTable, CallableComputeProperties, ComputeProperties, ComputePropertiesLookup,
        DynamismSource, ItemComputeProperties, PackageStoreComputeProperties, PatComputeProperties,
    },
};
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{Pat, PatId, PatKind, StoreBlockId};
use qsc_fir::{
    fir::{
        CallableDecl, CallableImpl, CallableKind, Global, NodeId, PackageId, PackageStore,
        PackageStoreLookup, SpecDecl, StoreItemId, StoreStmtId,
    },
    ty::{Prim, Ty},
};
use qsc_frontend::compile::RuntimeCapabilityFlags;
use rustc_hash::FxHashMap;

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

    // First, analyze all callables that have cycles. We need to do this because callables with cycles make the main
    // runtime capabilities analysis (RCA) algorithm loop forever. If we have use a heuristic to analyze the compute
    // properties of callables with cycles, then the RCA algorithm can use those properties safely without getting stuck
    // in a infinite loop.
    let callables_with_cycles = detect_callables_with_cycles(id, package);
    for cycled_callable in callables_with_cycles {
        analyze_callable_with_cycles(
            (id, cycled_callable.id).into(),
            &cycled_callable,
            package_store,
            package_store_compute_properties,
        )
    }

    // Once all callables with cycles have been analyzed, it is safe to continue analyzing all the other items.
    for (item_id, _) in &package.items {
        analyze_item(
            (id, item_id).into(),
            package_store,
            package_store_compute_properties,
        );
    }

    // Finally, analyze the statements in the package. By this point, only top-level statements remain unanalyzed.
    for (stmt_id, _) in &package.stmts {
        analyze_statement(
            (id, stmt_id).into(),
            package_store,
            package_store_compute_properties,
        );
    }
}

fn analyze_block(
    id: StoreBlockId,
    input_map: &FxHashMap<NodeId, CallableVariable>,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // This function is only called when a block has not already been analyzed.
    if package_store_compute_properties.find_block(id).is_some() {
        panic!("block is already analyzed");
    }

    let _variables_map = input_map.clone();
    let _block = package_store.get_block(id);
    // TODO (cesarzc): implement properly.
    package_store_compute_properties.insert_block(id, CallableElementComputeProperties::Invalid);
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

fn analyze_callable_with_cycles(
    id: StoreItemId,
    cycled_callable_info: &CycledCallableInfo,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // This function is only called when a callable has not already been analyzed.
    if package_store_compute_properties.find_item(id).is_some() {
        panic!("callable is already analyzed");
    }

    let Some(Global::Callable(callable)) = package_store.get_global(id) else {
        panic!("item should be a callable")
    };

    // First, set the compute properties of all the patterns that are part of the callable input.
    let input_elements = derive_callable_input_elements(
        callable,
        &package_store
            .get(id.package)
            .expect("package should exist")
            .pats,
    );
    analyze_callable_input_elements(id, input_elements.iter(), package_store_compute_properties);

    // Create compute properties differently depending on whether it is a function or an operation.
    let input_params = derive_callable_input_params(input_elements.iter());
    let callable_compute_properties = match &callable.kind {
        CallableKind::Function => create_cycled_function_compute_properties(
            callable,
            cycled_callable_info,
            input_params.iter(),
        ),
        CallableKind::Operation => create_cycled_operation_compute_properties(
            id,
            callable,
            cycled_callable_info,
            input_params.iter(),
            package_store,
            package_store_compute_properties,
        ),
    };

    // Finally, insert the callable compute properties.
    package_store_compute_properties.insert_item(
        id,
        ItemComputeProperties::Callable(callable_compute_properties),
    );
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
    callable: &CallableDecl,
    input_params: impl Iterator<Item = &'a InputParam>,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // This function is only called when a callable has not already been analyzed.
    if package_store_compute_properties.find_item(id).is_some() {
        panic!("callable is already analyzed");
    }

    // This function is not meant for instrinsics.
    let CallableImpl::Spec(implementation) = &callable.implementation else {
        panic!("callable is assumed to have a specialized implementation");
    };

    // Analyze each one of the specializations.
    let input_map = derive_callable_input_map(input_params);
    let body = create_specialization_applications_table(
        id,
        &implementation.body,
        &input_map,
        package_store,
        package_store_compute_properties,
    );
    let adj = implementation.adj.as_ref().map(|specialization| {
        create_specialization_applications_table(
            id,
            specialization,
            &input_map,
            package_store,
            package_store_compute_properties,
        )
    });
    let ctl = implementation.ctl.as_ref().map(|specialization| {
        create_specialization_applications_table(
            id,
            specialization,
            &input_map,
            package_store,
            package_store_compute_properties,
        )
    });
    let ctl_adj = implementation.ctl_adj.as_ref().map(|specialization| {
        create_specialization_applications_table(
            id,
            specialization,
            &input_map,
            package_store,
            package_store_compute_properties,
        )
    });

    let callable_compute_properties = CallableComputeProperties {
        body,
        adj,
        ctl,
        ctl_adj,
    };
    package_store_compute_properties.insert_item(
        id,
        ItemComputeProperties::Callable(callable_compute_properties),
    );
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

fn create_cycled_function_compute_properties<'a>(
    callable: &CallableDecl,
    cycled_callable_info: &CycledCallableInfo,
    input_params: impl Iterator<Item = &'a InputParam>,
) -> CallableComputeProperties {
    // This function is not meant for intrinsics.
    let CallableImpl::Spec(spec_impl) = &callable.implementation else {
        panic!("a non-intrinsic callable is expected");
    };

    // The only specialization that functions have is the body.
    assert!(spec_impl.adj.is_none() && spec_impl.ctl.is_none() && spec_impl.ctl_adj.is_none());
    assert!(spec_impl.body.input.is_none());
    assert!(
        cycled_callable_info.is_adj_cycled.is_none()
            && cycled_callable_info.is_ctl_cycled.is_none()
            && cycled_callable_info.is_ctl_adj_cycled.is_none()
    );
    assert!(cycled_callable_info.is_body_cycled);

    // Since functions are classically pure, they inherently do not require any capabilities nor represent a quantum
    // source.
    let inherent_properties = ComputeProperties {
        runtime_capabilities: RuntimeCapabilityFlags::empty(),
        dynamism_sources: Vec::new(),
    };

    // If any parameter is dynamic, then we assume a function with cycles is a a source of dynamism if its output type
    // is non-unit.
    let dynamism_sources = if callable.output == Ty::UNIT {
        Vec::new()
    } else {
        vec![DynamismSource::Assumed]
    };

    // Create compute properties for each dynamic parameter.
    let mut dynamic_params_properties = Vec::new();
    for _ in input_params {
        // Since convert functions can be used on dynamic parameters, we assume that all capabilities are required for any
        // dynamic parameter.
        let compute_properties = ComputeProperties {
            runtime_capabilities: RuntimeCapabilityFlags::all(),
            dynamism_sources: dynamism_sources.clone(),
        };
        dynamic_params_properties.push(compute_properties);
    }

    let body = ApplicationsTable {
        inherent_properties,
        dynamic_params_properties,
    };

    // Create the callable compute properties.
    CallableComputeProperties {
        body,
        adj: None,
        ctl: None,
        ctl_adj: None,
    }
}

fn create_cycled_operation_compute_properties<'a>(
    callable_id: StoreItemId,
    callable: &CallableDecl,
    cycled_callable_info: &CycledCallableInfo,
    input_params: impl Iterator<Item = &'a InputParam>,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) -> CallableComputeProperties {
    // This function is not meant for intrinsics.
    let CallableImpl::Spec(spec_impl) = &callable.implementation else {
        panic!("a non-intrinsic callable is expected");
    };

    // Create the compute properties for each one of the specializations.
    // When a specialization has call cycles, assume its properties. Otherwise, create the specialization applications
    // table the same way that for any other specialization.
    let input_map = derive_callable_input_map(input_params);
    let mut create_specialization_applications_table_internal = |specialization, is_spec_cycled| {
        if is_spec_cycled {
            create_cycled_operation_specialization_applications_table(
                callable_id,
                specialization,
                &input_map,
                &callable.output,
                package_store,
            )
        } else {
            create_specialization_applications_table(
                callable_id,
                specialization,
                &input_map,
                package_store,
                package_store_compute_properties,
            )
        }
    };
    let body = create_specialization_applications_table_internal(
        &spec_impl.body,
        cycled_callable_info.is_body_cycled,
    );
    let adj = spec_impl.adj.as_ref().map(|specialization| {
        create_specialization_applications_table_internal(
            specialization,
            cycled_callable_info
                .is_adj_cycled
                .expect("is_adj_cycled should be some"),
        )
    });
    let ctl = spec_impl.ctl.as_ref().map(|specialization| {
        create_specialization_applications_table_internal(
            specialization,
            cycled_callable_info
                .is_ctl_cycled
                .expect("is_ctl_cycled should be some"),
        )
    });
    let ctl_adj = spec_impl.ctl_adj.as_ref().map(|specialization| {
        create_specialization_applications_table_internal(
            specialization,
            cycled_callable_info
                .is_ctl_adj_cycled
                .expect("is_ctl_adj_cycled should be some"),
        )
    });

    // Create the callable compute properties.
    CallableComputeProperties {
        body,
        adj,
        ctl,
        ctl_adj,
    }
}

fn create_cycled_operation_specialization_applications_table(
    callable_id: StoreItemId,
    specialization: &SpecDecl,
    callable_input_map: &FxHashMap<NodeId, CallableVariable>,
    output_type: &Ty,
    package_store: &PackageStore,
) -> ApplicationsTable {
    // We expand the input map for controlled specializations, which have its own additional input (the control qubit
    // register).
    let package_patterns = &package_store
        .get(callable_id.package)
        .expect("package should exist")
        .pats;
    let mut input_map =
        create_specialization_input_map(callable_input_map, specialization.input, package_patterns);

    // Since operations can allocate and measure qubits freely, we assume it requires all capabilities and that they are
    // a source of dynamism for all non-unit outputs. These will be both the inherent compute properties and the compute
    // properties for all dynamic parameters.
    let dynamism_sources = if *output_type == Ty::UNIT {
        Vec::new()
    } else {
        vec![DynamismSource::Assumed]
    };
    let compute_properties = ComputeProperties {
        runtime_capabilities: RuntimeCapabilityFlags::all(),
        dynamism_sources,
    };

    // Create compute properties for each dynamic parameter, which are the same than the inherent properties.
    let mut dynamic_params_properties = Vec::new();
    input_map
        .drain()
        .map(|(_, callable_variable)| callable_variable)
        .for_each(|callable_variable| {
            let CallableVariableKind::InputParam(_) = callable_variable.kind else {
                panic!("all callable variables should be input parameters at this point")
            };
            dynamic_params_properties.push(compute_properties.clone())
        });

    ApplicationsTable {
        inherent_properties: compute_properties,
        dynamic_params_properties,
    }
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

    // Functions are purely classical, so no runtime capabilities are needed and cannot be an inherent dynamism source.
    let inherent_properties = ComputeProperties {
        runtime_capabilities: RuntimeCapabilityFlags::empty(),
        dynamism_sources: Vec::new(),
    };

    // Calculate the properties for all parameters.
    let mut dynamic_params_properties = Vec::new();
    for param in input_params {
        // For each parameter, its properties when it is used as a dynamic argument in a particular application depend
        // on the parameter type.
        let param_runtime_capabilities = derive_runtime_capabilities_from_type(&param.ty);

        // For intrinsic functions, we assume any parameter can contribute to the output, so if any parameter is dynamic
        // the output of the function is dynamic. Therefore, this function becomes a source of dynamism for all dynamic
        // params if its output is non-unit.
        let dynamism_sources = if callable.output == Ty::UNIT {
            Vec::new()
        } else {
            vec![DynamismSource::Intrinsic]
        };
        let param_compute_properties = ComputeProperties {
            runtime_capabilities: param_runtime_capabilities,
            dynamism_sources,
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

    // For intrinsic operations, they inherently do not require any runtime capabilities and they are a source of
    // dynamism if their output is not qubit nor unit.
    let dynamism_sources =
        if callable.output == Ty::Prim(Prim::Qubit) || callable.output == Ty::UNIT {
            Vec::new()
        } else {
            vec![DynamismSource::Intrinsic]
        };
    let inherent_properties = ComputeProperties {
        runtime_capabilities: RuntimeCapabilityFlags::empty(),
        dynamism_sources,
    };

    // Calculate the properties for all parameters.
    let mut dynamic_params_properties = Vec::new();
    for param in input_params {
        // For each parameter, its properties when it is used as a dynamic argument in a particular application depend
        // on the parameter type.
        let param_runtime_capabilities = derive_runtime_capabilities_from_type(&param.ty);

        // For intrinsic operations, we assume any parameter can contribute to the output, so if any parameter is
        // dynamic the output of the operation is dynamic. Therefore, this operation becomes a source of dynamism for
        // all dynamic params if its output is non-unit.
        let dynamism_sources = if callable.output == Ty::UNIT {
            Vec::new()
        } else {
            vec![DynamismSource::Intrinsic]
        };
        let param_compute_properties = ComputeProperties {
            runtime_capabilities: param_runtime_capabilities,
            dynamism_sources,
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

fn create_specialization_applications_table(
    callable_id: StoreItemId,
    specialization: &SpecDecl,
    callable_input_map: &FxHashMap<NodeId, CallableVariable>,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) -> ApplicationsTable {
    // We expand the input map for controlled specializations, which have its own additional input (the control qubit
    // register).
    let package_patterns = &package_store
        .get(callable_id.package)
        .expect("package should exist")
        .pats;
    let input_map =
        create_specialization_input_map(callable_input_map, specialization.input, package_patterns);

    // Then we analyze the block.
    let block_id = (callable_id.package, specialization.block).into();
    analyze_block(
        block_id,
        &input_map,
        package_store,
        package_store_compute_properties,
    );

    // Finally, we get the compute properties of the analyzed block and use it to create the application table of the
    // specialization.
    let block_compute_properties = package_store_compute_properties.get_block(block_id);
    match block_compute_properties {
        CallableElementComputeProperties::ApplicationDependent(applications_table) => {
            applications_table.clone()
        }
        CallableElementComputeProperties::ApplicationIndependent(compute_properties) => {
            let inherent_properties = compute_properties.clone();
            let dynamic_params_properties = vec![ComputeProperties::default(); input_map.len()];
            ApplicationsTable {
                inherent_properties,
                dynamic_params_properties,
            }
        }
        // TODO (cesarzc): remove this case.
        CallableElementComputeProperties::Invalid => ApplicationsTable {
            inherent_properties: ComputeProperties::default(),
            dynamic_params_properties: Vec::new(),
        },
    }
}

fn create_specialization_input_map(
    callable_input_map: &FxHashMap<NodeId, CallableVariable>,
    specialization_input: Option<PatId>,
    package_patterns: &IndexMap<PatId, Pat>,
) -> FxHashMap<NodeId, CallableVariable> {
    if let Some(spec_input_pat_id) = specialization_input {
        // If the specialization has its own input, as it is the case for controlled specializations, create a new map
        // with the specialization input as the first parameter..
        let spec_input_pat = package_patterns
            .get(spec_input_pat_id)
            .expect("specialization input pattern should exist");
        let PatKind::Bind(ident) = &spec_input_pat.kind else {
            panic!("a specialization input is expected to be an identifier");
        };
        let spec_input_variable = CallableVariable {
            node: ident.id,
            pat: spec_input_pat_id,
            ty: spec_input_pat.ty.clone(),
            kind: CallableVariableKind::InputParam(InputParamIndex::from(0)),
        };
        let mut input_map = FxHashMap::<NodeId, CallableVariable>::default();
        input_map.insert(spec_input_variable.node, spec_input_variable);
        for (node_id, variable) in callable_input_map {
            let CallableVariableKind::InputParam(input_param_idx) = variable.kind else {
                panic!("all callable input variables should be of the input parameter kind");
            };
            let new_variable = CallableVariable {
                node: variable.node,
                pat: variable.pat,
                ty: variable.ty.clone(),
                // The input param index increases because now the specialization input has the index zero, so we must
                // move the index of all the other parameters by one.
                kind: CallableVariableKind::InputParam(input_param_idx + 1),
            };
            input_map.insert(*node_id, new_variable);
        }

        input_map
    } else {
        // If the specialization does not have its own input, as it is the case for body and adjoint specializations,
        // the input map for the specialization is the same as then one for the callable.
        callable_input_map.clone()
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
