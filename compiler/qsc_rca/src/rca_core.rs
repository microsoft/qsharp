// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    applications::{
        ApplicationInstance, ApplicationInstanceIndex, GeneratorSetsBuilder, LocalComputeKind,
    },
    common::{
        aggregate_compute_kind, derive_callable_input_params, try_resolve_callee, Callee,
        FunctorAppExt, GlobalSpecId, InputParam, Local, LocalKind,
    },
    scaffolding::{ItemComputeProperties, PackageStoreComputeProperties},
    ApplicationGeneratorSet, ComputeKind, QuantumProperties, RuntimeFeatureFlags, ValueKind,
};
use qsc_data_structures::{functors::FunctorApp, index_map::IndexMap};
use qsc_fir::{
    fir::{
        Block, BlockId, CallableDecl, CallableImpl, CallableKind, Expr, ExprId, ExprKind, Global,
        Ident, Item, ItemKind, Mutability, Package, PackageId, PackageLookup, PackageStore,
        PackageStoreLookup, Pat, PatId, PatKind, Res, SpecDecl, SpecImpl, Stmt, StmtId, StmtKind,
        StoreExprId, StoreItemId, StorePatId, StringComponent,
    },
    ty::{FunctorSetValue, Prim, Ty},
    visit::Visitor,
};

pub struct CoreAnalyzer<'a> {
    package_store: &'a PackageStore,
    package_store_compute_properties: PackageStoreComputeProperties,
    active_items: Vec<ItemContext>,
}

impl<'a> CoreAnalyzer<'a> {
    pub fn new(
        package_store: &'a PackageStore,
        package_store_compute_properties: PackageStoreComputeProperties,
    ) -> Self {
        Self {
            package_store,
            package_store_compute_properties,
            active_items: Vec::<ItemContext>::default(),
        }
    }

    pub fn analyze_all(mut self) -> PackageStoreComputeProperties {
        for (package_id, package) in self.package_store {
            self.analyze_package_internal(package_id, package);
        }
        self.package_store_compute_properties
    }

    pub fn analyze_package(mut self, package_id: PackageId) -> PackageStoreComputeProperties {
        let package = self.package_store.get(package_id);
        self.analyze_package_internal(package_id, package);
        self.package_store_compute_properties
    }

    fn analyze_expr_array(&mut self, exprs: &Vec<ExprId>) -> ComputeKind {
        // Visit each sub-expression in the array to determine their compute kind, and aggregate ONLY the runtime
        // features to the array's compute kind.
        let mut compute_kind = ComputeKind::Classical;
        for expr_id in exprs {
            self.visit_expr(*expr_id);
            let application_instance = self.get_current_application_instance();
            let expr_compute_kind = application_instance.get_expr_compute_kind(*expr_id);
            compute_kind =
                aggregate_compute_kind_runtime_features(compute_kind, *expr_compute_kind);
        }

        // The value kind of an array expression itself cannot be dynamic since its size is always known.
        assert!(!compute_kind.is_value_dynamic());
        compute_kind
    }

    fn analyze_expr_array_repeat(
        &mut self,
        value_expr_id: ExprId,
        size_expr_id: ExprId,
    ) -> ComputeKind {
        // Visit the value and size expressions to determine their compute kind.
        self.visit_expr(value_expr_id);
        self.visit_expr(size_expr_id);

        // The runtime features the array repeat expression is determined by aggregating the runtime features of both
        // the size and value expressions.
        let application_instance = self.get_current_application_instance();
        let size_expr_compute_kind = *application_instance.get_expr_compute_kind(size_expr_id);
        let value_expr_compute_kind = *application_instance.get_expr_compute_kind(value_expr_id);
        let mut compute_kind = ComputeKind::Classical;
        compute_kind =
            aggregate_compute_kind_runtime_features(compute_kind, size_expr_compute_kind);
        compute_kind =
            aggregate_compute_kind_runtime_features(compute_kind, value_expr_compute_kind);

        // An array repeat expression is dynamic if its size is dynamic.
        if size_expr_compute_kind.is_value_dynamic() {
            compute_kind.aggregate_value_kind(ValueKind::Dynamic);
        }

        compute_kind
    }

    fn analyze_expr_assign(
        &mut self,
        assignee_expr_id: ExprId,
        value_expr_id: ExprId,
    ) -> ComputeKind {
        // Visit the assignee and value expressions to determine their compute kind.
        self.visit_expr(assignee_expr_id);
        self.visit_expr(value_expr_id);

        // Since this is an assignment, update the local variables on the assignee expression with the compute kind of
        // the value expression.
        self.update_locals_compute_kind(assignee_expr_id, value_expr_id);

        // The compute kind is determined by the aggregated runtime features of the assignee and value expressions.
        // We do not care about the value kind for this kind of expression because it is an assignment.
        let application_instance = self.get_current_application_instance();
        let lhs_compute_kind = *application_instance.get_expr_compute_kind(value_expr_id);
        let rhs_compute_kind = *application_instance.get_expr_compute_kind(value_expr_id);
        let mut compute_kind = ComputeKind::Classical;
        compute_kind = aggregate_compute_kind_runtime_features(compute_kind, lhs_compute_kind);
        compute_kind = aggregate_compute_kind_runtime_features(compute_kind, rhs_compute_kind);
        compute_kind
    }

    fn analyze_expr_assign_index(
        &mut self,
        array_expr_id: ExprId,
        index_expr_id: ExprId,
        replacement_value_expr_id: ExprId,
    ) -> ComputeKind {
        // Visit the array, index and replacement value expressions to determine their compute kind.
        self.visit_expr(array_expr_id);
        self.visit_expr(index_expr_id);
        self.visit_expr(replacement_value_expr_id);

        // Since this is an assignment, the compute kind of the local variable (assignee expression) needs to be updated
        // with the appropriate compute kind. This is determined by aggregating the compute kind of the replacement
        // value expression, the compute kind of the index expression and an additional runtime feature if the index
        // expression is dynamic.
        let application_instance = self.get_current_application_instance();
        let array_compute_kind = *application_instance.get_expr_compute_kind(array_expr_id);
        let index_compute_kind = *application_instance.get_expr_compute_kind(index_expr_id);
        let replacement_value_compute_kind =
            *application_instance.get_expr_compute_kind(replacement_value_expr_id);
        let mut assigned_value_compute_kind = ComputeKind::Classical;
        assigned_value_compute_kind =
            aggregate_compute_kind(assigned_value_compute_kind, replacement_value_compute_kind);
        assigned_value_compute_kind =
            aggregate_compute_kind(assigned_value_compute_kind, index_compute_kind);
        if index_compute_kind.is_value_dynamic() {
            assigned_value_compute_kind = aggregate_compute_kind_runtime_features(
                assigned_value_compute_kind,
                ComputeKind::with_runtime_features(RuntimeFeatureFlags::UseOfDynamicIndex),
            );
        }

        // Update compute kind of the local variable in the locals map.
        let lhs_expr = self.get_expr(array_expr_id);
        let ExprKind::Var(Res::Local(node_id), _) = &lhs_expr.kind else {
            panic!("LHS expression should be a local");
        };
        let application_instance = self.get_current_application_instance_mut();
        application_instance
            .locals_map
            .aggregate_compute_kind(*node_id, assigned_value_compute_kind);

        // The compute kind of this expression is the aggregated runtime features of the array expression and the
        // assigned value. We do not care about the value kind for this kind of expression because it is an assignment.
        let mut compute_kind = ComputeKind::Classical;
        compute_kind = aggregate_compute_kind_runtime_features(compute_kind, array_compute_kind);
        compute_kind =
            aggregate_compute_kind_runtime_features(compute_kind, assigned_value_compute_kind);
        compute_kind
    }

