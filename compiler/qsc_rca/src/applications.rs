// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    common::{
        aggregate_compute_kind, aggregate_value_kind, initalize_locals_map, InputParam,
        InputParamIndex, Local, LocalKind, LocalsLookup,
    },
    scaffolding::PackageScaffolding,
    ApplicationsTable, ComputeKind, QuantumProperties, RuntimeFeatureFlags, ValueKind,
};
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{BlockId, ExprId, NodeId, Pat, PatId, PatKind, SpecDecl, StmtId};
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct ApplicationInstancesTable {
    pub inherent: ApplicationInstance,
    pub dynamic_params: Vec<ApplicationInstance>,
    is_settled: bool,
}

impl ApplicationInstancesTable {
    pub fn from_spec(
        spec_decl: &SpecDecl,
        input_params: &Vec<InputParam>,
        pats: &IndexMap<PatId, Pat>,
    ) -> Self {
        let spec_input = derive_spec_input(spec_decl, pats);
        let inherent = ApplicationInstance::new(input_params, None, spec_input.as_ref());
        let mut dynamic_params = Vec::<ApplicationInstance>::with_capacity(input_params.len());
        for input_param in input_params {
            let application_instance = ApplicationInstance::new(
                input_params,
                Some(input_param.index),
                spec_input.as_ref(),
            );
            dynamic_params.push(application_instance);
        }

        Self {
            inherent,
            dynamic_params,
            is_settled: false,
        }
    }

    pub fn parameterless() -> Self {
        let inherent = ApplicationInstance::new(&Vec::new(), None, None);
        Self {
            inherent,
            dynamic_params: Vec::new(),
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

        // Collect the value kind (if any) from the return expressions.
        let (inherent_value_kind, dynamic_param_applications_value_kinds) =
            self.collect_value_kinds();

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

            // Now incorporate the previously collected value kinds.
            assert!(
                applications_table.dynamic_param_applications.len()
                    == dynamic_param_applications_value_kinds.len()
            );
            if let Some(inherent_value_kind) = inherent_value_kind {
                applications_table
                    .inherent
                    .aggregate_value_kind(inherent_value_kind);
            }
            for (application, sources) in applications_table
                .dynamic_param_applications
                .iter_mut()
                .zip(dynamic_param_applications_value_kinds)
            {
                if let Some(value_kind) = sources {
                    application.aggregate_value_kind(value_kind);
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

    fn collect_value_kinds(&mut self) -> (Option<ValueKind>, Vec<Option<ValueKind>>) {
        let inherent = self.inherent.collect_value_kind();
        let mut dynamic_param_applications =
            Vec::<Option<ValueKind>>::with_capacity(self.dynamic_params.len());
        for application_instance in self.dynamic_params.iter_mut() {
            dynamic_param_applications.push(application_instance.collect_value_kind());
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

fn derive_spec_input(spec_decl: &SpecDecl, pats: &IndexMap<PatId, Pat>) -> Option<Local> {
    spec_decl.input.and_then(|pat_id| {
        let pat = pats.get(pat_id).expect("pat should exist");
        match &pat.kind {
            PatKind::Bind(ident) => Some(Local {
                node: ident.id,
                pat: pat_id,
                ty: pat.ty.clone(),
                kind: LocalKind::SpecInput,
            }),
            PatKind::Discard => None, // Nothing to bind to.
            PatKind::Tuple(_) => panic!("expected specialization input pattern"),
        }
    })
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
    fn new(
        input_params: &Vec<InputParam>,
        dynamic_param_index: Option<InputParamIndex>,
        spec_input: Option<&Local>,
    ) -> Self {
        // Initialize the locals map with the specialization input (if any).
        let mut locals_map = LocalsComputeKindMap::default();
        if let Some(spec_input_local) = spec_input {
            // Specialization inputs are currently only used for controls, whose compute properties are handled at
            // the call expression, so just use classical compute kind here for when controls are explicitly used.
            locals_map.insert(
                spec_input_local.node,
                LocalComputeKind {
                    local: spec_input_local.clone(),
                    compute_kind: ComputeKind::Classical,
                },
            );
        }
        let mut unprocessed_locals_map = initalize_locals_map(input_params);
        for (node_id, local) in unprocessed_locals_map.drain() {
            let LocalKind::InputParam(input_param_index) = local.kind else {
                panic!("only input parameters are expected");
            };

            // If a dynamic parameter index is provided, set the local compute kind as dynamic.
            let compute_kind = if let Some(dynamic_param_index) = dynamic_param_index {
                if input_param_index == dynamic_param_index {
                    ComputeKind::Quantum(QuantumProperties {
                        runtime_features: RuntimeFeatureFlags::empty(),
                        value_kind: ValueKind::Dynamic,
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

    fn collect_value_kind(&mut self) -> Option<ValueKind> {
        // Cannot do this until the application instance has been settled, but not yet flushed.
        assert!(self.is_settled);
        assert!(!self.was_flushed);

        // Go through each return expression aggregating their value kind (if any).
        let mut value_kinds = Vec::<ValueKind>::new();
        for expr_id in self.return_expressions.drain(..) {
            let expr_compute_kind = self
                .exprs
                .get(&expr_id)
                .expect("expression compute kind should exist");
            if let ComputeKind::Quantum(quantum_properties) = expr_compute_kind {
                value_kinds.push(quantum_properties.value_kind);
            }
        }

        // Only return something if at least one return expression was quantum.
        if value_kinds.is_empty() {
            None
        } else {
            let value_kind = value_kinds.iter().fold(
                ValueKind::Static,
                |aggregated_value_kind, current_value_kind| {
                    aggregate_value_kind(aggregated_value_kind, current_value_kind)
                },
            );
            Some(value_kind)
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
    pub fn aggregate_compute_kind(&mut self, node_id: NodeId, delta: &ComputeKind) {
        let local_compute_kind = self
            .0
            .get_mut(&node_id)
            .expect("compute kind for local should exist");
        local_compute_kind.compute_kind =
            aggregate_compute_kind(local_compute_kind.compute_kind.clone(), delta);
    }

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
