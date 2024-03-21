// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod base;

#[cfg(test)]
mod adaptive;

#[cfg(test)]
mod adaptive_plus_integers;

#[cfg(test)]
pub mod common;

use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_fir::{
    fir::{
        Block, BlockId, CallableImpl, Expr, ExprId, Global, Item, ItemKind, LocalItemId, Package,
        PackageLookup, Pat, PatId, SpecDecl, SpecImpl, Stmt, StmtId,
    },
    ty::FunctorSetValue,
    visit::Visitor,
};
use qsc_frontend::compile::RuntimeCapabilityFlags;
use qsc_rca::{ComputeKind, ItemComputeProperties, PackageComputeProperties, RuntimeFeatureFlags};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("cannot use a dynamic boolean value")]
    #[diagnostic(help(
        "using a dynamic boolean value, a boolean value that depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicBool"))]
    UseOfDynamicBool(#[label] Span),

    #[error("cannot use a dynamic integer value")]
    #[diagnostic(help(
        "using a dynamic integer value, an integer value that depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicInt"))]
    UseOfDynamicInt(#[label] Span),

    #[error("cannot use a dynamic Pauli value")]
    #[diagnostic(help(
        "using a dynamic Pauli value, a Pauli value that depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicPauli"))]
    UseOfDynamicPauli(#[label] Span),

    #[error("cannot use a dynamic Range value")]
    #[diagnostic(help(
        "using a dynamic Range value, a Range value that depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicRange"))]
    UseOfDynamicRange(#[label] Span),

    #[error("cannot use a dynamic double value")]
    #[diagnostic(help(
        "using a dynamic double value, a double value that depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicDouble"))]
    UseOfDynamicDouble(#[label] Span),

    #[error("cannot use a dynamically-sized array")]
    #[diagnostic(help(
        "using a dynamically-sized array, an array whose size depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicallySizedArray"))]
    UseOfDynamicallySizedArray(#[label] Span),

    #[error("cannot call a cyclic function with a dynamic value as argument")]
    #[diagnostic(help(
        "calling a cyclic function with a dynamic value as argument, an argument value that depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.CallToCyclicFunctionWithDynamicArg"))]
    CallToCyclicFunctionWithDynamicArg(#[label] Span),

    #[error("cannot define a cyclic operation specialization")]
    #[diagnostic(help("cyclic operation specializations, specializations that contain cycles, are not supported by the target"))]
    #[diagnostic(code("Qsc.CapabilitiesCk.CyclicOperationSpec"))]
    CyclicOperationSpec(#[label] Span),

    #[error("cannot call a cyclic operation")]
    #[diagnostic(help("calling a cyclic operation is not supported by the target"))]
    #[diagnostic(code("Qsc.CapabilitiesCk.CallToCyclicOperation"))]
    CallToCyclicOperation(#[label] Span),
}

#[must_use]
pub fn check_supported_capabilities(
    package: &Package,
    compute_properties: &PackageComputeProperties,
    capabilities: RuntimeCapabilityFlags,
) -> Vec<Error> {
    let checker = Checker {
        package,
        compute_properties,
        target_capabilities: capabilities,
        current_callable: None,
        errors: Vec::new(),
    };

    checker.run()
}

struct Checker<'a> {
    package: &'a Package,
    compute_properties: &'a PackageComputeProperties,
    target_capabilities: RuntimeCapabilityFlags,
    current_callable: Option<LocalItemId>,
    errors: Vec<Error>,
}

impl<'a> Visitor<'a> for Checker<'a> {
    fn get_block(&self, id: BlockId) -> &'a Block {
        self.package.get_block(id)
    }

    fn get_expr(&self, id: ExprId) -> &'a Expr {
        self.package.get_expr(id)
    }

    fn get_pat(&self, id: PatId) -> &'a Pat {
        self.package.get_pat(id)
    }

    fn get_stmt(&self, id: StmtId) -> &'a Stmt {
        self.package.get_stmt(id)
    }

    fn visit_callable_impl(&mut self, callable_impl: &'a CallableImpl) {
        match callable_impl {
            CallableImpl::Intrinsic => self.check_spec_decl(FunctorSetValue::Empty, None),
            CallableImpl::Spec(spec_impl) => {
                self.check_spec_decl(FunctorSetValue::Empty, Some(&spec_impl.body));
                spec_impl.adj.iter().for_each(|spec_decl| {
                    self.check_spec_decl(FunctorSetValue::Adj, Some(spec_decl));
                });
                spec_impl.ctl.iter().for_each(|spec_decl| {
                    self.check_spec_decl(FunctorSetValue::Ctl, Some(spec_decl));
                });
                spec_impl.ctl_adj.iter().for_each(|spec_decl| {
                    self.check_spec_decl(FunctorSetValue::CtlAdj, Some(spec_decl));
                });
            }
        };
    }

    fn visit_item(&mut self, item: &'a Item) {
        // We only care about callables.
        if let ItemKind::Callable(callable_decl) = &item.kind {
            self.set_current_callable(item.id);
            self.visit_callable_decl(callable_decl);
            let callable_id = self.clear_current_callable();
            assert!(item.id == callable_id);
        }
    }

    fn visit_spec_impl(&mut self, _: &'a SpecImpl) {
        panic!("visiting a specialization implementation directly should not happen");
    }

    fn visit_stmt(&mut self, stmt_id: StmtId) {
        let compute_kind = self.compute_properties.get_stmt(stmt_id).inherent;
        let ComputeKind::Quantum(quantum_properties) = compute_kind else {
            return;
        };

        let missing_features = get_missing_runtime_features(
            quantum_properties.runtime_features,
            self.target_capabilities,
        );
        let stmt = self.get_stmt(stmt_id);
        let mut stmt_errors = generate_errors_from_runtime_features(missing_features, stmt.span);
        self.errors.append(&mut stmt_errors);
    }
}