    fn analyze_expr_assign_op(
        &mut self,
        assignee_expr_id: ExprId,
        compund_value_expr_id: ExprId,
    ) -> ComputeKind {
        // Visit the assignee and compound value expressions to determine their compute kind.
        self.visit_expr(assignee_expr_id);
        self.visit_expr(compund_value_expr_id);

        // Since this is an assignment, the compute kind of the local variable (assignee expression) needs to be updated
        // with the appropriate compute kind. This is determined from the compute kind of the compound value expression.
        // However, this is done slightly differently depending on whether the assignee is an array or not.
        let application_instance = self.get_current_application_instance();
        let assignee_compute_kind = *application_instance.get_expr_compute_kind(assignee_expr_id);
        let compound_value_expr = self.get_expr(compund_value_expr_id);
        let assigned_value_compute_kind = if let Ty::Array(_) = compound_value_expr.ty {
            // If this is an array concatenation within a dynamic scope, then its is considered a dynamic array.
            let mut assigned_value_compute_kind =
                *application_instance.get_expr_compute_kind(compund_value_expr_id);
            if !application_instance.active_dynamic_scopes.is_empty() {
                assigned_value_compute_kind = aggregate_compute_kind(
                    assigned_value_compute_kind,
                    ComputeKind::Quantum(QuantumProperties {
                        runtime_features: RuntimeFeatureFlags::UseOfDynamicArray,
                        value_kind: ValueKind::Dynamic,
                    }),
                );
            }
            assigned_value_compute_kind
        } else {
            // If this is not an array, the compute kind of the assigned value is just the compute kind of the RHS
            // expression.
            *application_instance.get_expr_compute_kind(compund_value_expr_id)
        };

        // Update compute kind of the local variable in the locals map.
        let lhs_expr = self.get_expr(assignee_expr_id);
        let ExprKind::Var(Res::Local(node_id), _) = &lhs_expr.kind else {
            panic!("LHS expression should be a local");
        };
        let application_instance = self.get_current_application_instance_mut();
        application_instance
            .locals_map
            .aggregate_compute_kind(*node_id, assigned_value_compute_kind);

        // The compute kind of this expression is the aggregated runtime features of the assignee expression and the
        // assigned value. We do not care about the value kind for this kind of expression because it is an assignment.
        let mut compute_kind = ComputeKind::Classical;
        compute_kind = aggregate_compute_kind_runtime_features(compute_kind, assignee_compute_kind);
        compute_kind =
            aggregate_compute_kind_runtime_features(compute_kind, assigned_value_compute_kind);
        compute_kind
    }

    fn analyze_expr_bin_op(&mut self, lhs_expr_id: ExprId, rhs_expr_id: ExprId) -> ComputeKind {
        // Visit the LHS and RHS expressions to determine their compute kind.
        self.visit_expr(lhs_expr_id);
        self.visit_expr(rhs_expr_id);

        // The compute kind of a binary operator expression is the aggregation of its LHS and RHS expressions.
        let application_instance = self.get_current_application_instance();
        let lhs_compute_kind = *application_instance.get_expr_compute_kind(lhs_expr_id);
        let rhs_compute_kind = *application_instance.get_expr_compute_kind(rhs_expr_id);
        let mut compute_kind = ComputeKind::Classical;
        compute_kind = aggregate_compute_kind(compute_kind, lhs_compute_kind);
        compute_kind = aggregate_compute_kind(compute_kind, rhs_compute_kind);
        compute_kind
    }

    fn analyze_expr_block(&mut self, block_id: BlockId) -> ComputeKind {
        // Visit the block to determine its compute kind.
        self.visit_block(block_id);

        // The compute kind of a block expression is the same as the compute kind of the block.
        let application_instance = self.get_current_application_instance();
        *application_instance.get_block_compute_kind(block_id)
    }

    fn analyze_expr_call(
        &mut self,
        callee_expr_id: ExprId,
        args_expr_id: ExprId,
        expr_type: &Ty,
    ) -> ComputeKind {
        // Visit the callee and arguments expressions to determine their compute kind.
        self.visit_expr(callee_expr_id);
        self.visit_expr(args_expr_id);

        // The compute kind of this expression depends on whether the callee expression is dynamic.
        let application_instance = self.get_current_application_instance();
        let callee_expr_compute_kind = *application_instance.get_expr_compute_kind(callee_expr_id);
        let mut compute_kind = if callee_expr_compute_kind.is_value_dynamic() {
            // The value kind of a call expression with a dynamic callee depends on whether the expression type is Unit.
            let value_kind = if *expr_type == Ty::UNIT {
                ValueKind::Static
            } else {
                ValueKind::Dynamic
            };
            ComputeKind::Quantum(QuantumProperties {
                runtime_features: RuntimeFeatureFlags::DynamicCallee,
                value_kind,
            })
        } else {
            self.analyze_expr_call_with_static_callee(callee_expr_id, args_expr_id, expr_type)
        };

        // If this call happens within a dynamic scope, there might be additional runtime features being used.
        let application_instance = self.get_current_application_instance();
        if !application_instance.active_dynamic_scopes.is_empty() {
            // Any call that happens within a dynamic scope uses the forward branching runtime feature.
            compute_kind = aggregate_compute_kind_runtime_features(
                compute_kind,
                ComputeKind::with_runtime_features(
                    RuntimeFeatureFlags::ForwardBranchingOnDynamicValue,
                ),
            );

            // If the call expression type is either a result or a qubit, it uses dynamic allocation runtime features.
            if let Ty::Prim(Prim::Qubit) = expr_type {
                compute_kind = aggregate_compute_kind_runtime_features(
                    compute_kind,
                    ComputeKind::with_runtime_features(RuntimeFeatureFlags::DynamicQubitAllocation),
                );
            }
            if let Ty::Prim(Prim::Result) = expr_type {
                compute_kind = aggregate_compute_kind_runtime_features(
                    compute_kind,
                    ComputeKind::with_runtime_features(
                        RuntimeFeatureFlags::DynamicResultAllocation,
                    ),
                );
            }
        }

        // Aggregate the runtime features of the callee and arguments expressions.
        let callee_expr_compute_kind = *application_instance.get_expr_compute_kind(callee_expr_id);
        let args_expr_compute_kind = *application_instance.get_expr_compute_kind(args_expr_id);
        compute_kind =
            aggregate_compute_kind_runtime_features(compute_kind, callee_expr_compute_kind);
        compute_kind =
            aggregate_compute_kind_runtime_features(compute_kind, args_expr_compute_kind);
        compute_kind
    }

    fn analyze_expr_call_with_spec_callee(
        &mut self,
        callee: &Callee,
        callable_decl: &'a CallableDecl,
        args_expr_id: ExprId,
    ) -> ComputeKind {
        // Analyze the specialization to determine its application generator set.
        let callee_id = GlobalSpecId::from((callee.item, callee.functor_app.functor_set_value()));
        self.analyze_spec(callee_id, callable_decl);
        let application_generator_set = self.package_store_compute_properties.get_spec(callee_id);

        // We need to split controls and specialization input arguments so we can derive the correct callable
        // application.
        let package_id = self.get_current_package_id();
        let args_package = self.package_store.get(package_id);
        let (args_controls, args_input_id) =
            split_controls_and_input(args_expr_id, callee.functor_app, args_package);

        // To map the input pattern to input expressions we need to provide global (store-level) pattern and expression
        // identifiers since the callable can be in a different package than the input expressions.
        let callee_input_pattern_id =
            StorePatId::from((callee_id.callable.package, callable_decl.input));
        let args_input_id = StoreExprId::from((package_id, args_input_id));
        let input_exprs = map_input_pattern_to_input_expressions(
            callee_input_pattern_id,
            args_input_id,
            self.package_store,
        );
        let application_instance = self.get_current_application_instance();
        let input_params_dynamism: Vec<bool> = input_exprs
            .iter()
            .map(|expr_id| {
                let input_param_compute_kind = application_instance.get_expr_compute_kind(*expr_id);
                input_param_compute_kind.is_value_dynamic()
            })
            .collect();

        // Derive the compute kind based on the dynamism of input parameters.
        let mut compute_kind =
            application_generator_set.derive_application_compute_kind(&input_params_dynamism);

        // Aggreagate the compute properties of the qubit controls expressions.
        for control_expr in args_controls {
            let control_expr_compute_kind =
                *application_instance.get_expr_compute_kind(control_expr);

            if callable_decl.output == Ty::UNIT {
                // If the callable output is unit, just aggregate the runtime features because in this case we do not
                // care about the value kind.
                compute_kind = aggregate_compute_kind_runtime_features(
                    compute_kind,
                    control_expr_compute_kind,
                );
            } else {
                // If the callable output is not unit, aggregate the compute kind because we care about the value kind
                // in this case.
                compute_kind = aggregate_compute_kind(compute_kind, control_expr_compute_kind);
            }
        }

        compute_kind
    }

