// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    applications::{ApplicationInstance, LocalComputeProperties, SpecApplicationInstances},
    common::{
        derive_callable_input_params, derive_specialization_input_params, GlobalSpecializationId,
        InputParam, Local, LocalKind, SpecializationKind,
    },
    scaffolding::{ItemScaffolding, PackageStoreScaffolding},
    ApplicationsTable, ComputeKind, ComputeProperties, ComputePropertiesLookup, DynamismSource,
    RuntimeFeatureFlags,
};
use itertools::Itertools;
use qsc_fir::fir::{
    ExprId, ExprKind, Ident, Mutability, PackageLookup, Pat, PatId, PatKind, Res, StmtId, StmtKind,
    StoreBlockId, StoreExprId,
};
use qsc_fir::{
    fir::{
        CallableDecl, CallableImpl, CallableKind, Global, PackageId, PackageStore,
        PackageStoreLookup, SpecDecl, StoreItemId, StoreStmtId,
    },
    ty::{Prim, Ty},
};
use rustc_hash::FxHashSet;

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

    // By this point, only top-level statements remain unanalyzed, so analyze them now.
    let top_level_stmts: Vec<StmtId> = package
        .stmts
        .iter()
        .map(|(stmt_id, _)| stmt_id)
        .filter(|stmt_id| {
            package_store_scaffolding
                .find_stmt((id, *stmt_id).into())
                .is_none()
        })
        .sorted() // This is needed since the statements might depend on previous top-level statements.
        .collect();
    analyze_top_level_stmts(
        id,
        &top_level_stmts,
        package_store,
        package_store_scaffolding,
    );
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
    callable_input_params: &Vec<InputParam>,
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

    // Then we analyze the block which implements the specialization by simulating callable applications.
    let block_id = (callable_id.package, spec_decl.block).into();
    let mut spec_application_instances =
        SpecApplicationInstances::new(&specialization_input_params);

    // First, we simulate the inherent application, in which all arguments are static.
    simulate_block(
        block_id,
        &mut spec_application_instances.inherent,
        package_store,
        package_store_scaffolding,
    );
    spec_application_instances.inherent.settle();

    // Then, we simulate an application for each imput parameter, in which we consider it dynamic.
    for application_instance in spec_application_instances.dynamic_params.iter_mut() {
        simulate_block(
            block_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        );
        application_instance.settle();
    }

    // Now that we have all the application instances for the block that implements the specialization, we can close the
    // application instances for the specialization, which will save all the analysis to the package store scaffolding
    // and will return the applications table corresponding to the specialization.
    let package_scaffolding = package_store_scaffolding
        .get_mut(callable_id.package)
        .expect("package scaffolding should exist");
    let specialization_applications_table = spec_application_instances
        .close(package_scaffolding, Some(block_id.block))
        .expect("applications table should be some");

    // Finally, we insert the applications table to the scaffolding data structure.
    package_store_scaffolding.insert_spec(specialization_id, specialization_applications_table);
}

pub fn analyze_top_level_stmts(
    id: PackageId,
    top_level_stmts: &Vec<StmtId>,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // TODO (cesarzc): Temporary workaround while all statements are thoruoughly analyzed in std lib packages.
    if id == PackageId::from(0) || id == PackageId::from(1) {
        return;
    }

    // Analyze top-level statements as if they were all part of a parameterless operation.
    let input_params = Vec::<InputParam>::new();
    let mut application_instances = SpecApplicationInstances::new(&input_params);
    for stmt_id in top_level_stmts {
        simulate_stmt(
            (id, *stmt_id).into(),
            &mut application_instances.inherent,
            package_store,
            package_store_scaffolding,
        );
    }
    application_instances.inherent.settle();

    // Closing the application instances saves the analysis to the corresponding package scaffolding.
    let package_scaffolding = package_store_scaffolding
        .get_mut(id)
        .expect("package scaffolding should exist");
    _ = application_instances.close(package_scaffolding, None);
}

