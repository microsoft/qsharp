// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    common::{
        derive_callable_input_params, derive_specialization_input_params, initalize_locals_map,
        InputParam, InputParamIndex, LocalsMap,
    },
    cycle_detection::{detect_callables_with_cycles, CycledCallableInfo},
    ApplicationsTable, CallableComputeProperties, ComputeProperties, ComputePropertiesLookup,
    DynamismSource, ItemComputeProperties, PackageStoreComputeProperties, RuntimeFeatureFlags,
};
use qsc_fir::fir::{BlockId, ExprId, StmtId, StoreBlockId};
use qsc_fir::{
    fir::{
        CallableDecl, CallableImpl, CallableKind, Global, NodeId, PackageId, PackageStore,
        PackageStoreLookup, SpecDecl, StoreItemId, StoreStmtId,
    },
    ty::{Prim, Ty},
};
use rustc_hash::FxHashMap;

/// An instance of a callable application.
#[derive(Debug, Default)]
struct ApplicationInstance {
    // TODO (cesarzc): document field.
    pub locals_map: LocalsMap,
    // TODO (cesarzc): document field.
    pub dynamic_scopes_stack: Vec<ExprId>,
    // TODO (cesarzc): document field.
    pub compute_properties: ApplicationInstanceComputeProperties,
}

impl ApplicationInstance {
    fn new(input_params: &Vec<InputParam>, dynamic_param_index: InputParamIndex) -> Self {
        let locals_map = initalize_locals_map(input_params, Some(dynamic_param_index));
        Self {
            locals_map,
            dynamic_scopes_stack: Vec::new(),
            compute_properties: ApplicationInstanceComputeProperties::default(),
        }
    }
}