    fn analyze_expr_call_with_static_callee(
        &mut self,
        callee_expr_id: ExprId,
        args_expr_id: ExprId,
        expr_type: &Ty,
    ) -> ComputeKind {
        // Try to resolve the callee.
        let package_id = self.get_current_package_id();
        let package = self.package_store.get(package_id);
        let application_instance = self.get_current_application_instance();
        let maybe_callee = try_resolve_callee(
            callee_expr_id,
            package_id,
            package,
            &application_instance.locals_map,
        );

        // If the callee could not be resolved, return a compute kind with certain runtime features.
        let Some(callee) = maybe_callee else {
            // Assume a dynamic value kind if the output of the call expression is non-unit.
            let value_kind = if *expr_type == Ty::UNIT {
                ValueKind::Static
            } else {
                ValueKind::Dynamic
            };
            return ComputeKind::Quantum(QuantumProperties {
                runtime_features: RuntimeFeatureFlags::UnresolvedCallee,
                value_kind,
            });
        };

        // We could resolve the callee. Determine the compute kind of the call depending on the callee kind.
        let global_callee = self
            .package_store
            .get_global(callee.item)
            .expect("global should exist");
        match global_callee {
            Global::Callable(callable_decl) => {
                self.analyze_expr_call_with_spec_callee(&callee, callable_decl, args_expr_id)
            }
            Global::Udt => self.analyze_expr_call_with_udt_callee(args_expr_id),
        }
    }

