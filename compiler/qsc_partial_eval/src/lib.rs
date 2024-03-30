// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_data_structures::functors::FunctorApp;
use qsc_eval::{
    self, backend::Backend, exec_graph_section, output::GenericReceiver, val::Value, Env, State,
    StepAction, StepResult,
};
use qsc_fir::{
    fir::{
        Block, BlockId, Expr, ExprId, LocalItemId, PackageId, PackageStore, PackageStoreLookup,
        Pat, PatId, Stmt, StmtId, StoreBlockId, StoreExprId, StorePatId, StoreStmtId,
    },
    visit::Visitor,
};
use qsc_rca::{ComputeKind, ComputePropertiesLookup, PackageStoreComputeProperties, ValueKind};
use qsc_rir::rir::{self, CallableType, Program};
use rustc_hash::FxHashMap;
use std::result::Result;

pub fn partially_evaluate(
    package_id: PackageId,
    package_store: &PackageStore,
    compute_properties: &PackageStoreComputeProperties,
) -> Result<Program, Error> {
    let partial_evaluator = PartialEvaluator::new(package_id, package_store, compute_properties);
    partial_evaluator.eval()
}

pub enum Error {
    EvaluationFailed(qsc_eval::Error),
}

struct PartialEvaluator<'a> {
    package_store: &'a PackageStore,
    compute_properties: &'a PackageStoreComputeProperties,
    _assigner: Assigner,
    backend: QubitsAndResultsAllocator,
    context: EvaluationContext,
    program: Program,
    errors: Vec<Error>,
}

impl<'a> PartialEvaluator<'a> {
    fn new(
        entry_package_id: PackageId,
        package_store: &'a PackageStore,
        compute_properties: &'a PackageStoreComputeProperties,
    ) -> Self {
        // Create the entry-point callable.
        let mut assigner = Assigner::default();
        let mut program = Program::new();
        let entry_block_id = assigner.next_block();
        let entry_block = rir::Block(Vec::new());
        program.blocks.insert(entry_block_id, entry_block);
        let entry_point_id = assigner.next_callable();
        let entry_point = rir::Callable {
            name: "main".into(),
            input_type: Vec::new(),
            output_type: None,
            body: Some(entry_block_id),
            call_type: CallableType::Regular,
        };
        program.callables.insert(entry_point_id, entry_point);
        program.entry = entry_point_id;

        // Initialize the backend and the evaluation context.
        let backend = QubitsAndResultsAllocator::default();
        let context = EvaluationContext::new(entry_package_id, package_store, entry_block_id);

        Self {
            package_store,
            compute_properties,
            _assigner: assigner,
            backend,
            context,
            program,
            errors: Vec::new(),
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn eval(mut self) -> Result<Program, Error> {
        let current_package = self.get_current_package();
        let entry_package = self.package_store.get(current_package);
        let Some(entry_expr_id) = entry_package.entry else {
            panic!("package does not have an entry expression");
        };

        // Visit the entry point expression.
        self.visit_expr(entry_expr_id);
        Ok(self.program)
    }

    fn eval_expr_classical(&mut self, expr_id: ExprId) {
        let current_package_id = self.get_current_package();
        let current_package = self.package_store.get(current_package_id);
        let store_expr_id = StoreExprId::from((current_package_id, expr_id));
        let expr = self.package_store.get_expr(store_expr_id);

        // What happens if we are executing an expression in a package that does not have an entry-point (nor an
        // entry execution graph) such as the core or standard libraries?
        let exec_graph = exec_graph_section(
            &current_package.entry_exec_graph,
            expr.exec_graph_range.clone(),
        );
        let mut out = Vec::new();
        let mut receiver = GenericReceiver::new(&mut out);
        let mut state = State::new(current_package_id, exec_graph, None);
        let eval_result = state.eval(
            self.package_store,
            &mut self.context.env,
            &mut self.backend,
            &mut receiver,
            &[],
            StepAction::Continue,
        );
        match eval_result {
            Ok(step_result) => {
                let StepResult::Return(value) = step_result else {
                    panic!("evaluating a classical expression should always return a value");
                };
                self.context
                    .expression_value_map
                    .insert_expr_value(store_expr_id, value);
            }
            Err((error, _)) => self.errors.push(Error::EvaluationFailed(error)),
        };
    }

    fn generate_expr_instructions(&mut self, _expr_id: ExprId) {
        unimplemented!();
    }

    fn get_current_package(&self) -> PackageId {
        self.context.get_current_callable_scope().package_id
    }
}

impl<'a> Visitor<'a> for PartialEvaluator<'a> {
    fn get_block(&self, id: BlockId) -> &'a Block {
        let block_id = StoreBlockId::from((self.get_current_package(), id));
        self.package_store.get_block(block_id)
    }