/// The compute properties of a callable application instance.
#[derive(Debug, Default)]
struct ApplicationInstanceComputeProperties {
    pub blocks: FxHashMap<BlockId, ComputeProperties>,
    pub stmts: FxHashMap<StmtId, ComputeProperties>,
    pub exprs: FxHashMap<ExprId, ComputeProperties>,
}

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
    _input_params: &Vec<InputParam>,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) {
    // This function is only called when a block has not already been analyzed.
    if package_store_compute_properties.find_block(id).is_some() {
        panic!("block is already analyzed");
    }

    let _block = package_store.get_block(id);
    // TODO (cesarzc): implement properly.
    package_store_compute_properties.insert_block(
        id,
        ApplicationsTable {
            inherent_properties: ComputeProperties::default(),
            dynamic_params_properties: Vec::new(),
        },
    );
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

    // Analyze the callable depending on its type.
    let input_params = derive_callable_input_params(
        callable,
        &package_store
            .get(id.package)
            .expect("package should exist")
            .pats,
    );
    match callable.implementation {
        CallableImpl::Intrinsic => analyze_intrinsic_callable_compute_properties(
            id,
            callable,
            &input_params,
            package_store_compute_properties,
        ),
        CallableImpl::Spec(_) => analyze_non_intrinsic_callable_compute_properties(
            id,
            callable,
            &input_params,
            package_store,
            package_store_compute_properties,
        ),
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

    // Create compute properties differently depending on whether it is a function or an operation.
    let input_params = derive_callable_input_params(
        callable,
        &package_store
            .get(id.package)
            .expect("package should exist")
            .pats,
    );
    let callable_compute_properties = match &callable.kind {
        CallableKind::Function => create_cycled_function_compute_properties(
            callable,
            cycled_callable_info,
            input_params.len(),
        ),
        CallableKind::Operation => create_cycled_operation_compute_properties(
            id,
            callable,
            cycled_callable_info,
            &input_params,
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

fn analyze_intrinsic_callable_compute_properties(
    id: StoreItemId,
    callable: &CallableDecl,
    input_params: &Vec<InputParam>,
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

fn analyze_non_intrinsic_callable_compute_properties(
    id: StoreItemId,
    callable: &CallableDecl,
    input_params: &Vec<InputParam>,
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
    let body = create_specialization_applications_table(
        id,
        &implementation.body,
        input_params,
        package_store,
        package_store_compute_properties,
    );
    let adj = implementation.adj.as_ref().map(|specialization| {
        create_specialization_applications_table(
            id,
            specialization,
            input_params,
            package_store,
            package_store_compute_properties,
        )
    });
    let ctl = implementation.ctl.as_ref().map(|specialization| {
        create_specialization_applications_table(
            id,
            specialization,
            input_params,
            package_store,
            package_store_compute_properties,
        )
    });
    let ctl_adj = implementation.ctl_adj.as_ref().map(|specialization| {
        create_specialization_applications_table(
            id,
            specialization,
            input_params,
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

fn create_cycled_function_compute_properties(
    callable: &CallableDecl,
    cycled_callable_info: &CycledCallableInfo,
    input_params_count: usize,
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

    // Since functions are classically pure, they inherently do not use any runtime feature nor represent a source of
    // dynamism.
    let inherent_properties = ComputeProperties {
        runtime_features: RuntimeFeatureFlags::empty(),
        dynamism_sources: Vec::new(),
    };

    // Create compute properties for each dynamic parameter.
    let mut dynamic_params_properties = Vec::new();
    for _ in 0..input_params_count {
        // If any parameter is dynamic, then we assume a function with cycles is a a source of dynamism if its output type
        // is non-unit.
        let dynamism_sources = if callable.output == Ty::UNIT {
            Vec::new()
        } else {
            vec![DynamismSource::Assumed]
        };

        // Since convert functions can be called with dynamic parameters, we assume that all capabilities are required
        // for any dynamic parameter. The `CycledFunctionWithDynamicArg` feature flag encompasses this.
        let compute_properties = ComputeProperties {
            runtime_features: RuntimeFeatureFlags::CycledFunctionApplicationUsesDynamicArg,
            dynamism_sources,
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

fn create_cycled_operation_compute_properties(
    callable_id: StoreItemId,
    callable: &CallableDecl,
    cycled_callable_info: &CycledCallableInfo,
    input_params: &Vec<InputParam>,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) -> CallableComputeProperties {
    // This function is not meant for intrinsics.
    let CallableImpl::Spec(spec_impl) = &callable.implementation else {
        panic!("a non-intrinsic callable is expected");
    };

    // Create the compute properties for each one of the specializations.
    // When a specialization has call cycles, assume its properties. Otherwise, create the specialization applications
    // table the same way we would do it for any other specialization.
    let mut create_specialization_applications_table_internal = |specialization, is_spec_cycled| {
        if is_spec_cycled {
            create_cycled_operation_specialization_applications_table(
                specialization,
                input_params.len(),
                &callable.output,
            )
        } else {
            create_specialization_applications_table(
                callable_id,
                specialization,
                input_params,
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
    specialization: &SpecDecl,
    callable_input_params_count: usize,
    output_type: &Ty,
) -> ApplicationsTable {
    // Since operations can allocate and measure qubits freely, we assume it requires all capabilities (encompassed by
    // the `CycledOperationSpecialization` runtime feature) and that they are a source of dynamism if they have a
    // non-unit output.
    let dynamism_sources = if *output_type == Ty::UNIT {
        Vec::new()
    } else {
        vec![DynamismSource::Assumed]
    };
    let compute_properties = ComputeProperties {
        runtime_features: RuntimeFeatureFlags::CycledOperationSpecializationApplication,
        dynamism_sources,
    };

    // Create compute properties for each dynamic parameter. These compute properties are the same than the inherent
    // properties.
    let mut dynamic_params_properties = Vec::new();
    let specialization_input_params_count = if specialization.input.is_some() {
        callable_input_params_count + 1
    } else {
        callable_input_params_count
    };
    for _ in 0..specialization_input_params_count {
        dynamic_params_properties.push(compute_properties.clone());
    }

    // Finally, create the applications table.
    ApplicationsTable {
        inherent_properties: compute_properties,
        dynamic_params_properties,
    }
}

fn create_intrinsic_callable_compute_properties(
    callable: &CallableDecl,
    input_params: &Vec<InputParam>,
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

fn create_intrinsic_function_compute_properties(
    callable: &CallableDecl,
    input_params: &Vec<InputParam>,
) -> CallableComputeProperties {
    assert!(matches!(callable.kind, CallableKind::Function));

    // Functions are purely classical, so no runtime features are needed and cannot be an inherent dynamism source.
    let inherent_properties = ComputeProperties {
        runtime_features: RuntimeFeatureFlags::empty(),
        dynamism_sources: Vec::new(),
    };

    // Calculate the properties for all parameters.
    let mut dynamic_params_properties = Vec::new();
    for param in input_params {
        // For intrinsic functions, we assume any parameter can contribute to the output, so if any parameter is dynamic
        // the output of the function is dynamic. Therefore, for all dynamic parameters, if the function's output is
        // non-unit:
        // - It becomes a source of dynamism.
        // - The output type contributes to the runtime features used by the function.
        let (dynamism_sources, mut runtime_features) = if callable.output == Ty::UNIT {
            (Vec::new(), RuntimeFeatureFlags::empty())
        } else {
            (
                vec![DynamismSource::Intrinsic],
                derive_intrinsic_runtime_features_from_type(&callable.output),
            )
        };

        // When a parameter is binded to a dynamic value, its type contributes to the runtime features used by the
        // function.
        runtime_features |= derive_intrinsic_runtime_features_from_type(&param.ty);
        let param_compute_properties = ComputeProperties {
            runtime_features,
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

fn create_instrinsic_operation_compute_properties(
    callable: &CallableDecl,
    input_params: &Vec<InputParam>,
) -> CallableComputeProperties {
    assert!(matches!(callable.kind, CallableKind::Operation));

    // Intrinsic operations inherently use runtime features if their output is not `Unit`, `Qubit` or `Result`, and
    // these runtime features are derived from the output type.
    let runtime_features = if callable.output == Ty::UNIT
        || callable.output == Ty::Prim(Prim::Qubit)
        || callable.output == Ty::Prim(Prim::Result)
    {
        RuntimeFeatureFlags::empty()
    } else {
        derive_intrinsic_runtime_features_from_type(&callable.output)
    };

    // Intrinsic are an inherent source of dynamism if their output is not `Unit` or `Qubit`.
    let dynamism_sources =
        if callable.output == Ty::UNIT || callable.output == Ty::Prim(Prim::Qubit) {
            Vec::new()
        } else {
            vec![DynamismSource::Intrinsic]
        };

    // Build the inherent properties.
    let inherent_properties = ComputeProperties {
        runtime_features,
        dynamism_sources,
    };

    // Calculate the properties for all dynamic parameters.
    let mut dynamic_params_properties = Vec::new();
    for param in input_params {
        // For intrinsic operations, we assume any parameter can contribute to the output, so if any parameter is
        // dynamic the output of the operation is dynamic. Therefore, this operation becomes a source of dynamism for
        // all dynamic params if its output is not `Unit`.
        let dynamism_sources = if callable.output == Ty::UNIT {
            Vec::new()
        } else {
            vec![DynamismSource::Intrinsic]
        };

        // When a parameter is binded to a dynamic value, its runtime features depend on the parameter type.
        let param_compute_properties = ComputeProperties {
            runtime_features: derive_intrinsic_runtime_features_from_type(&param.ty),
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
    callable_input_params: &Vec<InputParam>,
    package_store: &PackageStore,
    package_store_compute_properties: &mut PackageStoreComputeProperties,
) -> ApplicationsTable {
    // We expand the input map for controlled specializations, which have its own additional input (the control qubit
    // register).
    let package_patterns = &package_store
        .get(callable_id.package)
        .expect("package should exist")
        .pats;

    // Derive the input parameters for the specialization, which can be different from the callable input parameters
    // if the specialization has its own input.
    let specialization_input_params =
        derive_specialization_input_params(specialization, callable_input_params, package_patterns);

    // Then we analyze the block which implements the specialization.
    let block_id = (callable_id.package, specialization.block).into();
    analyze_block(
        block_id,
        &specialization_input_params,
        package_store,
        package_store_compute_properties,
    );

    // Finally, we get the applications table of the analyzed block, which also represents the application table of the
    // specialization.
    let block_applications_table = package_store_compute_properties.get_block(block_id);
    block_applications_table.clone()
}

fn derive_intrinsic_runtime_features_from_type(ty: &Ty) -> RuntimeFeatureFlags {
    fn intrinsic_runtime_features_from_primitive_type(prim: &Prim) -> RuntimeFeatureFlags {
        match prim {
            Prim::BigInt => RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicBigInt,
            Prim::Bool => RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicBool,
            Prim::Double => RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicDouble,
            Prim::Int => RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicInt,
            Prim::Pauli => RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicPauli,
            Prim::Qubit => RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicQubit,
            Prim::Range | Prim::RangeFrom | Prim::RangeTo | Prim::RangeFull => {
                RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicRange
            }
            Prim::Result => RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicResult,
            Prim::String => RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicString,
        }
    }

    fn intrinsic_runtime_features_from_tuple(tuple: &Vec<Ty>) -> RuntimeFeatureFlags {
        let mut runtime_features = if tuple.is_empty() {
            RuntimeFeatureFlags::empty()
        } else {
            RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicTuple
        };
        for item_type in tuple {
            runtime_features |= derive_intrinsic_runtime_features_from_type(item_type);
        }
        runtime_features
    }

    match ty {
        Ty::Array(_) => RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicArray,
        Ty::Arrow(arrow) => match arrow.kind {
            CallableKind::Function => {
                RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicArrowFunction
            }
            CallableKind::Operation => {
                RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicArrowOperation
            }
        },
        Ty::Prim(prim) => intrinsic_runtime_features_from_primitive_type(prim),
        Ty::Tuple(tuple) => intrinsic_runtime_features_from_tuple(tuple),
        Ty::Udt(_) => RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicUdt,
        _ => panic!("unexpected type"),
    }
}