    fn analyze_expr_call_with_udt_callee(&self, args_expr_id: ExprId) -> ComputeKind {
        let application_instance = self.get_current_application_instance();
        let args_expr_compute_kind = application_instance.get_expr_compute_kind(args_expr_id);
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

    fn analyze_expr_fail(&mut self, msg_expr_id: ExprId) -> ComputeKind {
        // Visit the message expression to determine its compute kind.
        self.visit_expr(msg_expr_id);

        // The compute kind of the expression is determined from the message expression runtime features plus an
        // additional runtime feature if the message expresion is dynamic.
        let application_instance = self.get_current_application_instance();
        let msg_expr_compute_kind = *application_instance.get_expr_compute_kind(msg_expr_id);
        let mut compute_kind = ComputeKind::Classical;
        compute_kind = aggregate_compute_kind_runtime_features(compute_kind, msg_expr_compute_kind);
        if msg_expr_compute_kind.is_value_dynamic() {
            compute_kind = aggregate_compute_kind_runtime_features(
                compute_kind,
                ComputeKind::with_runtime_features(
                    RuntimeFeatureFlags::FailureWithDynamicExpression,
                ),
            );
        }

        compute_kind
    }

    fn analyze_expr_field(&mut self, record_expr_id: ExprId) -> ComputeKind {
        // Visit the record expression to determine its compute kind.
        self.visit_expr(record_expr_id);

        // The compute kind of the field expression is the same as the compute kind of the record expression.
        let application_instance = self.get_current_application_instance();
        *application_instance.get_expr_compute_kind(record_expr_id)
    }

    fn analyze_expr_if(
        &mut self,
        condition_expr_id: ExprId,
        body_expr_id: ExprId,
        otherwise_expr_id: Option<ExprId>,
        expr_type: &Ty,
    ) -> ComputeKind {
        // Visit the condition expression to determine its compute kind.
        self.visit_expr(condition_expr_id);

        // If the condition expression is dynamic, we push a new dynamic scope.
        let application_instance = self.get_current_application_instance_mut();
        let condition_expr_compute_kind =
            *application_instance.get_expr_compute_kind(condition_expr_id);
        let within_dynamic_scope = condition_expr_compute_kind.is_value_dynamic();
        if within_dynamic_scope {
            application_instance
                .active_dynamic_scopes
                .push(condition_expr_id);
        }

        // Visit the body and otherwise expressions to determine their compute kind.
        self.visit_expr(body_expr_id);
        otherwise_expr_id.iter().for_each(|e| self.visit_expr(*e));

        // Pop the dynamic scope.
        if within_dynamic_scope {
            let application_instance = self.get_current_application_instance_mut();
            let dynamic_scope_expr_id = application_instance
                .active_dynamic_scopes
                .pop()
                .expect("at least one dynamic scope should exist");
            assert!(dynamic_scope_expr_id == condition_expr_id);
        }

        // Aggregate the runtime features of the sub-expressions.
        let application_instance = self.get_current_application_instance();
        let mut compute_kind = ComputeKind::Classical;
        let condition_expr_compute_kind =
            *application_instance.get_expr_compute_kind(condition_expr_id);
        compute_kind =
            aggregate_compute_kind_runtime_features(compute_kind, condition_expr_compute_kind);
        let body_expr_compute_kind = *application_instance.get_expr_compute_kind(body_expr_id);
        compute_kind = aggregate_compute_kind(compute_kind, body_expr_compute_kind);
        if let Some(otherwise_expr_id) = otherwise_expr_id {
            let otherwise_expr_compute_kind =
                *application_instance.get_expr_compute_kind(otherwise_expr_id);
            compute_kind =
                aggregate_compute_kind_runtime_features(compute_kind, otherwise_expr_compute_kind);
        }

        // If the expression's type is non-unit and any of the sub-expressions is dynamic, then the compute kind of an
        // if-expression is dynamic.
        if *expr_type != Ty::UNIT {
            let is_any_sub_expr_dynamic = condition_expr_compute_kind.is_value_dynamic()
                || body_expr_compute_kind.is_value_dynamic()
                || otherwise_expr_id.map_or(false, |e| {
                    application_instance
                        .get_expr_compute_kind(e)
                        .is_value_dynamic()
                });
            let value_kind = if is_any_sub_expr_dynamic {
                ValueKind::Dynamic
            } else {
                ValueKind::Static
            };
            compute_kind.aggregate_value_kind(value_kind);
        }

        compute_kind
    }

    fn analyze_expr_index(&mut self, array_expr_id: ExprId, index_expr_id: ExprId) -> ComputeKind {
        // Visit the array and index expressions to determine their compute kind.
        self.visit_expr(array_expr_id);
        self.visit_expr(index_expr_id);

        // We use the array expression runtime features as the basis for the compute kind of the whole index accessor
        // expression.
        let application_instance = self.get_current_application_instance();
        let array_expr_compute_kind = *application_instance.get_expr_compute_kind(array_expr_id);
        let mut compute_kind = match array_expr_compute_kind {
            ComputeKind::Classical => ComputeKind::Classical,
            ComputeKind::Quantum(quantum_properties) => ComputeKind::Quantum(QuantumProperties {
                runtime_features: quantum_properties.runtime_features,
                // Since we do not track the compute kind of individual array elements, we assume that any element is
                // dynamic if the compute kind of the array is quantum.
                value_kind: ValueKind::Dynamic,
            }),
        };

        // Aggregate the index's compute kind before returning the index accessor expression compute kind.
        let index_expr_compute_kind = *application_instance.get_expr_compute_kind(index_expr_id);
        compute_kind = aggregate_compute_kind(compute_kind, index_expr_compute_kind);
        compute_kind
    }

    fn analyze_expr_range(
        &mut self,
        start_expr_id: Option<ExprId>,
        step_expr_id: Option<ExprId>,
        end_expr_id: Option<ExprId>,
    ) -> ComputeKind {
        // Visit the start, step and end expressions to determine their compute kind.
        start_expr_id.iter().for_each(|e| self.visit_expr(*e));
        step_expr_id.iter().for_each(|e| self.visit_expr(*e));
        end_expr_id.iter().for_each(|e| self.visit_expr(*e));

        // The compute kind of a range expression is the aggregation of its start, step and end expressions.
        let application_instance = self.get_current_application_instance();
        let start_expr_compute_kind = start_expr_id.map_or(ComputeKind::Classical, |e| {
            *application_instance.get_expr_compute_kind(e)
        });
        let step_expr_compute_kind = step_expr_id.map_or(ComputeKind::Classical, |e| {
            *application_instance.get_expr_compute_kind(e)
        });
        let end_expr_compute_kind = end_expr_id.map_or(ComputeKind::Classical, |e| {
            *application_instance.get_expr_compute_kind(e)
        });
        let mut compute_kind = ComputeKind::Classical;
        compute_kind = aggregate_compute_kind(compute_kind, start_expr_compute_kind);
        compute_kind = aggregate_compute_kind(compute_kind, step_expr_compute_kind);
        compute_kind = aggregate_compute_kind(compute_kind, end_expr_compute_kind);
        compute_kind
    }

    fn analyze_expr_return(&mut self, value_expr_id: ExprId) -> ComputeKind {
        // Visit the value expression to determine its compute kind.
        self.visit_expr(value_expr_id);

        // Add the value expression ID to the return expressions tracked by the application instance.
        let application_instance = self.get_current_application_instance_mut();
        let value_expression_compute_kind =
            *application_instance.get_expr_compute_kind(value_expr_id);
        application_instance.return_expressions.push(value_expr_id);

        // The compute kind of the return expression itself consists of only the runtime features of the value
        // expression.
        let mut compute_kind = ComputeKind::Classical;
        compute_kind =
            aggregate_compute_kind_runtime_features(compute_kind, value_expression_compute_kind);
        compute_kind
    }

    fn analyze_expr_string(&mut self, components: &Vec<StringComponent>) -> ComputeKind {
        // Visit the string components to determine their compute kind and aggregate it to determine the compute kind
        // of the whole string expression.
        let mut compute_kind = ComputeKind::Classical;
        for component in components {
            match component {
                StringComponent::Expr(expr_id) => {
                    self.visit_expr(*expr_id);
                    let application_instance = self.get_current_application_instance();
                    let component_compute_kind =
                        *application_instance.get_expr_compute_kind(*expr_id);
                    compute_kind = aggregate_compute_kind(compute_kind, component_compute_kind);
                }
                StringComponent::Lit(_) => {
                    // Nothing to aggregate.
                }
            }
        }
        compute_kind
    }

    fn analyze_expr_tuple(&mut self, exprs: &Vec<ExprId>) -> ComputeKind {
        // Visit the sub-expressions to determine their compute kind and aggregate it to determine the compute kind of
        // the tuple.
        let mut compute_kind = ComputeKind::Classical;
        for expr_id in exprs {
            self.visit_expr(*expr_id);
            let application_instance = self.get_current_application_instance();
            let expr_compute_kind = *application_instance.get_expr_compute_kind(*expr_id);
            compute_kind = aggregate_compute_kind(compute_kind, expr_compute_kind);
        }
        compute_kind
    }

    fn analyze_expr_un_op(&mut self, operand_expr_id: ExprId) -> ComputeKind {
        // Visit the operand expression to determine its compute kind.
        self.visit_expr(operand_expr_id);

        // The compute kind of an unary expression is the same as the compute kind of its operand expression.
        let application_instance = self.get_current_application_instance();
        *application_instance.get_expr_compute_kind(operand_expr_id)
    }

    fn analyze_expr_update_field(
        &mut self,
        record_expr_id: ExprId,
        replace_expr_id: ExprId,
    ) -> ComputeKind {
        // Visit the record and replace expressions to determine their compute kind.
        self.visit_expr(record_expr_id);
        self.visit_expr(replace_expr_id);

        // The compute kind of an update field expression is just the aggregation of its record and replace expressions.
        let application_instance = self.get_current_application_instance();
        let record_expr_compute_kind = *application_instance.get_expr_compute_kind(record_expr_id);
        let replace_expr_compute_kind =
            *application_instance.get_expr_compute_kind(replace_expr_id);
        let mut compute_kind = ComputeKind::Classical;
        compute_kind = aggregate_compute_kind(compute_kind, record_expr_compute_kind);
        compute_kind = aggregate_compute_kind(compute_kind, replace_expr_compute_kind);
        compute_kind
    }

    fn analyze_expr_update_index(
        &mut self,
        array_expr_id: ExprId,
        index_expr_id: ExprId,
        replacement_value_expr_id: ExprId,
    ) -> ComputeKind {
        // Visit the array, index and replacement value expressions to determine their compute kind.
        self.visit_expr(array_expr_id);
        self.visit_expr(index_expr_id);
        self.visit_expr(replacement_value_expr_id);

        // The compute kind of an update index expression is the aggregation of its sub-expressions, with some nuanced
        // considerations.
        let application_instance = self.get_current_application_instance();
        let array_expr_compute_kind = *application_instance.get_expr_compute_kind(array_expr_id);
        let index_expr_compute_kind = *application_instance.get_expr_compute_kind(index_expr_id);
        let replacement_value_expr_compute_kind =
            *application_instance.get_expr_compute_kind(replacement_value_expr_id);
        let mut compute_kind = ComputeKind::Classical;

        // We fully aggregate the array compute kind and not just its runtime features because the updated array will be
        // dynamic if the original one was.
        compute_kind = aggregate_compute_kind(compute_kind, array_expr_compute_kind);

        // When aggregating the runtime features of the index we include an additional runtime feature if the index
        // expression is dynamic.
        compute_kind =
            aggregate_compute_kind_runtime_features(compute_kind, index_expr_compute_kind);
        if index_expr_compute_kind.is_value_dynamic() {
            compute_kind = aggregate_compute_kind_runtime_features(
                compute_kind,
                ComputeKind::with_runtime_features(RuntimeFeatureFlags::UseOfDynamicIndex),
            );
        }

        // Regarding the replacement value expression, we just aggregate its runtime features.
        compute_kind = aggregate_compute_kind_runtime_features(
            compute_kind,
            replacement_value_expr_compute_kind,
        );

        compute_kind
    }

    fn analyze_expr_var(&self, res: &Res) -> ComputeKind {
        match res {
            // Global items do not have quantum properties by themselves so we can consider them classical.
            Res::Item(_) => ComputeKind::Classical,
            // Gather the current compute kind of the local.
            Res::Local(local_var_id) => {
                let application_instance = self.get_current_application_instance();
                *application_instance
                    .locals_map
                    .get_compute_kind(*local_var_id)
            }
            Res::Err => panic!("unexpected error resolution"),
        }
    }

    fn analyze_expr_while(&mut self, condition_expr_id: ExprId, block_id: BlockId) -> ComputeKind {
        // Visit the condition expression to determine its compute kind.
        self.visit_expr(condition_expr_id);

        // If the condition expression is dynamic, we push a new dynamic scope before visiting the block.
        let application_instance = self.get_current_application_instance_mut();
        let condition_expr_compute_kind =
            *application_instance.get_expr_compute_kind(condition_expr_id);
        let within_dynamic_scope = condition_expr_compute_kind.is_value_dynamic();
        if within_dynamic_scope {
            application_instance
                .active_dynamic_scopes
                .push(condition_expr_id);
        }
        self.visit_block(block_id);
        if within_dynamic_scope {
            let application_instance = self.get_current_application_instance_mut();
            let dynamic_scope_expr_id = application_instance
                .active_dynamic_scopes
                .pop()
                .expect("at least one dynamic scope should exist");
            assert!(dynamic_scope_expr_id == condition_expr_id);
        }

        // Return the aggregated runtime features of the condition expression and the block.
        let application_instance = self.get_current_application_instance();
        let block_compute_kind = *application_instance.get_block_compute_kind(block_id);
        let mut compute_kind = ComputeKind::Classical;
        compute_kind =
            aggregate_compute_kind_runtime_features(compute_kind, condition_expr_compute_kind);
        compute_kind = aggregate_compute_kind_runtime_features(compute_kind, block_compute_kind);
        compute_kind
    }

    // Analyzes the currently active callable assuming it is intrinsic.
    fn analyze_intrinsic_callable(&mut self) {
        // Check whether the callable has already been analyzed.
        let current_item_context = self.get_current_item_context();
        let body_specialization_id =
            GlobalSpecId::from((current_item_context.id, FunctorSetValue::Empty));
        if self
            .package_store_compute_properties
            .find_specialization(body_specialization_id)
            .is_some()
        {
            return;
        }

        // Determine the application generator set depending on whether the callable is a function or an operation.
        let decl_info = current_item_context.get_callable_context();
        let application_generator_set = match decl_info.kind {
            CallableKind::Function => {
                derive_intrinsic_function_application_generator_set(decl_info)
            }
            CallableKind::Operation => {
                derive_instrinsic_operation_application_generator_set(decl_info)
            }
        };

        // Insert the generator set in the entry corresponding to the body specialization of the callable.
        self.package_store_compute_properties
            .insert_spec(body_specialization_id, application_generator_set);
    }

    fn analyze_item(&mut self, item_id: StoreItemId, item: &'a Item) {
        self.push_active_item_context(item_id);
        self.visit_item(item);
        let popped_item_id = self.pop_active_item_context();
        assert!(popped_item_id == item_id);
    }

    fn analyze_package_internal(&mut self, package_id: PackageId, package: &'a Package) {
        for (local_item_id, item) in &package.items {
            self.analyze_item((package_id, local_item_id).into(), item);
        }
    }

    fn analyze_spec(&mut self, id: GlobalSpecId, callable_decl: &'a CallableDecl) {
        // Only do this if the specialization has not been analyzed already.
        if self
            .package_store_compute_properties
            .find_specialization(id)
            .is_some()
        {
            return;
        }

        // Push the context of the callable the specialization belongs to.
        self.push_active_item_context(id.callable);
        let input_params = derive_callable_input_params(
            callable_decl,
            &self.package_store.get(id.callable.package).pats,
        );
        let current_callable_context = self.get_current_item_context_mut();
        current_callable_context.set_callable_context(
            callable_decl.kind,
            input_params,
            callable_decl.output.clone(),
        );

        // Continue with the analysis differently depending on whether the callable is an intrinsic or not.
        match &callable_decl.implementation {
            CallableImpl::Intrinsic => self.analyze_intrinsic_callable(),
            CallableImpl::Spec(spec_impl) => {
                // Only analyze the specialization that corresponds to the provided ID. Otherwise, we can get into an
                // infinite analysis loop.
                let spec_decl = match id.functor_set_value {
                    FunctorSetValue::Empty => &spec_impl.body,
                    FunctorSetValue::Adj => spec_impl
                        .adj
                        .as_ref()
                        .expect("adj specialization should exist"),
                    FunctorSetValue::Ctl => spec_impl
                        .ctl
                        .as_ref()
                        .expect("ctl specialization should exist"),
                    FunctorSetValue::CtlAdj => spec_impl
                        .ctl_adj
                        .as_ref()
                        .expect("ctladj specialization should exist"),
                };
                self.analyze_spec_decl(spec_decl, id.functor_set_value);
            }
        };

        // Since we are done analyzing the specialization, pop the active item context.
        let popped_item_id = self.pop_active_item_context();
        assert!(popped_item_id == id.callable);
    }

    fn analyze_spec_decl(&mut self, decl: &'a SpecDecl, functor_set_value: FunctorSetValue) {
        // Only do this if the specialization has not been analyzed already.
        let current_item_context = self.get_current_item_context();
        let global_spec_id = GlobalSpecId::from((current_item_context.id, functor_set_value));
        if self
            .package_store_compute_properties
            .find_specialization(global_spec_id)
            .is_some()
        {
            return;
        }

        // Set the context for the specialization declaration, visit it and then clear the context to get the results
        // of the analysis.
        let package_id = self.get_current_package_id();
        self.set_current_spec_context(decl, functor_set_value);
        self.visit_spec_decl(decl);
        let spec_context = self.clear_current_spec_context();
        assert!(spec_context.functor_set_value == functor_set_value);

        // Save the analysis to the corresponding package compute properties.
        let package_compute_properties = self.package_store_compute_properties.get_mut(package_id);
        spec_context
            .builder
            .save_to_package_compute_properties(package_compute_properties, Some(decl.block));
    }

    fn bind_compute_kind_to_ident(
        &mut self,
        pat: &Pat,
        ident: &Ident,
        local_kind: LocalKind,
        compute_kind: ComputeKind,
    ) {
        let application_instance = self.get_current_application_instance_mut();
        let local = Local {
            var: ident.id,
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

    fn bind_expr_compute_kind_to_pattern(
        &mut self,
        mutability: Mutability,
        pat_id: PatId,
        expr_id: ExprId,
    ) {
        let expr = self.get_expr(expr_id);
        let pat = self.get_pat(pat_id);
        match &pat.kind {
            PatKind::Bind(ident) => {
                let application_instance = self.get_current_application_instance();
                let compute_kind = *application_instance.get_expr_compute_kind(expr_id);
                let local_kind = match mutability {
                    Mutability::Immutable => LocalKind::Immutable(expr_id),
                    Mutability::Mutable => LocalKind::Mutable,
                };
                self.bind_compute_kind_to_ident(pat, ident, local_kind, compute_kind);
            }
            PatKind::Tuple(pats) => match &expr.kind {
                ExprKind::Tuple(exprs) => {
                    for (pat_id, expr_id) in pats.iter().zip(exprs.iter()) {
                        self.bind_expr_compute_kind_to_pattern(mutability, *pat_id, *expr_id);
                    }
                }
                _ => {
                    self.bind_fixed_expr_compute_kind_to_pattern(mutability, pat_id, expr_id);
                }
            },
            PatKind::Discard => {
                // Nothing to bind to.
            }
        }
    }

    fn bind_fixed_expr_compute_kind_to_pattern(
        &mut self,
        mutability: Mutability,
        pat_id: PatId,
        expr_id: ExprId,
    ) {
        let pat = self.get_pat(pat_id);
        match &pat.kind {
            PatKind::Bind(ident) => {
                let application_instance = self.get_current_application_instance();
                let compute_kind = *application_instance.get_expr_compute_kind(expr_id);
                let local_kind = match mutability {
                    Mutability::Immutable => LocalKind::Immutable(expr_id),
                    Mutability::Mutable => LocalKind::Mutable,
                };
                self.bind_compute_kind_to_ident(pat, ident, local_kind, compute_kind);
            }
            PatKind::Tuple(pats) => {
                for pat_id in pats {
                    self.bind_fixed_expr_compute_kind_to_pattern(mutability, *pat_id, expr_id);
                }
            }
            PatKind::Discard => {
                // Nothing to bind to.
            }
        }
    }

    fn clear_current_application_index(&mut self) -> ApplicationInstanceIndex {
        self.get_current_spec_context_mut()
            .clear_current_application_index()
    }

    fn clear_current_spec_context(&mut self) -> SpecContext {
        self.get_current_item_context_mut()
            .clear_current_spec_context()
    }

    fn get_current_application_instance(&self) -> &ApplicationInstance {
        self.get_current_spec_context()
            .get_current_application_instance()
    }

    fn get_current_application_instance_mut(&mut self) -> &mut ApplicationInstance {
        self.get_current_spec_context_mut()
            .get_current_application_instance_mut()
    }

    fn get_current_item_context(&self) -> &ItemContext {
        self.active_items.last().expect("there are no active items")
    }

    fn get_current_item_context_mut(&mut self) -> &mut ItemContext {
        self.active_items
            .last_mut()
            .expect("there are no active items")
    }

    fn get_current_spec_context(&self) -> &SpecContext {
        self.get_current_item_context().get_current_spec_context()
    }

    fn get_current_spec_context_mut(&mut self) -> &mut SpecContext {
        self.get_current_item_context_mut()
            .get_current_spec_context_mut()
    }

    fn get_current_package_id(&self) -> PackageId {
        self.get_current_item_context().id.package
    }

    fn pop_active_item_context(&mut self) -> StoreItemId {
        self.active_items
            .pop()
            .expect("there are no active items")
            .id
    }

    fn push_active_item_context(&mut self, id: StoreItemId) {
        self.active_items.push(ItemContext::new(id));
    }

    fn set_current_application_index(&mut self, index: ApplicationInstanceIndex) {
        self.get_current_spec_context_mut()
            .set_current_application_index(index);
    }

    fn set_current_spec_context(&mut self, decl: &'a SpecDecl, functor_set_value: FunctorSetValue) {
        assert!(self
            .get_current_item_context()
            .current_spec_context
            .is_none());
        let package_id = self.get_current_package_id();
        let pats = &self.package_store.get(package_id).pats;
        let input_params = self.get_current_item_context().get_input_params();
        let controls = derive_specialization_controls(decl, pats);
        let spec_context = SpecContext::new(functor_set_value, input_params, controls.as_ref());
        self.get_current_item_context_mut()
            .set_current_spec_context(spec_context);
    }

    fn update_locals_compute_kind(&mut self, lhs_expr_id: ExprId, rhs_expr_id: ExprId) {
        let lhs_expr = self.get_expr(lhs_expr_id);
        let rhs_expr = self.get_expr(rhs_expr_id);
        match &lhs_expr.kind {
            ExprKind::Var(res, _) => {
                let Res::Local(node_id) = res else {
                    panic!("expected a local variable");
                };
                let application_instance = self.get_current_application_instance_mut();
                let rhs_compute_kind = *application_instance.get_expr_compute_kind(rhs_expr_id);
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
                    self.update_locals_compute_kind(*lhs_expr_id, *rhs_expr_id);
                }
            }
            _ => panic!("expected a local variable or a tuple"),
        };
    }
}

impl<'a> Visitor<'a> for CoreAnalyzer<'a> {
    fn get_block(&self, id: BlockId) -> &'a Block {
        let package_id = self.get_current_package_id();
        self.package_store.get_block((package_id, id).into())
    }