    fn get_expr(&self, id: ExprId) -> &'a Expr {
        let expr_id = StoreExprId::from((self.get_current_package(), id));
        self.package_store.get_expr(expr_id)
    }

    fn get_pat(&self, id: PatId) -> &'a Pat {
        let pat_id = StorePatId::from((self.get_current_package(), id));
        self.package_store.get_pat(pat_id)
    }

    fn get_stmt(&self, id: StmtId) -> &'a Stmt {
        let stmt_id = StoreStmtId::from((self.get_current_package(), id));
        self.package_store.get_stmt(stmt_id)
    }

    fn visit_expr(&mut self, expr_id: ExprId) {
        if !self.errors.is_empty() {
            return;
        }

        // If the expression can be classically evaluated, do it. Otherwise, generate instructions for it.
        let current_package_id = self.get_current_package();
        let store_expr_id = StoreExprId::from((current_package_id, expr_id));
        let expr_generator_set = self.compute_properties.get_expr(store_expr_id);
        let callable_scope = self.context.get_current_callable_scope();
        let compute_kind = expr_generator_set
            .generate_application_compute_kind(&callable_scope.args_runtime_properties);
        if matches!(compute_kind, ComputeKind::Classical) {
            self.eval_expr_classical(expr_id);
        } else {
            self.generate_expr_instructions(expr_id);
        }
    }
}

#[derive(Default)]
struct Assigner {
    next_callable: rir::CallableId,
    next_block: rir::BlockId,
}

impl Assigner {
    pub fn next_block(&mut self) -> rir::BlockId {
        let id = self.next_block;
        self.next_block = id.successor();
        id
    }

    pub fn next_callable(&mut self) -> rir::CallableId {
        let id = self.next_callable;
        self.next_callable = id.successor();
        id
    }
}

#[derive(Default)]
struct QubitsAndResultsAllocator {
    qubit_id: usize,
    result_id: usize,
}

impl Backend for QubitsAndResultsAllocator {
    type ResultType = usize;

    fn m(&mut self, _q: usize) -> Self::ResultType {
        self.next_measurement()
    }

    fn mresetz(&mut self, _q: usize) -> Self::ResultType {
        self.next_measurement()
    }

    fn qubit_allocate(&mut self) -> usize {
        self.next_qubit()
    }

    fn qubit_release(&mut self, _q: usize) {
        // Do nothing.
    }
}

impl QubitsAndResultsAllocator {
    fn next_measurement(&mut self) -> usize {
        let result_id = self.result_id;
        self.result_id += 1;
        result_id
    }

    fn next_qubit(&mut self) -> usize {
        let qubit_id = self.qubit_id;
        self.qubit_id += 1;
        qubit_id
    }
}

struct EvaluationContext {
    expression_value_map: ExpressionValueMap,
    _current_block: rir::BlockId,
    callables_stack: Vec<CallableScope>,
    env: Env,
}

impl EvaluationContext {
    fn new(
        entry_package_id: PackageId,
        package_store: &PackageStore,
        initial_block: rir::BlockId,
    ) -> Self {
        let entry_callable_scope = CallableScope {
            package_id: entry_package_id,
            _callable: None,
            args_runtime_properties: Vec::new(),
        };
        Self {
            expression_value_map: ExpressionValueMap::new(package_store),
            _current_block: initial_block,
            callables_stack: vec![entry_callable_scope],
            env: Env::default(),
        }
    }

    fn get_current_callable_scope(&self) -> &CallableScope {
        self.callables_stack
            .last()
            .expect("the evaluation context does not have a current callable scope")
    }
}

struct CallableScope {
    package_id: PackageId,
    _callable: Option<(LocalItemId, FunctorApp)>,
    args_runtime_properties: Vec<ValueKind>,
}

struct ExpressionValueMap(FxHashMap<PackageId, FxHashMap<ExprId, Value>>);

impl ExpressionValueMap {
    fn new(package_store: &PackageStore) -> Self {
        let mut expression_value_map = FxHashMap::<PackageId, FxHashMap<ExprId, Value>>::default();
        for (package_id, _) in package_store {
            expression_value_map.insert(package_id, FxHashMap::<ExprId, Value>::default());
        }
        Self(expression_value_map)
    }

    fn insert_expr_value(&mut self, store_expr_id: StoreExprId, value: Value) {
        let package_expr_value_map = self
            .0
            .get_mut(&store_expr_id.package)
            .expect("package expression value map does not exist");
        package_expr_value_map.insert(store_expr_id.expr, value);
    }
}
