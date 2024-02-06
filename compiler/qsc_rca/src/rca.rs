// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    common::{
        derive_callable_input_params, derive_specialization_input_params, initalize_locals_map,
        GlobalSpecializationId, InputParam, InputParamIndex, Local, SpecializationKind,
    },
    scaffolding::{ItemScaffolding, PackageStoreScaffolding},
    ApplicationsTable, ComputeProperties, ComputePropertiesLookup, DynamismSource,
    RuntimeFeatureFlags,
};
use qsc_fir::fir::{BlockId, ExprId, NodeId, StmtId, StoreBlockId};
use qsc_fir::{
    fir::{
        CallableDecl, CallableImpl, CallableKind, Global, PackageId, PackageStore,
        PackageStoreLookup, SpecDecl, StoreItemId, StoreStmtId,
    },
    ty::{Prim, Ty},
};
use rustc_hash::{FxHashMap, FxHashSet};

/// An instance of a callable application.
#[derive(Debug, Default)]
struct ApplicationInstance {
    // TODO (cesarzc): document field.
    pub locals_map: FxHashMap<NodeId, LocalComputeProperties>,
    // TODO (cesarzc): document field.
    pub dynamic_scopes_stack: Vec<ExprId>,
    // TODO (cesarzc): document field.
    pub compute_properties: ApplicationInstanceComputeProperties,
}

impl ApplicationInstance {
    fn new(input_params: &Vec<InputParam>, dynamic_param_index: InputParamIndex) -> Self {
        let input_params_map = initalize_locals_map(input_params);

        Self {
            locals_map: FxHashMap::default(), // TODO (cesarzc): do the right thing.
            dynamic_scopes_stack: Vec::new(),
            compute_properties: ApplicationInstanceComputeProperties::default(),
        }
    }
}

#[derive(Debug)]
struct LocalComputeProperties {
    pub local: Local,
    pub compute_properties: ComputeProperties,
}

/// The compute properties of a callable application instance.
#[derive(Debug, Default)]
struct ApplicationInstanceComputeProperties {
    pub _blocks: FxHashMap<BlockId, ComputeProperties>,
    pub _stmts: FxHashMap<StmtId, ComputeProperties>,
    pub _exprs: FxHashMap<ExprId, ComputeProperties>,
}

/// Performs runtime capabilities analysis (RCA) on a package.
/// N.B. This function assumes specializations that are part of call cycles have already been analyzed. Otherwise, this
/// function will get stuck in an infinite analysis loop.
pub fn analyze_package(
    id: PackageId,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    let package = package_store.get(id).expect("package should exist");

    // Analyze all top-level items.
    for (item_id, _) in &package.items {
        analyze_item(
            (id, item_id).into(),
            package_store,
            package_store_scaffolding,
        );
    }

    // By this point, only top-level statements remain unanalyzed.
    for (stmt_id, _) in &package.stmts {
        analyze_statement(
            (id, stmt_id).into(),
            package_store,
            package_store_scaffolding,
        );
    }
}