    fn get_expr(&self, id: ExprId) -> &'a Expr {
        let package_id = self.get_current_package_id();
        self.package_store.get_expr((package_id, id).into())
    }

    fn get_pat(&self, id: PatId) -> &'a Pat {
        let package_id = self.get_current_package_id();
        self.package_store.get_pat((package_id, id).into())
    }

    fn get_stmt(&self, id: StmtId) -> &'a Stmt {
        let package_id = self.get_current_package_id();
        self.package_store.get_stmt((package_id, id).into())
    }

    fn visit_block(&mut self, block_id: BlockId) {
        // Visiting a block always happens in the context of an application instance.
        let block = self.get_block(block_id);

        // Visit each statement in the block and aggregate its compute kind.
        let mut block_compute_kind = ComputeKind::Classical;
        for stmt_id in &block.stmts {
            // Visiting a statement performs its analysis for the current application instance.
            self.visit_stmt(*stmt_id);

            // Now, we can query the statement's compute kind and aggregate it to the block's compute kind.
            let application_instance = self.get_current_application_instance();
            let stmt_compute_kind = application_instance.get_stmt_compute_kind(*stmt_id);
            block_compute_kind =
                aggregate_compute_kind_runtime_features(block_compute_kind, *stmt_compute_kind);
        }

        // Update the block's value kind if its non-unit.
        if block.ty != Ty::UNIT {
            let last_stmt_id = block
                .stmts
                .last()
                .expect("block should have at least one statement");
            let application_instance = self.get_current_application_instance();
            let last_stmt_compute_kind = application_instance.get_stmt_compute_kind(*last_stmt_id);
            if last_stmt_compute_kind.is_value_dynamic() {
                // If the block's last statement's compute kind is dynamic, the block's compute kind must be quantum.
                let ComputeKind::Quantum(block_quantum_properties) = &mut block_compute_kind else {
                    panic!("block's compute kind should be quantum");
                };

                // The block's value kind must be static since this is the first time a value kind is set.
                assert!(
                    matches!(block_quantum_properties.value_kind, ValueKind::Static),
                    "block's value kind should be static"
                );

                // Set the block value kind as dynamic.
                block_quantum_properties.value_kind = ValueKind::Dynamic;
            }
        }

        // Finally, insert the block's compute kind to the application instance.
        let application_instance = self.get_current_application_instance_mut();
        application_instance.insert_block_compute_kind(block_id, block_compute_kind);
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        let package_id = self.get_current_package_id();

        // Derive the input parameters of the callable and add them to the currently active callable.
        let input_params =
            derive_callable_input_params(decl, &self.package_store.get(package_id).pats);
        let current_callable_context = self.get_current_item_context_mut();
        current_callable_context.set_callable_context(decl.kind, input_params, decl.output.clone());
        self.visit_callable_impl(&decl.implementation);
    }

    fn visit_callable_impl(&mut self, callable_impl: &'a CallableImpl) {
        match callable_impl {
            CallableImpl::Intrinsic => self.analyze_intrinsic_callable(),
            CallableImpl::Spec(spec_impl) => {
                self.visit_spec_impl(spec_impl);
            }
        };
    }

    fn visit_expr(&mut self, expr_id: ExprId) {
        let expr = self.get_expr(expr_id);
        let mut compute_kind = match &expr.kind {
            ExprKind::Array(exprs) => self.analyze_expr_array(exprs),
            ExprKind::ArrayRepeat(value_expr_id, size_expr_id) => {
                self.analyze_expr_array_repeat(*value_expr_id, *size_expr_id)
            }
            ExprKind::Assign(assignee_expr_id, value_expr_id)
            | ExprKind::AssignField(assignee_expr_id, _, value_expr_id) => {
                self.analyze_expr_assign(*assignee_expr_id, *value_expr_id)
            }
            ExprKind::AssignIndex(array_expr_id, index_expr_id, replacement_value_expr_id) => self
                .analyze_expr_assign_index(
                    *array_expr_id,
                    *index_expr_id,
                    *replacement_value_expr_id,
                ),
            ExprKind::AssignOp(_, assignee_expr_id, compound_value_expr_id) => {
                self.analyze_expr_assign_op(*assignee_expr_id, *compound_value_expr_id)
            }
            ExprKind::BinOp(_, lhs_expr_id, rhs_expr_id) => {
                self.analyze_expr_bin_op(*lhs_expr_id, *rhs_expr_id)
            }
            ExprKind::Block(block_id) => self.analyze_expr_block(*block_id),
            ExprKind::Call(callee_expr_id, args_expr_id) => {
                let expr = self.get_expr(expr_id);
                self.analyze_expr_call(*callee_expr_id, *args_expr_id, &expr.ty)
            }
            ExprKind::Closure(_, _) => {
                ComputeKind::with_runtime_features(RuntimeFeatureFlags::Closure)
            }
            ExprKind::Fail(msg_expr_id) => self.analyze_expr_fail(*msg_expr_id),
            ExprKind::Field(record_expr_id, _) => self.analyze_expr_field(*record_expr_id),
            ExprKind::Hole | ExprKind::Lit(_) => {
                // Hole and literal expressions are purely classical.
                ComputeKind::Classical
            }
            ExprKind::If(condition_expr_id, body_expr_id, otherwise_expr_id) => {
                let expr = self.get_expr(expr_id);
                self.analyze_expr_if(
                    *condition_expr_id,
                    *body_expr_id,
                    otherwise_expr_id.to_owned(),
                    &expr.ty,
                )
            }
            ExprKind::Index(array_expr_id, index_expr_id) => {
                self.analyze_expr_index(*array_expr_id, *index_expr_id)
            }
            ExprKind::Range(start_expr_id, step_expr_id, end_expr_id) => self.analyze_expr_range(
                start_expr_id.to_owned(),
                step_expr_id.to_owned(),
                end_expr_id.to_owned(),
            ),
            ExprKind::Return(value_expr_id) => self.analyze_expr_return(*value_expr_id),
            ExprKind::String(components) => self.analyze_expr_string(components),
            ExprKind::Tuple(exprs) => self.analyze_expr_tuple(exprs),
            ExprKind::UnOp(_, operand_expr_id) => self.analyze_expr_un_op(*operand_expr_id),
            ExprKind::UpdateField(record_expr_id, _, replace_expr_id) => {
                self.analyze_expr_update_field(*record_expr_id, *replace_expr_id)
            }
            ExprKind::UpdateIndex(array_expr_id, index_expr_id, replacement_value_expr_id) => self
                .analyze_expr_update_index(
                    *array_expr_id,
                    *index_expr_id,
                    *replacement_value_expr_id,
                ),
            ExprKind::Var(res, _) => self.analyze_expr_var(res),
            ExprKind::While(condition_expr_id, block_id) => {
                self.analyze_expr_while(*condition_expr_id, *block_id)
            }
        };

        // If the expression's compute kind is dynamic, then additional runtime features might be needed depending on
        // the expression's type.
        if let ComputeKind::Quantum(quantum_properties) = &mut compute_kind {
            if let ValueKind::Dynamic = quantum_properties.value_kind {
                quantum_properties.runtime_features |=
                    derive_runtime_features_for_dynamic_type(&expr.ty);
            }
        }

        // Finally, insert the expresion's compute kind in the application instance.
        let application_instance = self.get_current_application_instance_mut();
        application_instance.insert_expr_compute_kind(expr_id, compute_kind);
    }

    fn visit_item(&mut self, item: &'a Item) {
        let current_item_context = self.get_current_item_context();
        match &item.kind {
            ItemKind::Namespace(_, _) | ItemKind::Ty(_, _) => {
                self.package_store_compute_properties
                    .insert_item(current_item_context.id, ItemComputeProperties::NonCallable);
            }
            ItemKind::Callable(decl) => {
                self.visit_callable_decl(decl);
            }
        };
    }

    fn visit_package(&mut self, _: &'a Package) {
        // Should never be called.
        unimplemented!("should never be called");
    }

    fn visit_pat(&mut self, _: PatId) {
        // Do nothing.
    }

    fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
        // Determine the compute properties of the specialization by visiting the implementation block configured for
        // each application instance in the generator set.
        let input_param_count = self.get_current_item_context().get_input_params().len();
        let start_index = -1i32;
        let end_index = i32::try_from(input_param_count).expect("could not compute end index");
        let applications = (start_index..end_index).map(ApplicationInstanceIndex::from);
        for application_index in applications {
            self.set_current_application_index(application_index);
            self.visit_block(decl.block);
            let cleared_index = self.clear_current_application_index();
            assert!(cleared_index == application_index);
        }
    }

    fn visit_spec_impl(&mut self, spec_impl: &'a SpecImpl) {
        self.analyze_spec_decl(&spec_impl.body, FunctorSetValue::Empty);
        spec_impl
            .adj
            .iter()
            .for_each(|spec_decl| self.analyze_spec_decl(spec_decl, FunctorSetValue::Adj));
        spec_impl
            .ctl
            .iter()
            .for_each(|spec_decl| self.analyze_spec_decl(spec_decl, FunctorSetValue::Ctl));
        spec_impl
            .ctl_adj
            .iter()
            .for_each(|spec_decl| self.analyze_spec_decl(spec_decl, FunctorSetValue::CtlAdj));
    }

    fn visit_stmt(&mut self, stmt_id: StmtId) {
        let stmt = self.get_stmt(stmt_id);
        let compute_kind = match &stmt.kind {
            StmtKind::Expr(expr_id) => {
                // Visit the expression to determine its compute kind.
                self.visit_expr(*expr_id);

                // The statement's compute kind is the same as the expression's compute kind.
                let application_instance = self.get_current_application_instance();
                *application_instance.get_expr_compute_kind(*expr_id)
            }
            StmtKind::Semi(expr_id) => {
                // Visit the expression to determine its compute kind.
                self.visit_expr(*expr_id);

                // Use the expression compute kind to construct the statement compute kind, using only the expression
                // runtime features since the value kind is meaningless for semicolon statements.
                let application_instance = self.get_current_application_instance();
                let expr_compute_kind = application_instance.get_expr_compute_kind(*expr_id);
                aggregate_compute_kind_runtime_features(ComputeKind::Classical, *expr_compute_kind)
            }
            StmtKind::Local(mutability, pat_id, value_expr_id) => {
                // Visit the expression to determine its compute kind.
                self.visit_expr(*value_expr_id);

                // Bind the expression's compute kind to the pattern.
                self.bind_expr_compute_kind_to_pattern(*mutability, *pat_id, *value_expr_id);

                // Use the expression compute kind to construct the statement compute kind, using only the expression
                // runtime features since the value kind is meaningless for local (binding) statements.
                let application_instance = self.get_current_application_instance();
                let expr_compute_kind = application_instance.get_expr_compute_kind(*value_expr_id);
                aggregate_compute_kind_runtime_features(ComputeKind::Classical, *expr_compute_kind)
            }
            StmtKind::Item(_) => {
                // An item statement does not have any inherent quantum properties, so we just treat it as classical compute.
                ComputeKind::Classical
            }
        };

        // Insert the statements's compute kind into the application instance.
        let application_instance = self.get_current_application_instance_mut();
        application_instance.insert_stmt_compute_kind(stmt_id, compute_kind);
    }
}

