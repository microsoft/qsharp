// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    applications::{ApplicationInstance, ApplicationInstancesTable, LocalComputeKind},
    common::{
        aggregate_compute_kind, derive_callable_input_params, try_resolve_callee, Callee,
        GlobalSpecId, InputParam, Local, LocalKind, SpecFunctor, SpecKind,
    },
    scaffolding::{ItemScaffolding, PackageScaffolding, PackageStoreScaffolding},
    ApplicationsTable, ComputeKind, ComputePropertiesLookup, QuantumProperties,
    RuntimeFeatureFlags, ValueKind,
};
use itertools::Itertools;
use qsc_fir::{
    fir::{
        BlockId, CallableDecl, CallableImpl, CallableKind, ExprId, ExprKind, Global, Ident,
        Mutability, PackageId, PackageLookup, PackageStore, PackageStoreLookup, Pat, PatId,
        PatKind, Res, SpecDecl, StmtId, StmtKind, StoreBlockId, StoreExprId, StoreItemId,
        StorePatId, StoreStmtId, StringComponent,
    },
    ty::{Prim, Ty},
};

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
    spec_id: GlobalSpecId,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // This function is only called when a specialization has not already been analyzed.
    assert!(package_store_scaffolding
        .find_specialization(spec_id)
        .is_none());
    let Some(Global::Callable(callable)) = package_store.get_global(spec_id.callable) else {
        panic!("global item should exist and it should be a global");
    };

    let CallableImpl::Spec(spec_impl) = &callable.implementation else {
        panic!("callable implementation should not be intrinsic");
    };

    let input_params = derive_callable_input_params(
        callable,
        &package_store
            .get(spec_id.callable.package)
            .expect("package should exist")
            .pats,
    );

    // Create compute properties differently depending on whether the callable is a function or an operation.
    let applications_table = match callable.kind {
        CallableKind::Function => create_cycled_function_specialization_applications_table(
            input_params.len(),
            &callable.output,
        ),
        CallableKind::Operation => create_cycled_operation_specialization_applications_table(
            input_params.len(),
            &callable.output,
        ),
    };

    // Now propagate the applications table through the implementation block.
    let package_id = spec_id.callable.package;
    let package = package_store.get_package(package_id);
    let package_scaffolding = package_store_scaffolding
        .get_mut(package_id)
        .expect("package scaffolding should exist");
    let spec_decl = match spec_id.spec_kind {
        SpecKind::Body => &spec_impl.body,
        SpecKind::Adj => spec_impl
            .adj
            .as_ref()
            .expect("adj specialization should exist"),
        SpecKind::Ctl => spec_impl
            .ctl
            .as_ref()
            .expect("ctl specialization should exist"),
        SpecKind::CtlAdj => spec_impl
            .ctl_adj
            .as_ref()
            .expect("ctl_adj specializatiob should exist"),
    };
    propagate_applications_table_through_block(
        spec_decl.block,
        &applications_table,
        package,
        package_scaffolding,
    );

    // Finally, update the package store scaffolding.
    package_store_scaffolding.insert_spec(spec_id, applications_table);
}

#[must_use]
fn aggregate_compute_kind_runtime_features(basis: ComputeKind, delta: &ComputeKind) -> ComputeKind {
    let ComputeKind::Quantum(delta_quantum_properties) = delta else {
        // A classical compute kind has nothing to aggregate so just return the base with no changes.
        return basis;
    };

    // Determine the aggregated runtime features.
    let runtime_features = match basis {
        ComputeKind::Classical => delta_quantum_properties.runtime_features,
        ComputeKind::Quantum(ref basis_quantum_properties) => {
            basis_quantum_properties.runtime_features | delta_quantum_properties.runtime_features
        }
    };

    // Use the value kind equivalent from the basis.
    let value_kind = match basis {
        ComputeKind::Classical => ValueKind::Static,
        ComputeKind::Quantum(basis_quantum_properties) => basis_quantum_properties.value_kind,
    };

    // Return the aggregated compute kind.
    ComputeKind::Quantum(QuantumProperties {
        runtime_features,
        value_kind,
    })
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
    let body_specialization_id = GlobalSpecId::from((id, SpecKind::Body));
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
    analyze_spec_decl(
        (id, SpecKind::Body).into(),
        &implementation.body,
        input_params,
        package_store,
        package_store_scaffolding,
    );

    if let Some(adj_spec) = &implementation.adj {
        analyze_spec_decl(
            (id, SpecKind::Adj).into(),
            adj_spec,
            input_params,
            package_store,
            package_store_scaffolding,
        );
    }

    if let Some(ctl_spec) = &implementation.ctl {
        analyze_spec_decl(
            (id, SpecKind::Ctl).into(),
            ctl_spec,
            input_params,
            package_store,
            package_store_scaffolding,
        );
    }

    if let Some(ctl_adj_spec) = &implementation.ctl_adj {
        analyze_spec_decl(
            (id, SpecKind::CtlAdj).into(),
            ctl_adj_spec,
            input_params,
            package_store,
            package_store_scaffolding,
        );
    }
}

fn analyze_spec(
    id: GlobalSpecId,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    let callable = package_store
        .get_global(id.callable)
        .expect("global should exist");
    let Global::Callable(callable_decl) = callable else {
        panic!("global should be a callable");
    };

    match &callable_decl.implementation {
        CallableImpl::Intrinsic => analyze_callable(
            id.callable,
            callable_decl,
            package_store,
            package_store_scaffolding,
        ),
        CallableImpl::Spec(spec_impl) => {
            let spec_decl = match id.spec_kind {
                SpecKind::Body => &spec_impl.body,
                SpecKind::Adj => spec_impl
                    .adj
                    .as_ref()
                    .expect("adj specialization should exist"),
                SpecKind::Ctl => spec_impl
                    .ctl
                    .as_ref()
                    .expect("ctl specialization should exist"),
                SpecKind::CtlAdj => spec_impl
                    .ctl_adj
                    .as_ref()
                    .expect("ctladj specialization should exist"),
            };
            let callable_input_params = derive_callable_input_params(
                callable_decl,
                &package_store
                    .get(id.callable.package)
                    .expect("package should exist")
                    .pats,
            );
            analyze_spec_decl(
                id,
                spec_decl,
                &callable_input_params,
                package_store,
                package_store_scaffolding,
            );
        }
    };
}

