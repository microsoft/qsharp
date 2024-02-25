// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    common::{derive_callable_input_params, GlobalSpecId, InputParam},
    scaffolding::{ItemComputeProperties, PackageStoreComputeProperties},
    ApplicationGeneratorSet, ComputeKind, QuantumProperties, RuntimeFeatureFlags, ValueKind,
};
use qsc_fir::{
    fir::{
        Block, BlockId, CallableDecl, CallableImpl, CallableKind, Expr, ExprId, Item, ItemKind,
        Package, PackageId, PackageStore, PackageStoreLookup, Pat, PatId, Stmt, StmtId,
        StoreItemId,
    },
    ty::{FunctorSetValue, Prim, Ty},
    visit::Visitor,
};

pub struct CoreAnalyzer<'a> {
    package_store: &'a PackageStore,
    package_store_compute_properties: PackageStoreComputeProperties,
    active_packages: Vec<PackageId>,
    active_callables: Vec<CallableContext>,
}

impl<'a> CoreAnalyzer<'a> {
    pub fn new(
        package_store: &'a PackageStore,
        package_store_compute_properties: PackageStoreComputeProperties,
    ) -> Self {
        Self {
            package_store,
            package_store_compute_properties,
            active_packages: Vec::<PackageId>::default(),
            active_callables: Vec::<CallableContext>::default(),
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
        let current_callable_context = self.get_current_callable_context();
        let decl_info = current_callable_context.get_decl_info();

        // Determine the application generator set depending on whether the callable is a function or an operation.
        let application_generator_set = match decl_info.kind {
            CallableKind::Function => {
                determine_intrinsic_function_application_generator_set(decl_info)
            }
            CallableKind::Operation => {
                determine_instrinsic_operation_application_generator_set(decl_info)
            }
        };

        // Insert the generator set in the entry corresponding to the body specialization of the callable.
        let body_specialization_id =
            GlobalSpecId::from((current_callable_context.id, FunctorSetValue::Empty));
        self.package_store_compute_properties
            .insert_spec(body_specialization_id, application_generator_set);
    }

    fn analyze_package_internal(&mut self, package_id: PackageId, package: &'a Package) {
        self.active_packages.push(package_id);
        self.visit_package(package);
        let popped_package_id = self
            .active_packages
            .pop()
            .expect("at least one package should be active");
        assert!(package_id == popped_package_id);
    }

    fn get_current_callable_context(&self) -> &CallableContext {
        self.active_callables
            .last()
            .expect("there are no active callables")
    }

    fn get_current_callable_context_mut(&mut self) -> &mut CallableContext {
        self.active_callables
            .last_mut()
            .expect("there are no active callables")
    }

    fn get_current_package(&self) -> PackageId {
        *self
            .active_packages
            .last()
            .expect("there are no active packages")
    }
}

impl<'a> Visitor<'a> for CoreAnalyzer<'a> {
    fn get_block(&self, id: BlockId) -> &'a Block {
        let package_id = self.get_current_package();
        self.package_store.get_block((package_id, id).into())
    }

    fn get_expr(&self, id: ExprId) -> &'a Expr {
        let package_id = self.get_current_package();
        self.package_store.get_expr((package_id, id).into())
    }

    fn get_pat(&self, _: PatId) -> &'a Pat {
        // Should never be used.
        unimplemented!()
    }

    fn get_stmt(&self, id: StmtId) -> &'a Stmt {
        let package_id = self.get_current_package();
        self.package_store.get_stmt((package_id, id).into())
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        let package_id = self.get_current_package();

        // Derive the input parameters of the callable and add them to the currently active callable.
        let input_params =
            derive_callable_input_params(decl, &self.package_store.get(package_id).pats);
        let current_callable_context = self.get_current_callable_context_mut();
        current_callable_context.set_decl_info(decl.kind, input_params, decl.output.clone());
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
        let package_id = self.get_current_package();
        let store_item_id = StoreItemId::from((package_id, item.id));
        match &item.kind {
            ItemKind::Callable(decl) => {
                self.active_callables
                    .push(CallableContext::new(store_item_id));
                self.visit_callable_decl(decl);
                let popped_callable_context = self
                    .active_callables
                    .pop()
                    .expect("there should be at least one active callable");
                assert!(popped_callable_context.id == store_item_id);
            }
            ItemKind::Namespace(_, _) | ItemKind::Ty(_, _) => {
                self.package_store_compute_properties
                    .insert_item(store_item_id, ItemComputeProperties::NonCallable);
            }
        };
    }

    fn visit_package(&mut self, package: &'a Package) {
        // First, analyze all top-level items.
        package.items.values().for_each(|i| self.visit_item(i));
    }

    fn visit_pat(&mut self, _: PatId) {
        // Do nothing.
    }
}

struct CallableContext {
    pub id: StoreItemId,
    decl_info: Option<CallableDeclContext>,
}

impl CallableContext {
    pub fn new(id: StoreItemId) -> Self {
        Self {
            id,
            decl_info: None,
        }
    }

    pub fn get_decl_info(&self) -> &CallableDeclContext {
        self.decl_info
            .as_ref()
            .expect("callable declaration context should not be none")
    }

    pub fn set_decl_info(
        &mut self,
        kind: CallableKind,
        input_params: Vec<InputParam>,
        output_type: Ty,
    ) {
        assert!(self.decl_info.is_none());
        self.decl_info = Some(CallableDeclContext {
            kind,
            input_params,
            output_type,
        });
    }
}

struct CallableDeclContext {
    pub kind: CallableKind,
    pub input_params: Vec<InputParam>,
    pub output_type: Ty,
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

fn determine_intrinsic_function_application_generator_set(
    callable_decl_context: &CallableDeclContext,
) -> ApplicationGeneratorSet {
    assert!(matches!(callable_decl_context.kind, CallableKind::Function));

    // Determine the compute kind for all dynamic parameter applications.
    let mut dynamic_param_applications = Vec::new();
    for param in &callable_decl_context.input_params {
        // For intrinsic functions, we assume any parameter can contribute to the output, so if any parameter is dynamic
        // the output of the function is dynamic. Therefore, for all dynamic parameters, if the function's output is
        // non-unit their value kind is dynamic.
        let value_kind = if callable_decl_context.output_type == Ty::UNIT {
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

fn determine_instrinsic_operation_application_generator_set(
    callable_decl_context: &CallableDeclContext,
) -> ApplicationGeneratorSet {
    assert!(matches!(
        callable_decl_context.kind,
        CallableKind::Operation
    ));

    // The value kind of intrinsic operations is inherently dynamic if their output is not `Unit` or `Qubit`.
    let value_kind = if callable_decl_context.output_type == Ty::UNIT
        || callable_decl_context.output_type == Ty::Prim(Prim::Qubit)
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
    for param in &callable_decl_context.input_params {
        // For intrinsic operations, we assume any parameter can contribute to the output, so if any parameter is
        // dynamic the output of the operation is dynamic. Therefore, for all dynamic parameters, if the operation's
        // output is non-unit it becomes a source of dynamism.
        let value_kind = if callable_decl_context.output_type == Ty::UNIT {
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