fn bind_expr_compute_properties(
    mutability: Mutability,
    pat_id: PatId,
    expr_id: ExprId,
    package: &impl PackageLookup,
    application_instance: &mut ApplicationInstance,
) {
    let expr = package.get_expr(expr_id);
    let pat = package.get_pat(pat_id);
    match &pat.kind {
        PatKind::Bind(ident) => {
            let compute_properties = application_instance
                .exprs
                .get(&expr_id)
                .expect("expression compute properties should exist")
                .clone();
            let kind = match mutability {
                Mutability::Immutable => LocalKind::Immutable(expr_id),
                Mutability::Mutable => LocalKind::Mutable,
            };
            bind_ident(pat, ident, kind, compute_properties, application_instance);
        }
        PatKind::Tuple(pats) => match &expr.kind {
            ExprKind::Tuple(exprs) => {
                for (pat_id, expr_id) in pats.iter().zip(exprs.iter()) {
                    bind_expr_compute_properties(
                        mutability,
                        *pat_id,
                        *expr_id,
                        package,
                        application_instance,
                    );
                }
            }
            _ => {
                bind_fixed_expr_compute_properties(
                    mutability,
                    pat_id,
                    expr_id,
                    package,
                    application_instance,
                );
            }
        },
        PatKind::Discard => {
            // Nothing to bind to.
        }
    }
}

fn bind_fixed_expr_compute_properties(
    mutability: Mutability,
    pat_id: PatId,
    expr_id: ExprId,
    package: &impl PackageLookup,
    application_instance: &mut ApplicationInstance,
) {
    let pat = package.get_pat(pat_id);
    match &pat.kind {
        PatKind::Bind(ident) => {
            let compute_properties = application_instance
                .exprs
                .get(&expr_id)
                .expect("expression compute properties should exist")
                .clone();
            let kind = match mutability {
                Mutability::Immutable => LocalKind::Immutable(expr_id),
                Mutability::Mutable => LocalKind::Mutable,
            };
            bind_ident(pat, ident, kind, compute_properties, application_instance);
        }
        PatKind::Tuple(pats) => {
            for pat_id in pats {
                bind_fixed_expr_compute_properties(
                    mutability,
                    *pat_id,
                    expr_id,
                    package,
                    application_instance,
                );
            }
        }
        PatKind::Discard => {
            // Nothing to bind to.
        }
    }
}