fn analyze_spec_decl(
    id: GlobalSpecId,
    spec_decl: &SpecDecl,
    callable_input_params: &Vec<InputParam>,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    // If an entry for the specialization already exists, there is nothing left to do.
    if package_store_scaffolding.find_specialization(id).is_some() {
        return;
    }

    // We expand the input map for controlled specializations, which have its own additional input (the control qubit
    // register).
    let package_patterns = &package_store
        .get(id.callable.package)
        .expect("package should exist")
        .pats;

    // Then we analyze the block which implements the specialization by simulating callable applications.
    let block_id = (id.callable.package, spec_decl.block).into();
    let mut application_instances_table =
        ApplicationInstancesTable::from_spec(spec_decl, callable_input_params, package_patterns);

    // First, we simulate the inherent application, in which all arguments are static.
    simulate_block(
        block_id,
        &mut application_instances_table.inherent,
        package_store,
        package_store_scaffolding,
    );
    application_instances_table.inherent.settle();

    // Then, we simulate an application for each imput parameter, in which we consider it dynamic.
    for application_instance in application_instances_table.dynamic_params.iter_mut() {
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
        .get_mut(id.callable.package)
        .expect("package scaffolding should exist");
    let specialization_applications_table = application_instances_table
        .close(package_scaffolding, Some(block_id.block))
        .expect("applications table should be some");

    // Finally, we insert the applications table to the scaffolding data structure.
    package_store_scaffolding.insert_spec(id, specialization_applications_table);
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
    let mut application_instances_table = ApplicationInstancesTable::parameterless();
    for stmt_id in top_level_stmts {
        simulate_stmt(
            (id, *stmt_id).into(),
            &mut application_instances_table.inherent,
            package_store,
            package_store_scaffolding,
        );
    }
    application_instances_table.inherent.settle();

    // Closing the application instances saves the analysis to the corresponding package scaffolding.
    let package_scaffolding = package_store_scaffolding
        .get_mut(id)
        .expect("package scaffolding should exist");
    _ = application_instances_table.close(package_scaffolding, None);
}

fn bind_expr_compute_kind_to_pattern(
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
            let compute_kind = application_instance
                .exprs
                .get(&expr_id)
                .expect("expression's compute kind should exist")
                .clone();
            let local_kind = match mutability {
                Mutability::Immutable => LocalKind::Immutable(expr_id),
                Mutability::Mutable => LocalKind::Mutable,
            };
            bind_compute_kind_to_ident(pat, ident, local_kind, compute_kind, application_instance);
        }
        PatKind::Tuple(pats) => match &expr.kind {
            ExprKind::Tuple(exprs) => {
                for (pat_id, expr_id) in pats.iter().zip(exprs.iter()) {
                    bind_expr_compute_kind_to_pattern(
                        mutability,
                        *pat_id,
                        *expr_id,
                        package,
                        application_instance,
                    );
                }
            }
            _ => {
                bind_fixed_expr_compute_kind_to_pattern(
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

fn bind_fixed_expr_compute_kind_to_pattern(
    mutability: Mutability,
    pat_id: PatId,
    expr_id: ExprId,
    package: &impl PackageLookup,
    application_instance: &mut ApplicationInstance,
) {
    let pat = package.get_pat(pat_id);
    match &pat.kind {
        PatKind::Bind(ident) => {
            let compute_kind = application_instance
                .exprs
                .get(&expr_id)
                .expect("expression's compute kind should exist")
                .clone();
            let local_kind = match mutability {
                Mutability::Immutable => LocalKind::Immutable(expr_id),
                Mutability::Mutable => LocalKind::Mutable,
            };
            bind_compute_kind_to_ident(pat, ident, local_kind, compute_kind, application_instance);
        }
        PatKind::Tuple(pats) => {
            for pat_id in pats {
                bind_fixed_expr_compute_kind_to_pattern(
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

fn bind_compute_kind_to_ident(
    pat: &Pat,
    ident: &Ident,
    local_kind: LocalKind,
    compute_kind: ComputeKind,
    application_instance: &mut ApplicationInstance,
) {
    let local = Local {
        node: ident.id,
        pat: pat.id,
        ty: pat.ty.clone(),
        kind: local_kind,
    };
    let local_compute_kind = LocalComputeKind {
        local,
        compute_kind,
    };
    application_instance
        .locals_map
        .insert(ident.id, local_compute_kind);
}

fn create_compute_kind_for_call_with_dynamic_callee(expr_type: &Ty) -> ComputeKind {
    let value_kind = if *expr_type == Ty::UNIT {
        ValueKind::Static
    } else {
        ValueKind::Dynamic
    };
    ComputeKind::Quantum(QuantumProperties {
        runtime_features: RuntimeFeatureFlags::DynamicCallee,
        value_kind,
    })
}

fn create_compute_kind_for_call_with_spec_callee(
    callee: &Callee,
    callable_decl: &CallableDecl,
    args_expr_id: StoreExprId,
    application_instance: &ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // Now that we know the callee, we can use its applications table to determine the compute kind of the call
    // expression.
    let callee_id = GlobalSpecId::from((callee.item, SpecKind::from(&callee.spec_functor)));
    analyze_spec(callee_id, package_store, package_store_scaffolding);
    let callee_applications_table = package_store_scaffolding.get_spec(callee_id);

    // We need to split controls and specialization input arguments so we can use the right entry in the applications
    // table.
    let args_package = package_store.get_package(args_expr_id.package);
    let (args_controls, args_input_id) =
        split_controls_and_input(args_expr_id.expr, &callee.spec_functor, args_package);
    let callee_input_pattern_id =
        StorePatId::from((callee_id.callable.package, callable_decl.input));
    let args_input_id = StoreExprId::from((args_expr_id.package, args_input_id));
    let input_param_exprs =
        map_input_param_exprs(callee_input_pattern_id, args_input_id, package_store);
    let input_params_dynamism: Vec<bool> = input_param_exprs
        .iter()
        .map(|expr_id| {
            let input_param_compute_kind = application_instance
                .exprs
                .get(expr_id)
                .expect("expression compute kind should exist");
            input_param_compute_kind.is_value_dynamic()
        })
        .collect();

    // Derive the compute kind based on the dynamism of input parameters.
    let mut compute_kind =
        callee_applications_table.derive_application_compute_kind(&input_params_dynamism);

    // Aggreagate runtime features from the qubit controls expressions.
    for control_expr in args_controls {
        let control_expr_compute_kind = application_instance
            .exprs
            .get(&control_expr)
            .expect("control expression compute kind should exist");
        compute_kind =
            aggregate_compute_kind_runtime_features(compute_kind, control_expr_compute_kind);
    }

    compute_kind
}

fn create_compute_kind_for_call_with_static_callee(
    callee_expr_id: StoreExprId,
    args_expr_id: StoreExprId,
    expr_type: &Ty,
    application_instance: &ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // Try to resolve the callee.
    let callee_expr_package = package_store
        .get(callee_expr_id.package)
        .expect("package should exist");
    let maybe_callee = try_resolve_callee(
        callee_expr_id.expr,
        callee_expr_id.package,
        callee_expr_package,
        &application_instance.locals_map,
    );
    let Some(callee) = maybe_callee else {
        return create_compute_kind_for_call_with_unresolved_callee(expr_type);
    };

    // We could resolve the callee. Determine the compute kind of the call depending on the callee kind.
    let global_callee = package_store
        .get_global(callee.item)
        .expect("global should exist");
    match global_callee {
        Global::Callable(callable_decl) => create_compute_kind_for_call_with_spec_callee(
            &callee,
            callable_decl,
            args_expr_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        Global::Udt => {
            create_compute_kind_for_call_with_udt_callee(args_expr_id.expr, application_instance)
        }
    }
}

fn create_compute_kind_for_call_with_udt_callee(
    args_expr_id: ExprId,
    application_instance: &ApplicationInstance,
) -> ComputeKind {
    let args_expr_compute_kind = application_instance
        .exprs
        .get(&args_expr_id)
        .expect("expression compute kind should exist");
    match args_expr_compute_kind {
        ComputeKind::Classical => ComputeKind::Classical,
        ComputeKind::Quantum(quantum_properties) => match quantum_properties.value_kind {
            ValueKind::Static => ComputeKind::Quantum(QuantumProperties::default()),
            // If any argument to the UDT constructor is dynamic, then mark this runtime feature as being used.
            ValueKind::Dynamic => ComputeKind::Quantum(QuantumProperties {
                runtime_features: RuntimeFeatureFlags::UdtConstructorUsesDynamicArg,
                value_kind: ValueKind::Dynamic,
            }),
        },
    }
}

fn create_compute_kind_for_call_with_unresolved_callee(expr_type: &Ty) -> ComputeKind {
    let value_kind = if *expr_type == Ty::UNIT {
        ValueKind::Static
    } else {
        // Assume a dynamic value kind if the output of the call expression is non-unit.
        ValueKind::Dynamic
    };
    ComputeKind::Quantum(QuantumProperties {
        // Mark the runtime feature being used.
        runtime_features: RuntimeFeatureFlags::UnresolvedCallee,
        value_kind,
    })
}

fn create_cycled_function_specialization_applications_table(
    callable_input_params_count: usize,
    output_type: &Ty,
) -> ApplicationsTable {
    // Set the compute kind of the function for each parameter when it is binded to a dynamic value.
    let mut using_dynamic_param = Vec::new();
    for _ in 0..callable_input_params_count {
        // If any parameter is dynamic, we assume the value of a function with cycles is a a source of dynamism if its
        // output type is non-unit.
        let value_kind = if *output_type == Ty::UNIT {
            ValueKind::Static
        } else {
            ValueKind::Dynamic
        };

        // Since cycled functions can be called with dynamic parameters, we assume that all capabilities are required
        // if the function is used with any dynamic parameter. The `CycledFunctionWithDynamicArg` feature conveys this
        // assumption.
        let quantum_properties = QuantumProperties {
            runtime_features: RuntimeFeatureFlags::CycledFunctionUsesDynamicArg,
            value_kind,
        };
        using_dynamic_param.push(ComputeKind::Quantum(quantum_properties));
    }

    ApplicationsTable {
        // Functions are inherently classically pure.
        inherent: ComputeKind::Classical,
        dynamic_param_applications: using_dynamic_param,
    }
}

fn create_cycled_operation_specialization_applications_table(
    callable_input_params_count: usize,
    output_type: &Ty,
) -> ApplicationsTable {
    // Since operations can allocate and measure qubits freely, we assume its compute kind is quantum, requires all
    // capabilities (encompassed by the `CycledOperation` runtime feature) and that their value kind is dynamic.
    let value_kind = if *output_type == Ty::UNIT {
        ValueKind::Static
    } else {
        ValueKind::Dynamic
    };
    let inherent_compute_kind = ComputeKind::Quantum(QuantumProperties {
        runtime_features: RuntimeFeatureFlags::CycledOperation,
        value_kind,
    });

    // The compute kind of a cycled function when any of its parameters is binded to a dynamic value is the same as its
    // inherent compute kind.
    let mut using_dynamic_param = Vec::new();
    for _ in 0..callable_input_params_count {
        using_dynamic_param.push(inherent_compute_kind.clone());
    }

    // Finally, create the applications table.
    ApplicationsTable {
        inherent: inherent_compute_kind,
        dynamic_param_applications: using_dynamic_param,
    }
}

fn create_intrinsic_function_applications_table(
    callable_decl: &CallableDecl,
    input_params: &Vec<InputParam>,
) -> ApplicationsTable {
    assert!(matches!(callable_decl.kind, CallableKind::Function));

    // Determine the compute kind for all dynamic parameter applications.
    let mut dynamic_param_applications = Vec::new();
    for param in input_params {
        // For intrinsic functions, we assume any parameter can contribute to the output, so if any parameter is dynamic
        // the output of the function is dynamic. Therefore, for all dynamic parameters, if the function's output is
        // non-unit their value kind is dynamic.
        let value_kind = if callable_decl.output == Ty::UNIT {
            ValueKind::Static
        } else {
            ValueKind::Dynamic
        };

        let param_compute_kind = ComputeKind::Quantum(QuantumProperties {
            // When a parameter is binded to a dynamic value, its type contributes to the runtime features used by the
            // function application.
            runtime_features: derive_runtime_features_for_dynamic_type(&param.ty),
            value_kind,
        });
        dynamic_param_applications.push(param_compute_kind);
    }

    ApplicationsTable {
        // Functions are inherently classical.
        inherent: ComputeKind::Classical,
        dynamic_param_applications,
    }
}

fn create_instrinsic_operation_applications_table(
    callable_decl: &CallableDecl,
    input_params: &Vec<InputParam>,
) -> ApplicationsTable {
    assert!(matches!(callable_decl.kind, CallableKind::Operation));

    // The value kind of intrinsic operations is inherently dynamic if their output is not `Unit` or `Qubit`.
    let value_kind =
        if callable_decl.output == Ty::UNIT || callable_decl.output == Ty::Prim(Prim::Qubit) {
            ValueKind::Static
        } else {
            ValueKind::Dynamic
        };

    // The compute kind of intrinsic operations is always quantum.
    let inherent_compute_kind = ComputeKind::Quantum(QuantumProperties {
        runtime_features: RuntimeFeatureFlags::empty(),
        value_kind,
    });

    // Determine the compute kind of all dynamic parameter applications.
    let mut dynamic_param_applications = Vec::new();
    for param in input_params {
        // For intrinsic operations, we assume any parameter can contribute to the output, so if any parameter is
        // dynamic the output of the operation is dynamic. Therefore, for all dynamic parameters, if the operation's
        // output is non-unit it becomes a source of dynamism.
        let value_kind = if callable_decl.output == Ty::UNIT {
            ValueKind::Static
        } else {
            ValueKind::Dynamic
        };

        // The compute kind of intrinsic operations is always quantum.
        let param_compute_kind = ComputeKind::Quantum(QuantumProperties {
            // When a parameter is binded to a dynamic value, its type contributes to the runtime features used by the
            // operation application.
            runtime_features: derive_runtime_features_for_dynamic_type(&param.ty),
            value_kind,
        });
        dynamic_param_applications.push(param_compute_kind);
    }

    ApplicationsTable {
        inherent: inherent_compute_kind,
        dynamic_param_applications,
    }
}

fn derive_runtime_features_for_dynamic_type(ty: &Ty) -> RuntimeFeatureFlags {
    fn intrinsic_runtime_features_from_primitive_type(prim: &Prim) -> RuntimeFeatureFlags {
        match prim {
            Prim::BigInt => RuntimeFeatureFlags::UseOfDynamicBigInt,
            Prim::Bool => RuntimeFeatureFlags::UseOfDynamicBool,
            Prim::Double => RuntimeFeatureFlags::UseOfDynamicDouble,
            Prim::Int => RuntimeFeatureFlags::UseOfDynamicInt,
            Prim::Pauli => RuntimeFeatureFlags::UseOfDynamicPauli,
            Prim::Qubit => RuntimeFeatureFlags::UseOfDynamicQubit,
            Prim::Range | Prim::RangeFrom | Prim::RangeTo | Prim::RangeFull => {
                RuntimeFeatureFlags::UseOfDynamicRange
            }
            // Results are inherently dynamic but they do not need special runtime features just to exist.
            Prim::Result => RuntimeFeatureFlags::empty(),
            Prim::String => RuntimeFeatureFlags::UseOfDynamicString,
        }
    }

    fn intrinsic_runtime_features_from_tuple(tuple: &Vec<Ty>) -> RuntimeFeatureFlags {
        let mut runtime_features = if tuple.is_empty() {
            RuntimeFeatureFlags::empty()
        } else {
            RuntimeFeatureFlags::UseOfDynamicTuple
        };
        for item_type in tuple {
            runtime_features |= derive_runtime_features_for_dynamic_type(item_type);
        }
        runtime_features
    }

    match ty {
        Ty::Array(ty) => {
            RuntimeFeatureFlags::UseOfDynamicArray | derive_runtime_features_for_dynamic_type(ty)
        }
        Ty::Arrow(arrow) => {
            let mut runtime_features = match arrow.kind {
                CallableKind::Function => RuntimeFeatureFlags::UseOfDynamicArrowFunction,
                CallableKind::Operation => RuntimeFeatureFlags::UseOfDynamicArrowOperation,
            };
            runtime_features |= derive_runtime_features_for_dynamic_type(&arrow.input);
            runtime_features |= derive_runtime_features_for_dynamic_type(&arrow.output);
            runtime_features
        }
        Ty::Infer(_) => panic!("cannot derive runtime features for `Infer` type"),
        Ty::Param(_) => RuntimeFeatureFlags::UseOfDynamicGeneric,
        Ty::Prim(prim) => intrinsic_runtime_features_from_primitive_type(prim),
        Ty::Tuple(tuple) => intrinsic_runtime_features_from_tuple(tuple),
        // N.B. Runtime features can be more nuanced by taking into account the contained types.
        Ty::Udt(_) => RuntimeFeatureFlags::UseOfDynamicUdt,
        Ty::Err => panic!("cannot derive runtime features for `Err` type"),
    }
}

fn determine_expr_array_compute_kind(
    exprs: impl Iterator<Item = StoreExprId>,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // Initialize the array's compute kind.
    let mut array_compute_kind = ComputeKind::Classical;

    // Go through each sub-expression in the tuple aggregating ONLY the runtime features.
    for expr_id in exprs {
        simulate_expr(
            expr_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        );
        let expr_compute_kind = application_instance
            .exprs
            .get(&expr_id.expr)
            .expect("expression's compute kind should exist");
        array_compute_kind =
            aggregate_compute_kind_runtime_features(array_compute_kind, expr_compute_kind);
    }

    // The value kind of an array expression itself cannot be dynamic since its size is always known.
    assert!(!array_compute_kind.is_value_dynamic());
    array_compute_kind
}

fn determine_expr_array_repeat_compute_kind(
    value_expr_id: StoreExprId,
    size_expr_id: StoreExprId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // First, simulate the value and size expressions.
    simulate_expr(
        value_expr_id,
        application_instance,
        package_store,
        package_store_scaffolding,
    );
    simulate_expr(
        size_expr_id,
        application_instance,
        package_store,
        package_store_scaffolding,
    );

    // Initialize the compute kind from the size expression (since this one determines whether the whole array is
    // dynamic), and then aggregate the runtime features from the value.
    let mut compute_kind = application_instance
        .exprs
        .get(&size_expr_id.expr)
        .expect("compute kind for the size expression should exist")
        .clone();
    let value_expr_compute_kind = application_instance
        .exprs
        .get(&value_expr_id.expr)
        .expect("compute kind for value expression should exist");
    compute_kind = aggregate_compute_kind_runtime_features(compute_kind, value_expr_compute_kind);
    compute_kind
}

fn determine_expr_assign_compute_kind(
    lhs_expr_id: StoreExprId,
    rhs_expr_id: StoreExprId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // First, simulate the LHS and RHS expressions.
    simulate_expr(
        lhs_expr_id,
        application_instance,
        package_store,
        package_store_scaffolding,
    );
    simulate_expr(
        rhs_expr_id,
        application_instance,
        package_store,
        package_store_scaffolding,
    );

    // Since this is an assignment, well update the local variables on the LHS expression with the compute kind of the
    // RHS expression.
    let package = package_store.get_package(lhs_expr_id.package);
    update_locals_compute_kind(
        lhs_expr_id.expr,
        rhs_expr_id.expr,
        package,
        application_instance,
    );

    // Finally, the compute kind of this expression is constructed from the RHS expression runtime features.
    let rhs_compute_kind = application_instance
        .exprs
        .get(&rhs_expr_id.expr)
        .expect("RHS expression compute kind should exist");
    aggregate_compute_kind_runtime_features(ComputeKind::Classical, rhs_compute_kind)
}

fn determine_expr_bin_op_compute_kind(
    lhs_expr_id: StoreExprId,
    rhs_expr_id: StoreExprId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // First, simulate the LHS and RHS expressions.
    simulate_expr(
        lhs_expr_id,
        application_instance,
        package_store,
        package_store_scaffolding,
    );
    simulate_expr(
        rhs_expr_id,
        application_instance,
        package_store,
        package_store_scaffolding,
    );

    // The compute kind of a binary operator expression is the aggregation of its LHS and RHS expressions.
    let mut compute_kind = application_instance
        .exprs
        .get(&lhs_expr_id.expr)
        .expect("LHS expression compute kind should exist")
        .clone();
    let rhs_compute_kind = application_instance
        .exprs
        .get(&rhs_expr_id.expr)
        .expect("RHS expression compute kind should exist");
    compute_kind = aggregate_compute_kind(compute_kind, rhs_compute_kind);
    compute_kind
}

fn determine_expr_block_compute_kind(
    block_id: StoreBlockId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // The compute kind of the block is the same as the block expression.
    simulate_block(
        block_id,
        application_instance,
        package_store,
        package_store_scaffolding,
    );
    application_instance
        .blocks
        .get(&block_id.block)
        .expect("block compute kind should exist")
        .clone()
}

fn determine_expr_call_compute_kind(
    callee_expr_id: StoreExprId,
    args_expr_id: StoreExprId,
    expr_type: &Ty,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // First, simulate the callee and arguments expressions.
    simulate_expr(
        callee_expr_id,
        application_instance,
        package_store,
        package_store_scaffolding,
    );
    simulate_expr(
        args_expr_id,
        application_instance,
        package_store,
        package_store_scaffolding,
    );

    // Get the compute kind of the callee expression and based on it create the compute kind for the call expression.
    let callee_expr_compute_kind = application_instance
        .exprs
        .get(&callee_expr_id.expr)
        .expect("compute kind should exist");
    let mut compute_kind = match callee_expr_compute_kind {
        ComputeKind::Classical => create_compute_kind_for_call_with_static_callee(
            callee_expr_id,
            args_expr_id,
            expr_type,
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        ComputeKind::Quantum(calle_expr_quantum_properties) => {
            match calle_expr_quantum_properties.value_kind {
                ValueKind::Static => create_compute_kind_for_call_with_static_callee(
                    callee_expr_id,
                    args_expr_id,
                    expr_type,
                    application_instance,
                    package_store,
                    package_store_scaffolding,
                ),
                ValueKind::Dynamic => create_compute_kind_for_call_with_dynamic_callee(expr_type),
            }
        }
    };

    // If this call happens within a dynamic scopre, there might be additional runtime features being used.
    if !application_instance.active_dynamic_scopes.is_empty() {
        // Any call that happens within a dynamic scope uses the forward branching runtime feature.
        compute_kind = aggregate_compute_kind_runtime_features(
            compute_kind,
            &ComputeKind::with_runtime_features(
                RuntimeFeatureFlags::ForwardBranchingOnDynamicValue,
            ),
        );

        // If the call expression type is either a result or a qubit, it uses the dynamic allocation runtime features.
        if let Ty::Prim(Prim::Qubit) = expr_type {
            compute_kind = aggregate_compute_kind_runtime_features(
                compute_kind,
                &ComputeKind::with_runtime_features(RuntimeFeatureFlags::DynamicQubitAllocation),
            );
        }
        if let Ty::Prim(Prim::Result) = expr_type {
            compute_kind = aggregate_compute_kind_runtime_features(
                compute_kind,
                &ComputeKind::with_runtime_features(RuntimeFeatureFlags::DynamicResultAllocation),
            );
        }
    }

    // Aggregate runtime features from the callee and args expressions.
    compute_kind = aggregate_compute_kind_runtime_features(compute_kind, callee_expr_compute_kind);
    compute_kind = aggregate_compute_kind_runtime_features(compute_kind, callee_expr_compute_kind);
    compute_kind
}

fn determine_expr_if_compute_kind(
    condition_expr_id: StoreExprId,
    if_true_expr_id: StoreExprId,
    if_false_expr_id: Option<StoreExprId>,
    expr_type: &Ty,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // First, simulate sub-expressions.
    simulate_expr(
        condition_expr_id,
        application_instance,
        package_store,
        package_store_scaffolding,
    );

    // If the condition sub-expression is dynamic, we push a new dynamic scope.
    let condition_expr_compute_kind = application_instance
        .exprs
        .get(&condition_expr_id.expr)
        .expect("condition expression compute kind should exist");
    let within_dynamic_scope = condition_expr_compute_kind.is_value_dynamic();
    if within_dynamic_scope {
        application_instance
            .active_dynamic_scopes
            .push(condition_expr_id.expr);
    }

    // Simulate the branch expressions.
    simulate_expr(
        if_true_expr_id,
        application_instance,
        package_store,
        package_store_scaffolding,
    );
    if let Some(if_false_expr_id) = if_false_expr_id {
        simulate_expr(
            if_false_expr_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        );
    }

    // Pop the dynamic scope.
    if within_dynamic_scope {
        let dynamic_scope_expr_id = application_instance
            .active_dynamic_scopes
            .pop()
            .expect("at least one dynamic scope should exist");
        assert!(dynamic_scope_expr_id == condition_expr_id.expr);
    }

    // Aggregate the runtime features of the sub-expressions, as well as whether any of the sub-expressions's value is
    // dynamic.
    let mut compute_kind = ComputeKind::Classical;
    let mut is_any_sub_expr_value_dynamic = false;
    let condition_expr_compute_kind = application_instance
        .exprs
        .get(&condition_expr_id.expr)
        .expect("condition expression compute kind should exist");
    compute_kind =
        aggregate_compute_kind_runtime_features(compute_kind, condition_expr_compute_kind);
    is_any_sub_expr_value_dynamic |= condition_expr_compute_kind.is_value_dynamic();
    let if_true_expr_compute_kind = application_instance
        .exprs
        .get(&if_true_expr_id.expr)
        .expect("condition expression compute kind should exist");
    compute_kind = aggregate_compute_kind_runtime_features(compute_kind, if_true_expr_compute_kind);
    is_any_sub_expr_value_dynamic |= if_true_expr_compute_kind.is_value_dynamic();
    if let Some(if_false_expr_id) = if_false_expr_id {
        let if_false_expr_compute_kind = application_instance
            .exprs
            .get(&if_false_expr_id.expr)
            .expect("condition expression compute kind should exist");
        compute_kind =
            aggregate_compute_kind_runtime_features(compute_kind, if_false_expr_compute_kind);
        is_any_sub_expr_value_dynamic |= if_false_expr_compute_kind.is_value_dynamic();
    }

    // If the expression's type is non-unit and any of the sub-expressions' value is dynamic, then the compute kind of
    // this expression is dynamic.
    if *expr_type != Ty::UNIT && is_any_sub_expr_value_dynamic {
        compute_kind.aggregate_value_kind(ValueKind::Dynamic);
    }
    compute_kind
}

fn determine_expr_lit_compute_kind() -> ComputeKind {
    // Literal expressions are purely classical.
    ComputeKind::Classical
}

fn determine_expr_tuple_compute_kind(
    exprs: impl Iterator<Item = StoreExprId>,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // Initialize the tuple's compute kind.
    let mut tuple_compute_kind = ComputeKind::Classical;

    // Go through each sub-expression in the tuple aggregating them.
    for expr_id in exprs {
        simulate_expr(
            expr_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        );
        let expr_compute_kind = application_instance
            .exprs
            .get(&expr_id.expr)
            .expect("expression's compute kind should exist");
        tuple_compute_kind = aggregate_compute_kind(tuple_compute_kind, expr_compute_kind);
    }

    tuple_compute_kind
}

fn determine_expr_var_compute_kind(
    res: &Res,
    application_instance: &mut ApplicationInstance,
) -> ComputeKind {
    match res {
        // Global items do not have quantum properties by themselves so we can consider them classical.
        Res::Item(_) => ComputeKind::Classical,
        // Gather the current compute kind of the local.
        Res::Local(node_id) => application_instance
            .locals_map
            .get_compute_kind(*node_id)
            .clone(),
        Res::Err => panic!("unexpected error resolution"),
    }
}

fn determine_stmt_expr_compute_kind(
    package_id: PackageId,
    expr_id: ExprId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // First, analyze the expression.
    simulate_expr(
        (package_id, expr_id).into(),
        application_instance,
        package_store,
        package_store_scaffolding,
    );

    // The statement's compute kind is the same as the expression's compute kind.
    application_instance
        .exprs
        .get(&expr_id)
        .expect("expression's compute kind should exist")
        .clone()
}

fn determine_stmt_local_compute_kind(
    package_id: PackageId,
    mutability: Mutability,
    pat_id: PatId,
    expr_id: ExprId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // First, analyze the expression.
    simulate_expr(
        (package_id, expr_id).into(),
        application_instance,
        package_store,
        package_store_scaffolding,
    );

    // Bind the expression's compute kind to the pattern.
    let package = package_store.get(package_id).expect("package should exist");
    bind_expr_compute_kind_to_pattern(mutability, pat_id, expr_id, package, application_instance);

    // Use the expression's compute kind to construct the statement's compute kind, only using using the expression's
    // runtime features since the value is meaningless for local (binding) statements.
    let expr_compute_kind = application_instance
        .exprs
        .get(&expr_id)
        .expect("expression's compute kind should exist");
    aggregate_compute_kind_runtime_features(ComputeKind::Classical, expr_compute_kind)
}

fn determine_stmt_semi_compute_kind(
    package_id: PackageId,
    expr_id: ExprId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) -> ComputeKind {
    // First, analyze the expression.
    simulate_expr(
        (package_id, expr_id).into(),
        application_instance,
        package_store,
        package_store_scaffolding,
    );

    // Use the expression's compute kind to construct the statement's compute kind, only using using the expression's
    // runtime features since the value is meaningless for semicolon statements.
    let expr_compute_kind = application_instance
        .exprs
        .get(&expr_id)
        .expect("expression's compute kind should exist");
    aggregate_compute_kind_runtime_features(ComputeKind::Classical, expr_compute_kind)
}

fn propagate_applications_table_through_block(
    block_id: BlockId,
    applications_table: &ApplicationsTable,
    package: &impl PackageLookup,
    package_scaffolding: &mut PackageScaffolding,
) {
    let block = package.get_block(block_id);
    for stmt_id in &block.stmts {
        propagate_applications_table_through_stmt(
            *stmt_id,
            applications_table,
            package,
            package_scaffolding,
        );
    }
    package_scaffolding
        .blocks
        .insert(block_id, applications_table.clone());
}

fn map_input_param_exprs(
    pat_id: StorePatId,
    expr_id: StoreExprId,
    package_store: &impl PackageStoreLookup,
) -> Vec<ExprId> {
    let pat = package_store.get_pat(pat_id);
    match &pat.kind {
        PatKind::Bind(_) | PatKind::Discard => vec![expr_id.expr],
        PatKind::Tuple(pats) => {
            let expr = package_store.get_expr(expr_id);
            match &expr.kind {
                ExprKind::Tuple(exprs) => {
                    assert!(pats.len() == exprs.len());
                    let mut input_param_exprs = Vec::<ExprId>::with_capacity(pats.len());
                    for (local_pat_id, local_expr_id) in pats.iter().zip(exprs.iter()) {
                        let global_pat_id = StorePatId::from((pat_id.package, *local_pat_id));
                        let global_expr_id = StoreExprId::from((expr_id.package, *local_expr_id));
                        let mut sub_input_param_exprs =
                            map_input_param_exprs(global_pat_id, global_expr_id, package_store);
                        input_param_exprs.append(&mut sub_input_param_exprs);
                    }
                    input_param_exprs
                }
                _ => panic!("expected tuple expression"),
            }
        }
    }
}

fn propagate_applications_table_through_expr(
    expr_id: ExprId,
    applications_table: &ApplicationsTable,
    package: &impl PackageLookup,
    package_scaffolding: &mut PackageScaffolding,
) {
    // Convenience closures to make this function more succint.
    let mut propagate_expr = |id: ExprId| {
        propagate_applications_table_through_expr(
            id,
            applications_table,
            package,
            package_scaffolding,
        );
    };

    // Propagate the application table through all the sub-expressions.
    let expr = package.get_expr(expr_id);
    match &expr.kind {
        ExprKind::Array(exprs) => exprs.iter().for_each(|e| propagate_expr(*e)),
        ExprKind::ArrayRepeat(item, size) => {
            propagate_expr(*item);
            propagate_expr(*size);
        }
        ExprKind::Assign(lhs, rhs)
        | ExprKind::AssignOp(_, lhs, rhs)
        | ExprKind::BinOp(_, lhs, rhs) => {
            propagate_expr(*lhs);
            propagate_expr(*rhs);
        }
        ExprKind::AssignField(record, _, replace) | ExprKind::UpdateField(record, _, replace) => {
            propagate_expr(*record);
            propagate_expr(*replace);
        }
        ExprKind::AssignIndex(array, index, replace) => {
            propagate_expr(*array);
            propagate_expr(*index);
            propagate_expr(*replace);
        }
        ExprKind::Block(block) => propagate_applications_table_through_block(
            *block,
            applications_table,
            package,
            package_scaffolding,
        ),
        ExprKind::Call(callee, arg) => {
            propagate_expr(*callee);
            propagate_expr(*arg);
        }
        ExprKind::Fail(msg) => propagate_expr(*msg),
        ExprKind::Field(record, _) => propagate_expr(*record),
        ExprKind::If(cond, body, otherwise) => {
            propagate_expr(*cond);
            propagate_expr(*body);
            otherwise.iter().for_each(|e| propagate_expr(*e));
        }
        ExprKind::Index(array, index) => {
            propagate_expr(*array);
            propagate_expr(*index);
        }
        ExprKind::Return(expr) | ExprKind::UnOp(_, expr) => {
            propagate_expr(*expr);
        }
        ExprKind::Range(start, step, end) => {
            start.iter().for_each(|s| propagate_expr(*s));
            step.iter().for_each(|s| propagate_expr(*s));
            end.iter().for_each(|e| propagate_expr(*e));
        }
        ExprKind::String(components) => {
            for component in components {
                match component {
                    StringComponent::Expr(expr) => propagate_expr(*expr),
                    StringComponent::Lit(_) => {}
                }
            }
        }
        ExprKind::UpdateIndex(e1, e2, e3) => {
            propagate_expr(*e1);
            propagate_expr(*e2);
            propagate_expr(*e3);
        }
        ExprKind::Tuple(exprs) => exprs.iter().for_each(|e| propagate_expr(*e)),
        ExprKind::While(cond, block) => {
            propagate_expr(*cond);
            propagate_applications_table_through_block(
                *block,
                applications_table,
                package,
                package_scaffolding,
            );
        }
        ExprKind::Closure(_, _) | ExprKind::Hole | ExprKind::Lit(_) | ExprKind::Var(_, _) => {}
    }

    // Insert the applications table.
    package_scaffolding
        .exprs
        .insert(expr_id, applications_table.clone());
}

fn propagate_applications_table_through_stmt(
    stmt_id: StmtId,
    applications_table: &ApplicationsTable,
    package: &impl PackageLookup,
    package_scaffolding: &mut PackageScaffolding,
) {
    let stmt = package.get_stmt(stmt_id);
    match stmt.kind {
        StmtKind::Expr(expr_id) | StmtKind::Semi(expr_id) | StmtKind::Local(_, _, expr_id) => {
            propagate_applications_table_through_expr(
                expr_id,
                applications_table,
                package,
                package_scaffolding,
            )
        }
        StmtKind::Item(_) => {}
    }
    package_scaffolding
        .stmts
        .insert(stmt_id, applications_table.clone())
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
    if application_instance.blocks.get(&id.block).is_some() {
        panic!("block is already analyzed in application instance");
    }

    // Initialize the block's compute kind.
    let block = package_store.get_block(id);
    let mut block_compute_kind = ComputeKind::Classical;

    // Iterate through the block statements and aggregate the runtime features of each into the block's compute kind.
    for stmt_id in block.stmts.iter() {
        let store_stmt_id = StoreStmtId::from((id.package, *stmt_id));
        simulate_stmt(
            store_stmt_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        );
        let stmt_compute_kind = application_instance
            .stmts
            .get(stmt_id)
            .expect("statement's compute kind should exist");
        block_compute_kind =
            aggregate_compute_kind_runtime_features(block_compute_kind, stmt_compute_kind);
    }

    // Update the block's value kind if its non-unit.
    if block.ty != Ty::UNIT {
        let last_stmt_id = block
            .stmts
            .last()
            .expect("block should have at least one statement");
        let last_stmt_compute_kind = application_instance
            .stmts
            .get(last_stmt_id)
            .expect("statement's compute kind should exist");

        // We only have to update the block's value kind if the last statement's compute kind is quantum and dynamic.
        if let ComputeKind::Quantum(last_stmt_quantum_properties) = last_stmt_compute_kind {
            if let ValueKind::Dynamic = last_stmt_quantum_properties.value_kind {
                // If the block's last statement's compute kind is quantum, the block's compute kind must be quantum too.
                let ComputeKind::Quantum(block_quantum_properties) = &mut block_compute_kind else {
                    panic!("block's compute kind should be quantum");
                };

                // The block's value kind must be static since this is the first time a value kind is set.
                let ValueKind::Static = block_quantum_properties.value_kind else {
                    panic!("block's value kind should be static");
                };

                // Set the last statement as a source of dynamism for the block.
                block_quantum_properties.value_kind = ValueKind::Dynamic;
            }
        }
    }

    // Finally, we insert the block's compute kind to the application instance.
    application_instance
        .blocks
        .insert(id.block, block_compute_kind);
}

fn simulate_expr(
    id: StoreExprId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    let expr = package_store.get_expr(id);
    let mut compute_kind = match &expr.kind {
        ExprKind::Array(exprs) => determine_expr_array_compute_kind(
            exprs.iter().map(|e| StoreExprId::from((id.package, *e))),
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        ExprKind::ArrayRepeat(value_expr_id, size_expr_id) => {
            determine_expr_array_repeat_compute_kind(
                (id.package, *value_expr_id).into(),
                (id.package, *size_expr_id).into(),
                application_instance,
                package_store,
                package_store_scaffolding,
            )
        }
        ExprKind::Assign(lhs_expr_id, rhs_expr_id) => determine_expr_assign_compute_kind(
            (id.package, *lhs_expr_id).into(),
            (id.package, *rhs_expr_id).into(),
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        ExprKind::AssignField(lhs_expr_id, _, rhs_expr_id) => determine_expr_assign_compute_kind(
            (id.package, *lhs_expr_id).into(),
            (id.package, *rhs_expr_id).into(),
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        ExprKind::BinOp(_, lhs_expr_id, rhs_expr_id) => determine_expr_bin_op_compute_kind(
            (id.package, *lhs_expr_id).into(),
            (id.package, *rhs_expr_id).into(),
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        ExprKind::Block(block_id) => determine_expr_block_compute_kind(
            (id.package, *block_id).into(),
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        ExprKind::Call(callee, args) => determine_expr_call_compute_kind(
            (id.package, *callee).into(),
            (id.package, *args).into(),
            &expr.ty,
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        ExprKind::If(condition_expr_id, if_true_expr_id, if_false_expr_id) => {
            determine_expr_if_compute_kind(
                (id.package, *condition_expr_id).into(),
                (id.package, *if_true_expr_id).into(),
                if_false_expr_id.map(|e| StoreExprId::from((id.package, e))),
                &expr.ty,
                application_instance,
                package_store,
                package_store_scaffolding,
            )
        }
        ExprKind::Lit(_) => determine_expr_lit_compute_kind(),
        ExprKind::Tuple(exprs) => determine_expr_tuple_compute_kind(
            exprs.iter().map(|e| StoreExprId::from((id.package, *e))),
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        ExprKind::Var(res, _) => determine_expr_var_compute_kind(res, application_instance),
        // TODO (cesarzc): handle each case separately.
        _ => ComputeKind::Classical,
    };

    // If the expression's compute kind is dynamic, then additional runtime features might be needed depending on the
    // expression's type.
    if let ComputeKind::Quantum(quantum_properties) = &mut compute_kind {
        if let ValueKind::Dynamic = quantum_properties.value_kind {
            quantum_properties.runtime_features |=
                derive_runtime_features_for_dynamic_type(&expr.ty);
        }
    }

    // Finally, insert the expresion's compute kind in the application instance.
    application_instance.exprs.insert(id.expr, compute_kind);
}

fn simulate_stmt(
    id: StoreStmtId,
    application_instance: &mut ApplicationInstance,
    package_store: &PackageStore,
    package_store_scaffolding: &mut PackageStoreScaffolding,
) {
    let stmt = package_store.get_stmt(id);

    // Determine the statement's compute kind depending on its type.
    let compute_kind = match stmt.kind {
        StmtKind::Expr(expr_id) => determine_stmt_expr_compute_kind(
            id.package,
            expr_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        StmtKind::Semi(expr_id) => determine_stmt_semi_compute_kind(
            id.package,
            expr_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        StmtKind::Local(mutability, pat_id, expr_id) => determine_stmt_local_compute_kind(
            id.package,
            mutability,
            pat_id,
            expr_id,
            application_instance,
            package_store,
            package_store_scaffolding,
        ),
        StmtKind::Item(_) => {
            // An item statement does not have any inherent quantum properties, so we just treat it as classical compute.
            ComputeKind::Classical
        }
    };

    // Insert the statements's compute kind into the application instance.
    application_instance.stmts.insert(id.stmt, compute_kind);
}

fn split_controls_and_input(
    args_expr_id: ExprId,
    spec_functor: &SpecFunctor,
    package: &impl PackageLookup,
) -> (Vec<ExprId>, ExprId) {
    let mut controls = Vec::new();
    let mut remainder_expr_id = args_expr_id;
    for _ in 0..spec_functor.controlled {
        let expr = package.get_expr(remainder_expr_id);
        match &expr.kind {
            ExprKind::Tuple(pats) => {
                assert!(pats.len() == 2);
                controls.push(pats[0]);
                remainder_expr_id = pats[1];
            }
            _ => panic!("expected tuple expression"),
        }
    }
    (controls, remainder_expr_id)
}

fn update_locals_compute_kind(
    lhs_expr_id: ExprId,
    rhs_expr_id: ExprId,
    package: &impl PackageLookup,
    application_instance: &mut ApplicationInstance,
) {
    let lhs_expr = package.get_expr(lhs_expr_id);
    let rhs_expr = package.get_expr(rhs_expr_id);
    match &lhs_expr.kind {
        ExprKind::Var(res, _) => {
            let Res::Local(node_id) = res else {
                panic!("expected a local variable");
            };
            let rhs_compute_kind = application_instance
                .exprs
                .get(&rhs_expr_id)
                .expect("RHS compute kind should exist");
            application_instance
                .locals_map
                .aggregate_compute_kind(*node_id, rhs_compute_kind);
        }
        ExprKind::Tuple(lhs_exprs) => {
            let ExprKind::Tuple(rhs_exprs) = &rhs_expr.kind else {
                panic!("expected a tuple");
            };
            assert!(lhs_exprs.len() == rhs_exprs.len());
            for (lhs_expr_id, rhs_expr_id) in lhs_exprs.iter().zip(rhs_exprs.iter()) {
                update_locals_compute_kind(
                    *lhs_expr_id,
                    *rhs_expr_id,
                    package,
                    application_instance,
                );
            }
        }
        _ => panic!("expected a local variable or a tuple"),
    };
}
