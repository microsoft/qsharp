// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    common::{initalize_locals_map, InputParam, InputParamIndex, Local, LocalKind, LocalsLookup},
    scaffolding::PackageScaffolding,
    ApplicationsTable, ComputeKind, DynamismSource, QuantumProperties, RuntimeFeatureFlags,
    ValueKind,
};
use qsc_fir::fir::{BlockId, ExprId, NodeId, StmtId};
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug)]
pub struct SpecApplicationInstances {
    pub inherent: ApplicationInstance,
    pub dynamic_params: Vec<ApplicationInstance>,
    is_settled: bool,
}

impl SpecApplicationInstances {
    pub fn new(input_params: &Vec<InputParam>) -> Self {
        let inherent = ApplicationInstance::new(input_params, None);
        let mut dynamic_params = Vec::<ApplicationInstance>::with_capacity(input_params.len());
        for input_param in input_params {
            let application_instance =
                ApplicationInstance::new(input_params, Some(input_param.index));
            dynamic_params.push(application_instance);
        }

        Self {
            inherent,
            dynamic_params,
            is_settled: false,
        }
    }

    pub fn close(
        &mut self,
        package_scaffolding: &mut PackageScaffolding,
        main_block: Option<BlockId>,
    ) -> Option<ApplicationsTable> {
        // We can close only if this structure is not yet settled and if all the internal application instances are
        // already settled.
        assert!(!self.is_settled);
        assert!(self.inherent.is_settled);
        self.dynamic_params
            .iter()
            .for_each(|application_instance| assert!(application_instance.is_settled));

        // Clear the locals since they are no longer needed.
        self.clear_locals();

        // Collect the sources of dynamism (if any) from the return statements.
        let (inherent_sources, dynamic_param_application_sources) = self.collect_dynamism_sources();

        // Flush the compute properties to the package scaffolding
        self.flush_compute_properties(package_scaffolding);

        // If a main block was provided, create an applications table that represents the specialization based on the
        // applications table of the main block.
        let close_output = main_block.map(|main_block_id| {
            let mut applications_table = package_scaffolding
                .blocks
                .get(main_block_id)
                .expect("block applications table should exist")
                .clone();

            // Now aggregate the previously collected sources of dynamism.
            assert!(
                applications_table.dynamic_param_applications.len()
                    == dynamic_param_application_sources.len()
            );
            if let Some(inherent_sources) = inherent_sources {
                applications_table
                    .inherent
                    .add_dynamism_sources(inherent_sources);
            }
            for (application, sources) in applications_table
                .dynamic_param_applications
                .iter_mut()
                .zip(dynamic_param_application_sources)
            {
                if let Some(sources) = sources {
                    application.add_dynamism_sources(sources);
                }
            }

            // Return the applications table with the updated dynamism sources.
            applications_table
        });

        // Mark the struct as settled and return the applications table that represents it.
        self.is_settled = true;
        close_output
    }

    fn clear_locals(&mut self) {
        self.inherent.clear_locals();
        self.dynamic_params
            .iter_mut()
            .for_each(|application_instance| application_instance.clear_locals());
    }

    fn collect_dynamism_sources(
        &mut self,
    ) -> (
        Option<FxHashSet<DynamismSource>>,
        Vec<Option<FxHashSet<DynamismSource>>>,
    ) {
        let inherent = self.inherent.collect_dynamism_sources();
        let mut dynamic_param_applications =
            Vec::<Option<FxHashSet<DynamismSource>>>::with_capacity(self.dynamic_params.len());
        for application_instance in self.dynamic_params.iter_mut() {
            dynamic_param_applications.push(application_instance.collect_dynamism_sources());
        }

        (inherent, dynamic_param_applications)
    }

    fn flush_compute_properties(&mut self, package_scaffolding: &mut PackageScaffolding) {
        let input_params_count = self.dynamic_params.len();

        // Flush blocks.
        for (block_id, inherent_compute_kind) in self.inherent.blocks.drain() {
            let mut dynamic_param_applications =
                Vec::<ComputeKind>::with_capacity(input_params_count);
            for application_instance in self.dynamic_params.iter_mut() {
                let block_compute_kind = application_instance
                    .blocks
                    .remove(&block_id)
                    .expect("block should exist in application instance");
                dynamic_param_applications.push(block_compute_kind);
            }
            let block_applications_table = ApplicationsTable {
                inherent: inherent_compute_kind,
                dynamic_param_applications,
            };
            package_scaffolding
                .blocks
                .insert(block_id, block_applications_table);
        }

        // Flush statements.
        for (stmt_id, inherent_compute_kind) in self.inherent.stmts.drain() {
            let mut dynamic_param_applications =
                Vec::<ComputeKind>::with_capacity(input_params_count);
            for application_instance in self.dynamic_params.iter_mut() {
                let stmt_compute_kind = application_instance
                    .stmts
                    .remove(&stmt_id)
                    .expect("statement should exist in application instance");
                dynamic_param_applications.push(stmt_compute_kind);
            }
            let stmt_applications_table = ApplicationsTable {
                inherent: inherent_compute_kind,
                dynamic_param_applications,
            };
            package_scaffolding
                .stmts
                .insert(stmt_id, stmt_applications_table);
        }

        // Flush expressions.
        for (expr_id, inherent_compute_kind) in self.inherent.exprs.drain() {
            let mut dynamic_param_applications =
                Vec::<ComputeKind>::with_capacity(input_params_count);
            for application_instance in self.dynamic_params.iter_mut() {
                let expr_compute_kind = application_instance
                    .exprs
                    .remove(&expr_id)
                    .expect("statement should exist in application instance");
                dynamic_param_applications.push(expr_compute_kind);
            }
            let expr_applications_table = ApplicationsTable {
                inherent: inherent_compute_kind,
                dynamic_param_applications,
            };
            package_scaffolding
                .exprs
                .insert(expr_id, expr_applications_table);
        }

        // Mark individual application instances as flushed.
        self.inherent.mark_flushed();
        self.dynamic_params
            .iter_mut()
            .for_each(|application_instance| application_instance.mark_flushed());
    }
}