fn bind_ident(
    pat: &Pat,
    ident: &Ident,
    local_kind: LocalKind,
    compute_properties: ComputeProperties,
    application_instance: &mut ApplicationInstance,
) {
    let local = Local {
        node: ident.id,
        pat: pat.id,
        ty: pat.ty.clone(),
        kind: local_kind,
    };
    let local_compute_properties = LocalComputeProperties {
        local,
        compute_properties,
    };
    application_instance
        .locals_map
        .insert(ident.id, local_compute_properties);
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

fn simulate_block(
    id: StoreBlockId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // This function is only called when a block has not already been analyzed.
    if package_store_scaffolding.find_block(id).is_some() {
        panic!("block is already analyzed");
    }

    // Initialize the compute properties of the block.
    let block = package_store.get_block(id);
    let mut block_compute_properties = ComputeProperties::empty();

    // Iterate through the block statements and aggregate the runtime features of each into the block compute properties.
    for (stmt_index, stmt_id) in block.stmts.iter().enumerate() {
        let store_stmt_id = StoreStmtId::from((id.package, *stmt_id));
        simulate_stmt(
            store_stmt_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        );
        let stmt_compute_properties = application_instance
            .stmts
            .get(stmt_id)
            .expect("statement compute properties should exist");
        block_compute_properties.runtime_features |= stmt_compute_properties.runtime_features;

        // If this is the last statement and it is a non-unit expression without a trailing semicolon, aggregate it to
        // the block dynamism sources since the statement represents the block "return" value.
        if stmt_index == block.stmts.len() - 1 {
            let stmt = package_store.get_stmt((id.package, *stmt_id).into());
            if let StmtKind::Expr(expr_id) = stmt.kind {
                let expr = package_store.get_expr((id.package, expr_id).into());
                if expr.ty != Ty::UNIT {
                    block_compute_properties
                        .dynamism_sources
                        .insert(DynamismSource::Expr(expr_id));
                }
            }
        }
    }

    // Finally, we insert the compute properties of the block to the application instance.
    application_instance
        .blocks
        .insert(id.block, block_compute_properties);
}

fn simulate_expr(
    id: StoreExprId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    let expr = package_store.get_expr(id);
    match &expr.kind {
        ExprKind::Lit(_) => simulate_expr_lit(id, application_instance),
        ExprKind::Tuple(exprs) => simulate_expr_tuple(
            id,
            exprs,
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        ExprKind::Var(res, _) => simulate_expr_var(id, res, application_instance),
        // TODO (cesarzc): handle each case separately.
        _ => {
            application_instance
                .exprs
                .insert(id.expr, ComputeProperties::default());
        }
    }
}

fn simulate_expr_lit(id: StoreExprId, application_instance: &mut ApplicationInstance) {
    // Literal expressions have no runtime features nor are sources of dynamism so they are just empty compute
    // properties.
    application_instance
        .exprs
        .insert(id.expr, ComputeProperties::default());
}

fn simulate_expr_tuple(
    id: StoreExprId,
    exprs: &Vec<ExprId>,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    let mut tuple_compute_properties = ComputeProperties::default();

    // Go through each sub-expression in the tuple aggregating its runtime features and marking them as sources of
    // dynamism when applicable.
    for expr_id in exprs {
        let store_expr_id = StoreExprId::from((id.package, *expr_id));
        simulate_expr(
            store_expr_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        );
        let expr_compute_properties = application_instance
            .exprs
            .get(expr_id)
            .expect("expression compute properties should exist");

        // Aggregate the runtime features of the sub-expression.
        tuple_compute_properties.runtime_features |= expr_compute_properties.runtime_features;

        // Mark the expression as a source of dynamims if its compute kind is dynamic.
        if let ComputeKind::Dynamic = expr_compute_properties.compute_kind() {
            tuple_compute_properties
                .dynamism_sources
                .insert(DynamismSource::Expr(*expr_id));
        }
    }

    // Finally, insert the compute properties of the tuple expression.
    application_instance
        .exprs
        .insert(id.expr, tuple_compute_properties);
}

fn simulate_expr_var(id: StoreExprId, res: &Res, application_instance: &mut ApplicationInstance) {
    let compute_properties = match res {
        // Global items do not have compute properties by themselves.
        Res::Item(_) => ComputeProperties::default(),
        // Gather the current compute properties of the local.
        Res::Local(node_id) => application_instance
            .locals_map
            .get(node_id)
            .expect("compute properties for local should exist")
            .compute_properties
            .clone(),
        Res::Err => panic!("unexpected error resolution"),
    };
    application_instance
        .exprs
        .insert(id.expr, compute_properties);
}

fn simulate_stmt(
    id: StoreStmtId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    let stmt = package_store.get_stmt(id);
    match stmt.kind {
        StmtKind::Expr(expr_id) | StmtKind::Semi(expr_id) => simulate_stmt_expr(
            id,
            expr_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        StmtKind::Local(mutability, pat_id, expr_id) => simulate_stmt_local(
            id,
            mutability,
            pat_id,
            expr_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        StmtKind::Item(_) => {
            // An item statement does not have any inherent compute properties.
            application_instance
                .stmts
                .insert(id.stmt, ComputeProperties::empty());
        }
    }
}

fn simulate_stmt_expr(
    id: StoreStmtId,
    expr_id: ExprId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // Analyze the expression, whose compute properties will also be the compute properties of the statement.
    simulate_expr(
        (id.package, expr_id).into(),
        application_instance,
        package_store,
        package_store_scaffolding,
    );
    let stmt_compute_properties = application_instance
        .exprs
        .get(&expr_id)
        .expect("expression compute properties should exist")
        .clone();
    application_instance
        .stmts
        .insert(id.stmt, stmt_compute_properties);
}

fn simulate_stmt_local(
    id: StoreStmtId,
    mutability: Mutability,
    pat_id: PatId,
    expr_id: ExprId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // First, analyze the expression.
    simulate_expr(
        (id.package, expr_id).into(),
        application_instance,
        package_store,
        package_store_scaffolding,
    );

    // The compute properties of the binded expression are associated to the compute properties of both the local symbol
    // and the statement.
    let package = package_store.get(id.package).expect("package should exist");
    bind_expr_compute_properties(mutability, pat_id, expr_id, package, application_instance);
    let stmt_compute_properties = application_instance
        .exprs
        .get(&expr_id)
        .expect("expression compute properties should exist")
        .clone();
    application_instance
        .stmts
        .insert(id.stmt, stmt_compute_properties);
}