struct ItemContext {
    pub id: StoreItemId,
    callable_context: Option<CallableContext>,
    current_spec_context: Option<SpecContext>,
}

impl ItemContext {
    pub fn new(id: StoreItemId) -> Self {
        Self {
            id,
            callable_context: None,
            current_spec_context: None,
        }
    }

    pub fn clear_current_spec_context(&mut self) -> SpecContext {
        self.current_spec_context
            .take()
            .expect("current specialization context has already been cleared")
    }

    pub fn get_current_spec_context(&self) -> &SpecContext {
        self.current_spec_context
            .as_ref()
            .expect("current specialization context is not set")
    }

    pub fn get_current_spec_context_mut(&mut self) -> &mut SpecContext {
        self.current_spec_context
            .as_mut()
            .expect("current specialization context is not set")
    }

    pub fn get_callable_context(&self) -> &CallableContext {
        self.callable_context
            .as_ref()
            .expect("callable declaration context should not be none")
    }

    pub fn get_input_params(&self) -> &Vec<InputParam> {
        &self.get_callable_context().input_params
    }

    pub fn set_callable_context(
        &mut self,
        kind: CallableKind,
        input_params: Vec<InputParam>,
        output_type: Ty,
    ) {
        assert!(self.callable_context.is_none());
        self.callable_context = Some(CallableContext {
            kind,
            input_params,
            output_type,
        });
    }