/// An instance of a callable application.
#[derive(Debug, Default)]
pub struct ApplicationInstance {
    /// A map of locals with their associated compute kind.
    pub locals_map: LocalsComputeKindMap,
    /// The currently active dynamic scopes in the application instance.
    pub active_dynamic_scopes: Vec<ExprId>,
    /// The return expressions througout the application instance.
    pub return_expressions: Vec<ExprId>,
    /// The compute kind of the blocks related to the application instance.
    pub blocks: FxHashMap<BlockId, ComputeKind>,
    /// The compute kind of the statements related to the application instance.
    pub stmts: FxHashMap<StmtId, ComputeKind>,
    /// The compute kind of the expressions related to the application instance.
    pub exprs: FxHashMap<ExprId, ComputeKind>,
    /// Whether the application instance analysis has been completed.
    /// This is used to verify that its contents are not used in a partial state.
    is_settled: bool,
    /// Whether the application instance's compute properties has been flushed.
    /// This is used to verify that its contents are not used in a partial state.
    was_flushed: bool,
}

impl ApplicationInstance {
    fn new(input_params: &Vec<InputParam>, dynamic_param_index: Option<InputParamIndex>) -> Self {
        let mut unprocessed_locals_map = initalize_locals_map(input_params);
        let mut locals_map = LocalsComputeKindMap::default();
        for (node_id, local) in unprocessed_locals_map.drain() {
            let LocalKind::InputParam(input_param_index) = local.kind else {
                panic!("only input parameters are expected");
            };

            // If a dynamic parameter index is provided, set the local compute kind as dynamic.
            let compute_kind = if let Some(dynamic_param_index) = dynamic_param_index {
                if input_param_index == dynamic_param_index {
                    ComputeKind::Quantum(QuantumProperties {
                        runtime_features: RuntimeFeatureFlags::empty(),
                        value_kind: ValueKind::Dynamic(FxHashSet::from_iter(vec![
                            DynamismSource::Assumed,
                        ])),
                    })
                } else {
                    ComputeKind::Classical
                }
            } else {
                ComputeKind::Classical
            };

            locals_map.insert(
                node_id,
                LocalComputeKind {
                    local,
                    compute_kind,
                },
            );
        }
        Self {
            locals_map,
            active_dynamic_scopes: Vec::new(),
            return_expressions: Vec::new(),
            blocks: FxHashMap::default(),
            stmts: FxHashMap::default(),
            exprs: FxHashMap::default(),
            is_settled: false,
            was_flushed: false,
        }
    }

    pub fn settle(&mut self) {
        // Cannot settle an application instance while there are active dynamic scopes.
        assert!(self.active_dynamic_scopes.is_empty());
        self.is_settled = true;
    }

    fn collect_dynamism_sources(&mut self) -> Option<FxHashSet<DynamismSource>> {
        // Cannot do this until the application instance has been settled, but not yet flushed.
        assert!(self.is_settled);
        assert!(!self.was_flushed);

        // Collect the sources of dynamism from the values of all return expressions.
        let mut all_dynamism_sources = FxHashSet::default();
        for expr_id in self.return_expressions.drain(..) {
            let expr_compute_kind = self
                .exprs
                .get(&expr_id)
                .expect("expression compute kind should exist");
            if let Some(dynamism_sources) = expr_compute_kind.get_dynamism_sources() {
                all_dynamism_sources.extend(dynamism_sources.iter());
            }
        }

        // Only return something if at least one source of dynamism was collected.
        if all_dynamism_sources.is_empty() {
            None
        } else {
            Some(all_dynamism_sources)
        }
    }

    fn clear_locals(&mut self) {
        // Cannot clear locals until the application instance has been settled.
        assert!(self.is_settled);
        self.locals_map.clear();
    }

    fn mark_flushed(&mut self) {
        // Cannot mark as flushed until the application instance has been settled, no return expressions remain and all
        // compute kind maps are empty.
        assert!(self.is_settled);
        assert!(self.return_expressions.is_empty());
        assert!(self.blocks.is_empty());
        assert!(self.stmts.is_empty());
        assert!(self.exprs.is_empty());
        self.was_flushed = true;
    }
}

#[derive(Debug, Default)]
pub struct LocalsComputeKindMap(FxHashMap<NodeId, LocalComputeKind>);

impl LocalsLookup for LocalsComputeKindMap {
    fn find(&self, node_id: NodeId) -> Option<&Local> {
        self.0
            .get(&node_id)
            .map(|local_compute_kind| &local_compute_kind.local)
    }
}

impl LocalsComputeKindMap {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn find_compute_kind(&self, node_id: NodeId) -> Option<&ComputeKind> {
        self.0
            .get(&node_id)
            .map(|local_compute_kind| &local_compute_kind.compute_kind)
    }

    pub fn get_compute_kind(&self, node_id: NodeId) -> &ComputeKind {
        self.find_compute_kind(node_id)
            .expect("compute kind for local should exist")
    }

    pub fn insert(&mut self, node_id: NodeId, value: LocalComputeKind) {
        self.0.insert(node_id, value);
    }
}

#[derive(Debug)]
pub struct LocalComputeKind {
    pub local: Local,
    pub compute_kind: ComputeKind,
}
