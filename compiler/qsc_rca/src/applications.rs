// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    common::{initalize_locals_map, InputParam, InputParamIndex, Local, LocalKind},
    scaffolding::PackageScaffolding,
    ApplicationsTable, ComputeProperties, DynamismSource, RuntimeFeatureFlags,
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
        main_block_id: Option<BlockId>,
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

        // Initialize the applications table and aggregate the return expressions to it.
        let mut provisional_applications_table = ApplicationsTable::new(self.dynamic_params.len());
        self.aggregate_return_expressions(&mut provisional_applications_table);

        // Flush the compute properties to the package scaffolding
        self.flush_compute_properties(package_scaffolding);

        // Get the applications table of the main block and aggregate its runtime features.
        if let Some(main_block_id) = main_block_id {
            let main_block_applications_table = package_scaffolding
                .blocks
                .get(main_block_id)
                .expect("block applications table should exist");
            provisional_applications_table
                .aggregate_runtime_features(main_block_applications_table);
        }

        // Mark the struct as settled and return the applications table that represents it.
        self.is_settled = true;

        // Only return an applications table if a main block was provided.
        main_block_id.map(|_| provisional_applications_table)
    }

    fn aggregate_return_expressions(&mut self, applications_table: &mut ApplicationsTable) {
        assert!(self.dynamic_params.len() == applications_table.dynamic_params_properties.len());
        let inherent_dynamism_sources = self.inherent.aggregate_return_expressions();
        applications_table
            .inherent_properties
            .dynamism_sources
            .extend(inherent_dynamism_sources);
        for (param_compute_properties, application_instance) in applications_table
            .dynamic_params_properties
            .iter_mut()
            .zip(self.dynamic_params.iter_mut())
        {
            let dynamism_sources = application_instance.aggregate_return_expressions();
            param_compute_properties
                .dynamism_sources
                .extend(dynamism_sources);
        }
    }

    fn clear_locals(&mut self) {
        self.inherent.clear_locals();
        self.dynamic_params
            .iter_mut()
            .for_each(|application_instance| application_instance.clear_locals());
    }

    fn flush_compute_properties(&mut self, package_scaffolding: &mut PackageScaffolding) {
        let input_params_count = self.dynamic_params.len();

        // Flush blocks.
        for (block_id, inherent_properties) in self.inherent.blocks.drain() {
            let mut dynamic_params_properties =
                Vec::<ComputeProperties>::with_capacity(input_params_count);
            for application_instance in self.dynamic_params.iter_mut() {
                let block_compute_properties = application_instance
                    .blocks
                    .remove(&block_id)
                    .expect("block should exist in application instance");
                dynamic_params_properties.push(block_compute_properties);
            }
            let block_applications_table = ApplicationsTable {
                inherent_properties,
                dynamic_params_properties,
            };
            package_scaffolding
                .blocks
                .insert(block_id, block_applications_table);
        }

        // Flush statements.
        for (stmt_id, inherent_properties) in self.inherent.stmts.drain() {
            let mut dynamic_params_properties =
                Vec::<ComputeProperties>::with_capacity(input_params_count);
            for application_instance in self.dynamic_params.iter_mut() {
                let stmt_compute_properties = application_instance
                    .stmts
                    .remove(&stmt_id)
                    .expect("statement should exist in application instance");
                dynamic_params_properties.push(stmt_compute_properties);
            }
            let stmt_applications_table = ApplicationsTable {
                inherent_properties,
                dynamic_params_properties,
            };
            package_scaffolding
                .stmts
                .insert(stmt_id, stmt_applications_table);
        }

        // Flush expressions.
        for (expr_id, inherent_properties) in self.inherent.exprs.drain() {
            let mut dynamic_params_properties =
                Vec::<ComputeProperties>::with_capacity(input_params_count);
            for application_instance in self.dynamic_params.iter_mut() {
                let expr_compute_properties = application_instance
                    .exprs
                    .remove(&expr_id)
                    .expect("statement should exist in application instance");
                dynamic_params_properties.push(expr_compute_properties);
            }
            let expr_applications_table = ApplicationsTable {
                inherent_properties,
                dynamic_params_properties,
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
    /// A map of locals with their associated compute properties.
    pub locals_map: FxHashMap<NodeId, LocalComputeProperties>,
    /// The currently active dynamic scopes in the application instance.
    pub active_dynamic_scopes: Vec<ExprId>,
    /// The return expressions througout the application instance.
    pub return_expressions: Vec<ExprId>,
    /// The compute properties of the blocks related to the application instance.
    pub blocks: FxHashMap<BlockId, ComputeProperties>,
    /// The compute properties of the statements related to the application instance.
    pub stmts: FxHashMap<StmtId, ComputeProperties>,
    /// The compute properties of the expressions related to the application instance.
    pub exprs: FxHashMap<ExprId, ComputeProperties>,
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
        let mut locals_map = FxHashMap::default();
        for (node_id, local) in unprocessed_locals_map.drain() {
            let LocalKind::InputParam(input_param_index) = local.kind else {
                panic!("only input parameters are expected");
            };

            // If a dynamic parameter index is provided, set the local compute properties as dynamic.
            let dynamism_sources = if let Some(dynamic_param_index) = dynamic_param_index {
                if input_param_index == dynamic_param_index {
                    FxHashSet::from_iter(vec![DynamismSource::Assumed])
                } else {
                    FxHashSet::default()
                }
            } else {
                FxHashSet::default()
            };
            let local_compute_properties = LocalComputeProperties {
                local,
                compute_properties: ComputeProperties {
                    runtime_features: RuntimeFeatureFlags::empty(),
                    dynamism_sources,
                },
            };
            locals_map.insert(node_id, local_compute_properties);
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

    fn aggregate_return_expressions(&mut self) -> FxHashSet<DynamismSource> {
        // Cannot aggregate return expressions until the application instance has been settled, but not yet flushed.
        assert!(self.is_settled);
        assert!(!self.was_flushed);
        let mut dynamism_sources = FxHashSet::default();
        for expr_id in self.return_expressions.drain(..) {
            let expr_compute_properties = self
                .exprs
                .get(&expr_id)
                .expect("expression compute properties should exist");
            if !expr_compute_properties.dynamism_sources.is_empty() {
                dynamism_sources.insert(DynamismSource::Expr(expr_id));
            }
        }
        dynamism_sources
    }

    fn clear_locals(&mut self) {
        // Cannot clear locals until the application instance has been settled.
        assert!(self.is_settled);
        self.locals_map.clear();
    }

    fn mark_flushed(&mut self) {
        // Cannot mark as flushed until the application instance has been settled, no return expressions remain and all
        // compute properties maps are empty.
        assert!(self.is_settled);
        assert!(self.return_expressions.is_empty());
        assert!(self.blocks.is_empty());
        assert!(self.stmts.is_empty());
        assert!(self.exprs.is_empty());
        self.was_flushed = true;
    }
}

#[derive(Debug)]
pub struct LocalComputeProperties {
    pub local: Local,
    pub compute_properties: ComputeProperties,
}