impl<'a> Checker<'a> {
    fn check_spec_decl(
        &mut self,
        functor_set_value: FunctorSetValue,
        spec_decl: Option<&'a SpecDecl>,
    ) {
        let current_callable_id = self.get_current_callable();
        let ItemComputeProperties::Callable(callable_compute_properties) =
            self.compute_properties.get_item(current_callable_id)
        else {
            panic!("expected callable variant of item compute properties");
        };

        let spec_compute_properties = match functor_set_value {
            FunctorSetValue::Empty => &callable_compute_properties.body,
            FunctorSetValue::Adj => callable_compute_properties
                .adj
                .as_ref()
                .expect("adj specialization is none"),
            FunctorSetValue::Ctl => callable_compute_properties
                .ctl
                .as_ref()
                .expect("ctl specialization is none"),
            FunctorSetValue::CtlAdj => callable_compute_properties
                .ctl_adj
                .as_ref()
                .expect("ctl_adj specialization is none"),
        };

        if let ComputeKind::Quantum(quantum_properties) = spec_compute_properties.inherent {
            let missing_features = get_missing_runtime_features(
                quantum_properties.runtime_features,
                self.target_capabilities,
            );
            let missing_spec_level_runtime_features =
                get_spec_level_runtime_features(missing_features);

            // If there are any missing specialization-level runtime features, runtime features that affect the whole
            // specialization, just generate errors for the missing specialization-level runtime features and do not
            // generate statement-level or expression-level errors for these specializations.
            if !missing_spec_level_runtime_features.is_empty() {
                let current_callable = self
                    .package
                    .get_global(current_callable_id)
                    .expect("callable not present in package");
                let Global::Callable(callable_decl) = current_callable else {
                    panic!("");
                };
                let mut spec_level_errors = generate_errors_from_runtime_features(
                    missing_spec_level_runtime_features,
                    callable_decl.name.span,
                );
                self.errors.append(&mut spec_level_errors);
                return;
            }
        }

        // Visit the specialization block.
        if let Some(spec_decl) = spec_decl {
            self.visit_block(spec_decl.block);
        }
    }

    fn clear_current_callable(&mut self) -> LocalItemId {
        self.current_callable
            .take()
            .expect("current callable is not set")
    }

    fn get_current_callable(&self) -> LocalItemId {
        self.current_callable.expect("current callable is not set")
    }

    fn run(mut self) -> Vec<Error> {
        self.visit_package(self.package);
        self.errors
    }

    fn set_current_callable(&mut self, id: LocalItemId) {
        assert!(self.current_callable.is_none());
        self.current_callable = Some(id);
    }
}

fn generate_errors_from_runtime_features(
    runtime_features: RuntimeFeatureFlags,
    span: Span,
) -> Vec<Error> {
    let mut errors = Vec::<Error>::new();
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicBool) {
        errors.push(Error::UseOfDynamicBool(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicInt) {
        errors.push(Error::UseOfDynamicInt(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicPauli) {
        errors.push(Error::UseOfDynamicPauli(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicRange) {
        errors.push(Error::UseOfDynamicRange(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicDouble) {
        errors.push(Error::UseOfDynamicDouble(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicallySizedArray) {
        errors.push(Error::UseOfDynamicallySizedArray(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::CallToCyclicFunctionWithDynamicArg) {
        errors.push(Error::CallToCyclicFunctionWithDynamicArg(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::CyclicOperationSpec) {
        errors.push(Error::CyclicOperationSpec(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::CallToCyclicOperation) {
        errors.push(Error::CallToCyclicOperation(span));
    }
    errors
}

fn get_missing_runtime_features(
    runtime_features: RuntimeFeatureFlags,
    target_capabilities: RuntimeCapabilityFlags,
) -> RuntimeFeatureFlags {
    let missing_capabilities = !target_capabilities & runtime_features.runtime_capabilities();
    runtime_features.contributing_features(missing_capabilities)
}

fn get_spec_level_runtime_features(runtime_features: RuntimeFeatureFlags) -> RuntimeFeatureFlags {
    const SPEC_LEVEL_RUNTIME_FEATURES: RuntimeFeatureFlags =
        RuntimeFeatureFlags::CyclicOperationSpec;
    runtime_features & SPEC_LEVEL_RUNTIME_FEATURES
}
