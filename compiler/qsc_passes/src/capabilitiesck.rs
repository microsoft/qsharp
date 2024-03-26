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

use itertools::Itertools;
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_fir::{
    fir::{
        Block, BlockId, CallableImpl, Expr, ExprId, ExprKind, Global, Ident, Item, ItemKind,
        LocalItemId, LocalVarId, Package, PackageLookup, Pat, PatId, PatKind, Res, SpecDecl,
        SpecImpl, Stmt, StmtId, StmtKind,
    },
    ty::FunctorSetValue,
    visit::Visitor,
};
use qsc_frontend::compile::RuntimeCapabilityFlags;
use qsc_rca::{ComputeKind, ItemComputeProperties, PackageComputeProperties, RuntimeFeatureFlags};
use rustc_hash::FxHashMap;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("cannot use a dynamic boolean value")]
    #[diagnostic(help(
        "using a bool value that depends on a measurement result is not supported by the current target"
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

    #[error("cannot use a dynamic qubit")]
    #[diagnostic(help(
        "using a dynamic qubit, a qubit whose allocation depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicQubit"))]
    UseOfDynamicQubit(#[label] Span),

    #[error("cannot use a dynamic big integer value")]
    #[diagnostic(help(
        "using a dynamic big integer value, a big integer value that depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicBigInt"))]
    UseOfDynamicBigInt(#[label] Span),

    #[error("cannot use a dynamic string value")]
    #[diagnostic(help(
        "using a dynamic string value, a string value that depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicString"))]
    UseOfDynamicString(#[label] Span),

    #[error("cannot use a dynamically-sized array")]
    #[diagnostic(help(
        "using a dynamically-sized array, an array whose size depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicallySizedArray"))]
    UseOfDynamicallySizedArray(#[label] Span),

    #[error("cannot use a dynamic user-defined type")]
    #[diagnostic(help(
        "using a dynamic user-defined type, a user-defined type in which one or more of its members depend on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicUdt"))]
    UseOfDynamicUdt(#[label] Span),

    #[error("cannot use a dynamic function")]
    #[diagnostic(help(
        "using a dynamically resolved function, a function whose resolution depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicArrowFunction"))]
    UseOfDynamicArrowFunction(#[label] Span),

    #[error("cannot use a dynamic operation")]
    #[diagnostic(help(
        "using a dynamically resolved operation, an operation whose resolution depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicArrowOperation"))]
    UseOfDynamicArrowOperation(#[label] Span),

    #[error("cannot call a cyclic function with a dynamic value as argument")]
    #[diagnostic(help(
        "calling a cyclic function with a dynamic value as argument, an argument value that depends on a measurement result, is not supported by the target"
    ))]
    #[diagnostic(code("Qsc.CapabilitiesCk.CallToCyclicFunctionWithDynamicArg"))]
    CallToCyclicFunctionWithDynamicArg(#[label] Span),

    #[error("cannot define a cyclic operation specialization")]
    #[diagnostic(help("operation specializations that contain call cycles are not supported by the current target"))]
    #[diagnostic(code("Qsc.CapabilitiesCk.CyclicOperationSpec"))]
    CyclicOperationSpec(#[label] Span),

    #[error("cannot call a cyclic operation")]
    #[diagnostic(help("calling a cyclic operation is not supported by the target"))]
    #[diagnostic(code("Qsc.CapabilitiesCk.CallToCyclicOperation"))]
    CallToCyclicOperation(#[label] Span),

    #[error("cannot call a function or operation whose resolution is dynamic")]
    #[diagnostic(help("calling a function or operation whose resolution is dynamic, a resolution that depends on a measurement result, is not supported by the target"))]
    #[diagnostic(code("Qsc.CapabilitiesCk.CallToDynamicCallee"))]
    CallToDynamicCallee(#[label] Span),

    #[error("cannot call a function or operation that can only be resolved at runtime")]
    #[diagnostic(help("calling a function or operation that can only be resolved at runtime is not supported by the target"))]
    #[diagnostic(code("Qsc.CapabilitiesCk.CallToUnresolvedCallee"))]
    CallToUnresolvedCallee(#[label] Span),

    #[error("cannot perform a measurement within a dynamic scope")]
    #[diagnostic(help("performing a measurement within dynamic scope, a scope that depends on a measurement result, is not supported by the target"))]
    #[diagnostic(code("Qsc.CapabilitiesCk.MeasurementWithinDynamicScope"))]
    MeasurementWithinDynamicScope(#[label] Span),

    #[error("cannot access an array using a dynamic index")]
    #[diagnostic(help("accessing an array using a dynamic index, an index that depends on a measurement result, is not supported by the target"))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfDynamicIndex"))]
    UseOfDynamicIndex(#[label] Span),

    #[error("cannot use a return within a dynamic scope")]
    #[diagnostic(help("using a return within a dynamic scope, a scope that depends on a measurement result, is not supported by the target"))]
    #[diagnostic(code("Qsc.CapabilitiesCk.ReturnWithinDynamicScope"))]
    ReturnWithinDynamicScope(#[label] Span),

    #[error("cannot have a loop with a dynamic condition")]
    #[diagnostic(help("using a loop with a dynamic condition, a condition that depends on a measurement result, is not supported by the target"))]
    #[diagnostic(code("Qsc.CapabilitiesCk.LoopWithDynamicCondition"))]
    LoopWithDynamicCondition(#[label] Span),

    #[error("cannot use a closure")]
    #[diagnostic(help("closures are not supported by the target"))]
    #[diagnostic(code("Qsc.CapabilitiesCk.UseOfClosure"))]
    UseOfClosure(#[label] Span),
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
        missing_features_map: FxHashMap::<Span, RuntimeFeatureFlags>::default(),
    };

    checker.check_all()
}

struct Checker<'a> {
    package: &'a Package,
    compute_properties: &'a PackageComputeProperties,
    target_capabilities: RuntimeCapabilityFlags,
    current_callable: Option<LocalItemId>,
    missing_features_map: FxHashMap<Span, RuntimeFeatureFlags>,
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

    fn visit_expr(&mut self, expr_id: ExprId) {
        let expr = self.get_expr(expr_id);

        // We do not want to generate errors for auto-generated expressions.
        if self.is_expr_auto_generated(expr) {
            return;
        }

        match &expr.kind {
            ExprKind::Block(block_id) => self.visit_block(*block_id),
            ExprKind::If(condition_expr_id, body_block_id, otherwise_block_id) => self
                .check_expr_if(
                    expr_id,
                    *condition_expr_id,
                    *body_block_id,
                    *otherwise_block_id,
                ),
            ExprKind::While(condition_expr_id, body_block_id) => {
                self.check_expr_while(expr_id, *condition_expr_id, *body_block_id);
            }
            _ => self.check_expr(expr_id),
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
        let stmt = self.get_stmt(stmt_id);
        match &stmt.kind {
            StmtKind::Item(_) => {}
            StmtKind::Expr(expr_id) | StmtKind::Semi(expr_id) | StmtKind::Local(_, _, expr_id) => {
                self.visit_expr(*expr_id);
            }
        }
    }
}

impl<'a> Checker<'a> {
    pub fn check_all(mut self) -> Vec<Error> {
        self.visit_package(self.package);
        self.generate_errors()
    }

    fn check_expr(&mut self, expr_id: ExprId) {
        let compute_kind = self.compute_properties.get_expr(expr_id).inherent;
        let ComputeKind::Quantum(quantum_properties) = compute_kind else {
            return;
        };

        let missing_features = get_missing_runtime_features(
            quantum_properties.runtime_features,
            self.target_capabilities,
        );
        let expr = self.get_expr(expr_id);
        if !missing_features.is_empty() {
            self.missing_features_map
                .entry(expr.span)
                .and_modify(|f| *f |= missing_features)
                .or_insert(missing_features);
        }
    }

    fn check_expr_if(
        &mut self,
        expr_id: ExprId,
        condition_expr_id: ExprId,
        body_expr_id: ExprId,
        otherwise_expr_id: Option<ExprId>,
    ) {
        // Check each one of the sub-expressions individually to provide more granularity.
        let pre_spans_with_missing_features_count = self.missing_features_map.len();
        self.visit_expr(condition_expr_id);
        self.visit_expr(body_expr_id);
        otherwise_expr_id.iter().for_each(|e| self.visit_expr(*e));
        let post_spans_with_missing_features_count = self.missing_features_map.len();

        // If no errors were added because of the individual expressions, check this expression as a whole.
        let errors_delta =
            post_spans_with_missing_features_count - pre_spans_with_missing_features_count;
        if errors_delta == 0 {
            self.check_expr(expr_id);
        }
    }

    fn check_expr_while(
        &mut self,
        expr_id: ExprId,
        condition_expr_id: ExprId,
        body_block_id: BlockId,
    ) {
        // Check each one of the sub-elements individually to provide more granularity.
        let pre_spans_with_missing_features_count = self.missing_features_map.len();
        self.visit_expr(condition_expr_id);
        self.visit_block(body_block_id);
        let post_spans_with_missing_features_count = self.missing_features_map.len();

        // If no errors were added because of the individual elements, check this expression as a whole.
        let errors_delta =
            post_spans_with_missing_features_count - pre_spans_with_missing_features_count;
        if errors_delta == 0 {
            self.check_expr(expr_id);
        }
    }

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

                self.missing_features_map
                    .entry(callable_decl.name.span)
                    .and_modify(|f| *f |= missing_spec_level_runtime_features)
                    .or_insert(missing_spec_level_runtime_features);
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

    fn find_local_var_ident(&self, local_var_id: LocalVarId) -> Option<&Ident> {
        self.package
            .pats
            .iter()
            .map(|(_, pat)| pat)
            .find_map(|pat| {
                if let PatKind::Bind(ident) = &pat.kind {
                    if ident.id == local_var_id {
                        Some(ident)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
    }

    fn generate_errors(&mut self) -> Vec<Error> {
        let mut errors = Vec::new();

        for (span, missing_features) in self.missing_features_map.drain().sorted() {
            let mut span_errors = generate_errors_from_runtime_features(missing_features, span);
            errors.append(&mut span_errors);
        }
        errors
    }

    fn get_current_callable(&self) -> LocalItemId {
        self.current_callable.expect("current callable is not set")
    }

    fn is_expr_auto_generated(&self, expr: &Expr) -> bool {
        if expr.span.hi == 0 && expr.span.lo == 0 {
            return true;
        }

        if let ExprKind::Var(Res::Local(local_var_id), _) = expr.kind {
            let maybe_ident = self.find_local_var_ident(local_var_id);
            if let Some(ident) = maybe_ident {
                return ident.name.starts_with('@');
            }
        }

        false
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
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicQubit) {
        errors.push(Error::UseOfDynamicQubit(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicBigInt) {
        errors.push(Error::UseOfDynamicBigInt(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicString) {
        errors.push(Error::UseOfDynamicString(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicallySizedArray) {
        errors.push(Error::UseOfDynamicallySizedArray(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicUdt) {
        errors.push(Error::UseOfDynamicUdt(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicArrowFunction) {
        errors.push(Error::UseOfDynamicArrowFunction(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicArrowOperation) {
        errors.push(Error::UseOfDynamicArrowOperation(span));
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
    if runtime_features.contains(RuntimeFeatureFlags::CallToDynamicCallee) {
        errors.push(Error::CallToDynamicCallee(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::CallToUnresolvedCallee) {
        errors.push(Error::CallToUnresolvedCallee(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::MeasurementWithinDynamicScope) {
        errors.push(Error::MeasurementWithinDynamicScope(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfDynamicIndex) {
        errors.push(Error::UseOfDynamicIndex(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::ReturnWithinDynamicScope) {
        errors.push(Error::ReturnWithinDynamicScope(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::LoopWithDynamicCondition) {
        errors.push(Error::LoopWithDynamicCondition(span));
    }
    if runtime_features.contains(RuntimeFeatureFlags::UseOfClosure) {
        errors.push(Error::UseOfClosure(span));
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
