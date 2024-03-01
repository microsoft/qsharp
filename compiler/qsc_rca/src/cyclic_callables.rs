// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    common::{derive_callable_input_params, LocalSpecId},
    cycle_detection::CycleDetector,
    scaffolding::PackageStoreComputeProperties,
    ApplicationGeneratorSet, ComputeKind, QuantumProperties, RuntimeFeatureFlags, ValueKind,
};
use qsc_fir::{
    fir::{
        Block, BlockId, CallableDecl, CallableImpl, CallableKind, Expr, ExprId, Global, Item,
        Package, PackageId, PackageStore, PackageStoreLookup, Pat, PatId, SpecImpl, Stmt, StmtId,
    },
    ty::{FunctorSetValue, Ty},
    visit::{self, Visitor},
};

pub struct Analyzer<'a> {
    package_store: &'a PackageStore,
    package_store_compute_properties: PackageStoreComputeProperties,
    current_package: Option<PackageId>,
    current_application_generator_set: Option<ApplicationGeneratorSet>,
}

impl<'a> Analyzer<'a> {
    pub fn new(
        package_store: &'a PackageStore,
        package_store_compute_properties: PackageStoreComputeProperties,
    ) -> Self {
        Self {
            package_store,
            package_store_compute_properties,
            current_package: None,
            current_application_generator_set: None,
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

    fn analyze_cyclic_specialization(&mut self, spec_id: LocalSpecId) {
        let package_id = self.get_current_package();
        let Some(Global::Callable(callable)) = self
            .package_store
            .get_global((package_id, spec_id.callable).into())
        else {
            panic!("global item should exist and it should be a global");
        };

        let CallableImpl::Spec(spec_impl) = &callable.implementation else {
            panic!("callable implementation should not be intrinsic");
        };

        // Create the application generator set differently depending on whether the callable is a function or an
        // operation.
        let input_params =
            derive_callable_input_params(callable, &self.package_store.get(package_id).pats);
        let application_generator_set = match callable.kind {
            CallableKind::Function => {
                Self::create_function_specialization_application_generator_set(
                    input_params.len(),
                    &callable.output,
                )
            }
            CallableKind::Operation => {
                Self::create_operation_specialization_application_generator_set(
                    input_params.len(),
                    &callable.output,
                )
            }
        };

        // Find the specialization.
        let spec_decl = match spec_id.functor_set_value {
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
                .expect("ctl_adj specializatiob should exist"),
        };

        // First visit the specialization to propagate the application generator set throughout all the relevant
        // sub-elements.
        // Then, insert the application generator set into the package store compute properties data structure.
        self.current_application_generator_set = Some(application_generator_set);
        self.visit_spec_decl(spec_decl);
        self.package_store_compute_properties.insert_spec(
            (package_id, spec_id).into(),
            self.get_current_application_generator_set().clone(),
        );
        self.current_application_generator_set = None;
    }

    fn analyze_package_internal(&mut self, package_id: PackageId, package: &'a Package) {
        self.current_package = Some(package_id);
        self.visit_package(package);
        self.current_package = None;
    }

    fn create_function_specialization_application_generator_set(
        callable_input_params_count: usize,
        output_type: &Ty,
    ) -> ApplicationGeneratorSet {
        // Set the compute kind of the function for each parameter when it is bound to a dynamic value.
        let mut using_dynamic_param = Vec::new();
        for _ in 0..callable_input_params_count {
            // If any parameter is dynamic, we assume the value of a function with cycles is a a source of dynamism if its
            // output type is non-unit.
            let value_kind = if *output_type == Ty::UNIT {
                ValueKind::Static
            } else {
                ValueKind::Dynamic
            };

            // Since using cyclic functions with dynamic parameters requires advanced runtime capabilities, we use a runtime
            // feature for these cases.
            let quantum_properties = QuantumProperties {
                runtime_features: RuntimeFeatureFlags::CyclicFunctionUsesDynamicArg,
                value_kind,
            };
            using_dynamic_param.push(ComputeKind::Quantum(quantum_properties));
        }

        ApplicationGeneratorSet {
            // Functions are inherently classically pure.
            inherent: ComputeKind::Classical,
            dynamic_param_applications: using_dynamic_param,
        }
    }

    fn create_operation_specialization_application_generator_set(
        callable_input_params_count: usize,
        output_type: &Ty,
    ) -> ApplicationGeneratorSet {
        // Since operations can allocate and measure qubits freely, we assume its compute kind is quantum, requires all
        // capabilities (encompassed by the `CyclicOperation` runtime feature) and that their value kind is dynamic.
        let value_kind = if *output_type == Ty::UNIT {
            ValueKind::Static
        } else {
            ValueKind::Dynamic
        };
        let inherent_compute_kind = ComputeKind::Quantum(QuantumProperties {
            runtime_features: RuntimeFeatureFlags::CyclicOperation,
            value_kind,
        });

        // The compute kind of a cyclic function when any of its parameters is bound to a dynamic value is the same as its
        // inherent compute kind.
        let mut using_dynamic_param = Vec::new();
        for _ in 0..callable_input_params_count {
            using_dynamic_param.push(inherent_compute_kind);
        }

        ApplicationGeneratorSet {
            inherent: inherent_compute_kind,
            dynamic_param_applications: using_dynamic_param,
        }
    }

    fn get_current_application_generator_set(&self) -> &ApplicationGeneratorSet {
        self.current_application_generator_set
            .as_ref()
            .expect("current application generator set should be valid")
    }

    fn get_current_package(&self) -> PackageId {
        self.current_package
            .expect("current package should be valid")
    }
}

impl<'a> Visitor<'a> for Analyzer<'a> {
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

    fn visit_callable_decl(&mut self, _: &'a CallableDecl) {
        // Should never be used.
        unimplemented!();
    }

    fn visit_callable_impl(&mut self, _: &'a CallableImpl) {
        // Should never be used.
        unimplemented!();
    }

    fn visit_expr(&mut self, expr: ExprId) {
        // First, visit the expression as it would normally be done.
        visit::walk_expr(self, expr);

        // Then, insert the application generator set into the package store compute properties data structure.
        let package_id = self.get_current_package();
        let application_generator_set = self.get_current_application_generator_set().clone();
        self.package_store_compute_properties
            .insert_expr((package_id, expr).into(), application_generator_set);
    }

    fn visit_item(&mut self, _: &'a Item) {
        // Should never be used.
        unimplemented!();
    }

    fn visit_package(&mut self, package: &'a Package) {
        let package_id = self.get_current_package();
        let cycle_detector = CycleDetector::new(package_id, package);
        let specializations_with_cycles = cycle_detector.detect_specializations_with_cycles();
        for cyclic_specialization in specializations_with_cycles {
            self.analyze_cyclic_specialization(cyclic_specialization);
        }
    }

    fn visit_block(&mut self, block: BlockId) {
        // First, visit the block like it would normally be done.
        visit::walk_block(self, block);

        // Then, insert the application generator set into the package store compute properties data structure.
        let package_id = self.get_current_package();
        let application_generator_set = self.get_current_application_generator_set().clone();
        self.package_store_compute_properties
            .insert_block((package_id, block).into(), application_generator_set);
    }

    fn visit_pat(&mut self, _: PatId) {
        // Do nothing.
    }

    fn visit_spec_impl(&mut self, _: &'a SpecImpl) {
        // Should never be used.
        unimplemented!();
    }

    fn visit_stmt(&mut self, stmt: StmtId) {
        // First, visit the statement like it would normally be done.
        visit::walk_stmt(self, stmt);

        // Then, insert the application generator set into the package store compute properties data structure.
        let package_id = self.get_current_package();
        let application_generator_set = self.get_current_application_generator_set().clone();
        self.package_store_compute_properties
            .insert_stmt((package_id, stmt).into(), application_generator_set);
    }
}
