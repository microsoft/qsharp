// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    applications::{
        ApplicationInstance, ApplicationInstanceIndex, GeneratorSetsBuilder, LocalComputeKind,
    },
    common::{derive_callable_input_params, GlobalSpecId, InputParam, Local, LocalKind},
    scaffolding::{ItemComputeProperties, PackageStoreComputeProperties},
    ApplicationGeneratorSet, ComputeKind, QuantumProperties, RuntimeFeatureFlags, ValueKind,
};
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{
        Block, BlockId, CallableDecl, CallableImpl, CallableKind, Expr, ExprId, ExprKind, Ident,
        Item, ItemKind, Mutability, Package, PackageId, PackageStore, PackageStoreLookup, Pat,
        PatId, PatKind, SpecDecl, SpecImpl, Stmt, StmtId, StmtKind, StoreItemId,
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

    fn analyze_spec_decl(&mut self, decl: &'a SpecDecl, functor_set_value: FunctorSetValue) {
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

    pub fn clear_current_application_index(&mut self) -> ApplicationInstanceIndex {
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

    pub fn set_current_application_index(&mut self, index: ApplicationInstanceIndex) {
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

    fn visit_item(&mut self, item: &'a Item) {
        let current_item_context = self.get_current_item_context();
        match &item.kind {
            ItemKind::Callable(decl) => {
                self.visit_callable_decl(decl);
            }
            ItemKind::Namespace(_, _) | ItemKind::Ty(_, _) => {
                self.package_store_compute_properties
                    .insert_item(current_item_context.id, ItemComputeProperties::NonCallable);
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
            StmtKind::Item(_) => {
                // An item statement does not have any inherent quantum properties, so we just treat it as classical compute.
                ComputeKind::Classical
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