/// Performs runtime capabilities analysis (RCA) on a specialization that is part of a callable cycle.
pub fn analyze_specialization_with_cyles(
    specialization_id: GlobalSpecializationId,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // This function is only called when a specialization has not already been analyzed.
    assert!(package_store_scaffolding
        .find_specialization(specialization_id)
        .is_none());
    let Some(Global::Callable(callable)) = package_store.get_global(specialization_id.callable)
    else {
        panic!("global item should exist and it should be a global");
    };

    let CallableImpl::Spec(spec_impl) = &callable.implementation else {
        panic!("callable implementation should not be intrinsic");
    };

    // Use the correct specialization declaration.
    let spec_decl = match specialization_id.specialization {
        SpecializationKind::Body => &spec_impl.body,
        SpecializationKind::Adj => spec_impl
            .adj
            .as_ref()
            .expect("adj specialization should exist"),
        SpecializationKind::Ctl => spec_impl
            .ctl
            .as_ref()
            .expect("ctl specialization should exist"),
        SpecializationKind::CtlAdj => spec_impl
            .ctl_adj
            .as_ref()
            .expect("ctl_adj specializatiob should exist"),
    };

    let input_params = derive_callable_input_params(
        callable,
        &package_store
            .get(specialization_id.callable.package)
            .expect("package should exist")
            .pats,
    );

    // Create compute properties differently depending on whether the callable is a function or an operation.
    let applications_table = match callable.kind {
        CallableKind::Function => create_cycled_function_specialization_applications_table(
            spec_decl,
            input_params.len(),
            &callable.output,
        ),
        CallableKind::Operation => create_cycled_operation_specialization_applications_table(
            spec_decl,
            input_params.len(),
            &callable.output,
        ),
    };

    // Finally, update the package store scaffolding.
    package_store_scaffolding.insert_spec(specialization_id, applications_table);
}