    pub fn set_current_spec_context(&mut self, spec_context: SpecContext) {
        assert!(self.current_spec_context.is_none());
        self.current_spec_context = Some(spec_context);
    }
}

struct CallableContext {
    pub kind: CallableKind,
    pub input_params: Vec<InputParam>,
    pub output_type: Ty,
}

struct SpecContext {
    functor_set_value: FunctorSetValue,
    builder: GeneratorSetsBuilder,
    current_application_index: Option<ApplicationInstanceIndex>,
}

impl SpecContext {
    pub fn new(
        functor_set_value: FunctorSetValue,
        input_params: &Vec<InputParam>,
        controls: Option<&Local>,
    ) -> Self {
        let builder = GeneratorSetsBuilder::new(input_params, controls);
        Self {
            functor_set_value,
            builder,
            current_application_index: None,
        }
    }

    pub fn clear_current_application_index(&mut self) -> ApplicationInstanceIndex {
        self.current_application_index
            .take()
            .expect("appication instance index is not set")
    }

    pub fn get_current_application_instance(&self) -> &ApplicationInstance {
        let index = self.get_current_application_index();
        self.builder.get_application_instance(index)
    }

    pub fn get_current_application_instance_mut(&mut self) -> &mut ApplicationInstance {
        let index = self.get_current_application_index();
        self.builder.get_application_instance_mut(index)
    }