fn analyze_block(
    id: StoreBlockId,
    _input_params: &[InputParam],
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // This function is only called when a block has not already been analyzed.
    if package_store_scaffolding.find_block(id).is_some() {
        panic!("block is already analyzed");
    }

    let _block = package_store.get_block(id);
    // TODO (cesarzc): implement properly.
    package_store_scaffolding.insert_block(
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
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // Analyze the callable depending on its type.
    let input_params = derive_callable_input_params(
        callable,
        &package_store
            .get(id.package)
            .expect("package should exist")
            .pats,
    );
    match callable.implementation {
        CallableImpl::Intrinsic => {
            analyze_intrinsic_callable(id, callable, &input_params, package_store_scaffolding)
        }
        CallableImpl::Spec(_) => analyze_non_intrinsic_callable(
            id,
            callable,
            &input_params,
            package_store,
            package_store_scaffolding,
        ),
    }
}

fn analyze_intrinsic_callable(
    id: StoreItemId,
    callable: &CallableDecl,
    input_params: &Vec<InputParam>,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // If an entry for the specialization already exists, there is nothing left to do. Note that intrinsic callables
    // only have a body specialization.
    let body_specialization_id = GlobalSpecializationId::from((id, SpecializationKind::Body));
    if package_store_scaffolding
        .find_specialization(body_specialization_id)
        .is_some()
    {
        return;
    }

    // This function is meant for instrinsic callables only.
    assert!(matches!(callable.implementation, CallableImpl::Intrinsic));

    // Create an applications table depending on whether the callable is a function or an operation.
    let applications_table = match callable.kind {
        CallableKind::Function => {
            create_intrinsic_function_applications_table(callable, input_params)
        }
        CallableKind::Operation => {
            create_instrinsic_operation_applications_table(callable, input_params)
        }
    };
    package_store_scaffolding.insert_spec(body_specialization_id, applications_table);
}

fn analyze_item(
    id: StoreItemId,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    if let Some(Global::Callable(callable)) = package_store.get_global(id) {
        analyze_callable(id, callable, package_store, package_store_scaffolding);
    } else {
        package_store_scaffolding.insert_item(id, ItemScaffolding::NonCallable);
    }
}

fn analyze_non_intrinsic_callable(
    id: StoreItemId,
    callable: &CallableDecl,
    input_params: &Vec<InputParam>,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // This function is not meant for instrinsics.
    let CallableImpl::Spec(implementation) = &callable.implementation else {
        panic!("callable is assumed to have a specialized implementation");
    };

    // Analyze each one of the specializations.
    analyze_specialization(
        id,
        SpecializationKind::Body,
        &implementation.body,
        input_params,
        package_store,
        package_store_scaffolding,
    );

    if let Some(adj_spec) = &implementation.adj {
        analyze_specialization(
            id,
            SpecializationKind::Adj,
            adj_spec,
            input_params,
            package_store,
            package_store_scaffolding,
        );
    }

    if let Some(ctl_spec) = &implementation.ctl {
        analyze_specialization(
            id,
            SpecializationKind::Ctl,
            ctl_spec,
            input_params,
            package_store,
            package_store_scaffolding,
        );
    }

    if let Some(ctl_adj_spec) = &implementation.ctl_adj {
        analyze_specialization(
            id,
            SpecializationKind::CtlAdj,
            ctl_adj_spec,
            input_params,
            package_store,
            package_store_scaffolding,
        );
    }
}

fn analyze_specialization(
    callable_id: StoreItemId,
    spec_kind: SpecializationKind,
    spec_decl: &SpecDecl,
    input_params: &Vec<InputParam>,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // If an entry for the specialization already exists, there is nothing left to do.
    let specialization_id = GlobalSpecializationId::from((callable_id, spec_kind));
    if package_store_scaffolding
        .find_specialization(specialization_id)
        .is_some()
    {
        return;
    }

    let specialization_applications_table = create_specialization_applications_table(
        callable_id,
        spec_decl,
        input_params,
        package_store,
        package_store_scaffolding,
    );
    package_store_scaffolding.insert_spec(specialization_id, specialization_applications_table);
}

fn analyze_statement(
    id: StoreStmtId,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // If the item has already been analyzed, there's nothing left to do.
    if package_store_scaffolding.find_stmt(id).is_some() {
        return;
    }

    let _stmt = package_store.get_stmt(id);
    // TODO (cesarzc): Implement.
}

fn create_cycled_function_specialization_applications_table(
    spec_decl: &SpecDecl,
    callable_input_params_count: usize,
    output_type: &Ty,
) -> ApplicationsTable {
    // Functions can only have a body specialization, which does not have its input.
    assert!(spec_decl.input.is_none());

    // Since functions are classically pure, they inherently do not use any runtime feature nor represent a source of
    // dynamism.
    let inherent_properties = ComputeProperties::empty();

    // Create compute properties for each dynamic parameter.
    let mut dynamic_params_properties = Vec::new();
    for _ in 0..callable_input_params_count {
        // If any parameter is dynamic, we assume a function with cycles is a a source of dynamism if its output type
        // is non-unit.
        let dynamism_sources = if *output_type == Ty::UNIT {
            FxHashSet::default()
        } else {
            FxHashSet::from_iter(vec![DynamismSource::Assumed])
        };

        // Since convert functions can be called with dynamic parameters, we assume that all capabilities are required
        // for any dynamic parameter. The `CycledFunctionWithDynamicArg` feature conveys this assumption.
        let compute_properties = ComputeProperties {
            runtime_features: RuntimeFeatureFlags::CycledFunctionApplicationUsesDynamicArg,
            dynamism_sources,
        };
        dynamic_params_properties.push(compute_properties);
    }

    ApplicationsTable {
        inherent_properties,
        dynamic_params_properties,
    }
}

fn create_cycled_operation_specialization_applications_table(
    spec_decl: &SpecDecl,
    callable_input_params_count: usize,
    output_type: &Ty,
) -> ApplicationsTable {
    // Since operations can allocate and measure qubits freely, we assume it requires all capabilities (encompassed by
    // the `CycledOperationSpecialization` runtime feature) and that they are a source of dynamism if they have a
    // non-unit output.
    let dynamism_sources = if *output_type == Ty::UNIT {
        FxHashSet::default()
    } else {
        FxHashSet::from_iter(vec![DynamismSource::Assumed])
    };
    let compute_properties = ComputeProperties {
        runtime_features: RuntimeFeatureFlags::CycledOperationSpecializationApplication,
        dynamism_sources,
    };

    // If the specialization has its own input, then the number of input params needs to be increased by one.
    let specialization_input_params_count = if spec_decl.input.is_some() {
        callable_input_params_count + 1
    } else {
        callable_input_params_count
    };

    // Create compute properties for each dynamic parameter. These compute properties are the same than the inherent
    // properties.
    let mut dynamic_params_properties = Vec::new();
    for _ in 0..specialization_input_params_count {
        dynamic_params_properties.push(compute_properties.clone());
    }

    // Finally, create the applications table.
    ApplicationsTable {
        inherent_properties: compute_properties,
        dynamic_params_properties,
    }
}

fn create_intrinsic_function_applications_table(
    callable_decl: &CallableDecl,
    input_params: &Vec<InputParam>,
) -> ApplicationsTable {
    assert!(matches!(callable_decl.kind, CallableKind::Function));

    // Functions are purely classical, so no runtime features are needed and cannot be an inherent dynamism source.
    let inherent_properties = ComputeProperties::empty();

    // Calculate the properties for all parameters.
    let mut dynamic_params_properties = Vec::new();
    for param in input_params {
        // For intrinsic functions, we assume any parameter can contribute to the output, so if any parameter is dynamic
        // the output of the function is dynamic. Therefore, for all dynamic parameters, if the function's output is
        // non-unit:
        // - It becomes a source of dynamism.
        // - The output type contributes to the runtime features used by the function.
        let (dynamism_sources, mut runtime_features) = if callable_decl.output == Ty::UNIT {
            (FxHashSet::default(), RuntimeFeatureFlags::empty())
        } else {
            (
                FxHashSet::from_iter(vec![DynamismSource::Intrinsic]),
                derive_intrinsic_runtime_features_from_type(&callable_decl.output),
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

    ApplicationsTable {
        inherent_properties,
        dynamic_params_properties,
    }
}

fn create_instrinsic_operation_applications_table(
    callable_decl: &CallableDecl,
    input_params: &Vec<InputParam>,
) -> ApplicationsTable {
    assert!(matches!(callable_decl.kind, CallableKind::Operation));

    // Intrinsic operations inherently use runtime features if their output is not `Unit`, `Qubit` or `Result`, and
    // these runtime features are derived from the output type.
    let runtime_features = if callable_decl.output == Ty::UNIT
        || callable_decl.output == Ty::Prim(Prim::Qubit)
        || callable_decl.output == Ty::Prim(Prim::Result)
    {
        RuntimeFeatureFlags::empty()
    } else {
        derive_intrinsic_runtime_features_from_type(&callable_decl.output)
    };

    // Intrinsic are an inherent source of dynamism if their output is not `Unit` or `Qubit`.
    let dynamism_sources =
        if callable_decl.output == Ty::UNIT || callable_decl.output == Ty::Prim(Prim::Qubit) {
            FxHashSet::default()
        } else {
            FxHashSet::from_iter(vec![DynamismSource::Intrinsic])
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
        let dynamism_sources = if callable_decl.output == Ty::UNIT {
            FxHashSet::default()
        } else {
            FxHashSet::from_iter(vec![DynamismSource::Intrinsic])
        };

        // When a parameter is binded to a dynamic value, its runtime features depend on the parameter type.
        let param_compute_properties = ComputeProperties {
            runtime_features: derive_intrinsic_runtime_features_from_type(&param.ty),
            dynamism_sources,
        };
        dynamic_params_properties.push(param_compute_properties);
    }

    ApplicationsTable {
        inherent_properties,
        dynamic_params_properties,
    }
}

fn create_specialization_applications_table(
    callable_id: StoreItemId,
    spec_decl: &SpecDecl,
    callable_input_params: &Vec<InputParam>,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
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
        derive_specialization_input_params(spec_decl, callable_input_params, package_patterns);

    // Then we analyze the block which implements the specialization.
    let block_id = (callable_id.package, spec_decl.block).into();
    analyze_block(
        block_id,
        &specialization_input_params,
        package_store,
        package_store_scaffolding,
    );

    // Finally, we get the applications table of the analyzed block, which also represents the application table of the
    // specialization.
    let block_applications_table = package_store_scaffolding.get_block(block_id);
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