    pub fn set_current_application_index(&mut self, index: ApplicationInstanceIndex) {
        assert!(self.current_application_index.is_none());
        self.current_application_index = Some(index);
    }

    fn get_current_application_index(&self) -> ApplicationInstanceIndex {
        self.current_application_index
            .expect("application instance index is not set")
    }
}

#[must_use]
fn aggregate_compute_kind_runtime_features(basis: ComputeKind, delta: ComputeKind) -> ComputeKind {
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

fn derive_intrinsic_function_application_generator_set(
    callable_context: &CallableContext,
) -> ApplicationGeneratorSet {
    assert!(matches!(callable_context.kind, CallableKind::Function));

    // Determine the compute kind for all dynamic parameter applications.
    let mut dynamic_param_applications = Vec::new();
    for param in &callable_context.input_params {
        // For intrinsic functions, we assume any parameter can contribute to the output, so if any parameter is dynamic
        // the output of the function is dynamic. Therefore, for all dynamic parameters, if the function's output is
        // non-unit their value kind is dynamic.
        let value_kind = if callable_context.output_type == Ty::UNIT {
            ValueKind::Static
        } else {
            ValueKind::Dynamic
        };

        let param_compute_kind = ComputeKind::Quantum(QuantumProperties {
            // When a parameter is bound to a dynamic value, its type contributes to the runtime features used by the
            // function application.
            runtime_features: derive_runtime_features_for_dynamic_type(&param.ty),
            value_kind,
        });
        dynamic_param_applications.push(param_compute_kind);
    }

    ApplicationGeneratorSet {
        // Functions are inherently classical.
        inherent: ComputeKind::Classical,
        dynamic_param_applications,
    }
}

fn derive_instrinsic_operation_application_generator_set(
    callable_context: &CallableContext,
) -> ApplicationGeneratorSet {
    assert!(matches!(callable_context.kind, CallableKind::Operation));

    // The value kind of intrinsic operations is inherently dynamic if their output is not `Unit` or `Qubit`.
    let value_kind = if callable_context.output_type == Ty::UNIT
        || callable_context.output_type == Ty::Prim(Prim::Qubit)
    {
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
    for param in &callable_context.input_params {
        // For intrinsic operations, we assume any parameter can contribute to the output, so if any parameter is
        // dynamic the output of the operation is dynamic. Therefore, for all dynamic parameters, if the operation's
        // output is non-unit it becomes a source of dynamism.
        let value_kind = if callable_context.output_type == Ty::UNIT {
            ValueKind::Static
        } else {
            ValueKind::Dynamic
        };

        // The compute kind of intrinsic operations is always quantum.
        let param_compute_kind = ComputeKind::Quantum(QuantumProperties {
            // When a parameter is bound to a dynamic value, its type contributes to the runtime features used by the
            // operation application.
            runtime_features: derive_runtime_features_for_dynamic_type(&param.ty),
            value_kind,
        });
        dynamic_param_applications.push(param_compute_kind);
    }

    ApplicationGeneratorSet {
        inherent: inherent_compute_kind,
        dynamic_param_applications,
    }
}

fn derive_runtime_features_for_dynamic_type(ty: &Ty) -> RuntimeFeatureFlags {
    fn intrinsic_runtime_features_from_primitive_type(prim: Prim) -> RuntimeFeatureFlags {
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
        Ty::Prim(prim) => intrinsic_runtime_features_from_primitive_type(*prim),
        Ty::Tuple(tuple) => intrinsic_runtime_features_from_tuple(tuple),
        // Runtime features can be more nuanced by taking into account the contained types.
        Ty::Udt(_) => RuntimeFeatureFlags::UseOfDynamicUdt,
        Ty::Err => panic!("cannot derive runtime features for `Err` type"),
    }
}

fn derive_specialization_controls(
    spec_decl: &SpecDecl,
    pats: &IndexMap<PatId, Pat>,
) -> Option<Local> {
    spec_decl.input.and_then(|pat_id| {
        let pat = pats.get(pat_id).expect("pat should exist");
        match &pat.kind {
            PatKind::Bind(ident) => Some(Local {
                var: ident.id,
                pat: pat_id,
                ty: pat.ty.clone(),
                kind: LocalKind::SpecInput,
            }),
            PatKind::Discard => None, // Nothing to bind to.
            PatKind::Tuple(_) => panic!("expected specialization input pattern"),
        }
    })
}

/// Maps an input pattern to a list of expressions that correspond to identifiers or discards.
fn map_input_pattern_to_input_expressions(
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
                        let mut sub_input_param_exprs = map_input_pattern_to_input_expressions(
                            global_pat_id,
                            global_expr_id,
                            package_store,
                        );
                        input_param_exprs.append(&mut sub_input_param_exprs);
                    }
                    input_param_exprs
                }
                _ => panic!("expected tuple expression"),
            }
        }
    }
}

fn split_controls_and_input(
    args_expr_id: ExprId,
    functor_app: FunctorApp,
    package: &impl PackageLookup,
) -> (Vec<ExprId>, ExprId) {
    let mut controls = Vec::new();
    let mut remainder_expr_id = args_expr_id;
    for _ in 0..functor_app.controlled {
        let expr = package.get_expr(remainder_expr_id);
        let ExprKind::Tuple(pats) = &expr.kind else {
            panic!("expected tuple expression");
        };
        assert!(pats.len() == 2);
        controls.push(pats[0]);
        remainder_expr_id = pats[1];
    }
    (controls, remainder_expr_id)
}
