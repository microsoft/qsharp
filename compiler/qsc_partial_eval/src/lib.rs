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
        Block, BlockId, CallableDecl, CallableImpl, ExecGraphNode, Expr, ExprId, ExprKind, Global,
        LocalItemId, PackageId, PackageStore, PackageStoreLookup, Pat, PatId, SpecDecl, SpecImpl,
        Stmt, StmtId, StmtKind, StoreBlockId, StoreExprId, StoreItemId, StorePatId, StoreStmtId,
    },
    visit::Visitor,
};
use qsc_rca::{ComputeKind, ComputePropertiesLookup, PackageStoreComputeProperties, ValueKind};
use qsc_rir::rir::{self, CallableType, Instruction, Program};
use rustc_hash::FxHashMap;
use std::{rc::Rc, result::Result};

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
    eval_context: EvaluationContext,
    program: Program,
    error: Option<Error>,
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

        // Initialize the evaluation context and create a new partial evaluator.
        let context = EvaluationContext::new(entry_package_id, entry_block_id);
        Self {
            package_store,
            compute_properties,
            eval_context: context,
            _assigner: assigner,
            backend: QubitsAndResultsAllocator::default(),
            program,
            error: None,
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn eval(mut self) -> Result<Program, Error> {
        let current_package = self.get_current_package_id();
        let entry_package = self.package_store.get(current_package);
        let Some(entry_expr_id) = entry_package.entry else {
            panic!("package does not have an entry expression");
        };

        // Visit the entry point expression.
        self.visit_expr(entry_expr_id);

        // Insert the return expression.
        let current_block = self
            .program
            .blocks
            .get_mut(self.eval_context.current_block)
            .expect("block does not exist");
        current_block.0.push(Instruction::Return);

        Ok(self.program)
    }

    fn eval_callee_expr(
        &mut self,
        callee_expr_id: ExprId,
    ) -> (StoreItemId, FunctorApp, CallableDecl) {
        let current_package_id = self.get_current_package_id();
        let store_callee_expr_id = StoreExprId::from((current_package_id, callee_expr_id));

        // Verify that the callee expression is classical.
        let callable_scope = self.eval_context.get_current_scope();
        let callee_expr_generator_set = self.compute_properties.get_expr(store_callee_expr_id);
        let callee_expr_compute_kind = callee_expr_generator_set
            .generate_application_compute_kind(&callable_scope.args_runtime_properties);
        assert!(matches!(callee_expr_compute_kind, ComputeKind::Classical));

        // Evaluate the callee expression to get the global to call.
        self.eval_classical_expr(callee_expr_id);
        let callable_scope = self.eval_context.get_current_scope();
        let callee_value = callable_scope
            .expression_value_map
            .get(&callee_expr_id)
            .expect("callee expression value not present");
        let Value::Global(store_item_id, functor_app) = callee_value else {
            panic!("callee expression value must be a global");
        };

        // Get the callable.
        let global = self
            .package_store
            .get_global(*store_item_id)
            .expect("global not present");
        let Global::Callable(callable_decl) = global else {
            // Instruction generation for UDTs is not supported.
            panic!("global is not a callable");
        };
        (*store_item_id, *functor_app, callable_decl.clone())
    }

    fn eval_classical_expr(&mut self, expr_id: ExprId) {
        let current_package_id = self.get_current_package_id();
        let store_expr_id = StoreExprId::from((current_package_id, expr_id));
        let expr = self.package_store.get_expr(store_expr_id);
        let scope_exec_graph = self.get_current_scope_exec_graph().clone();
        let scope = self.eval_context.get_current_scope_mut();
        let exec_graph = exec_graph_section(&scope_exec_graph, expr.exec_graph_range.clone());
        let mut out = Vec::new();
        let mut receiver = GenericReceiver::new(&mut out);
        let mut state = State::new(current_package_id, exec_graph, None);
        let eval_result = state.eval(
            self.package_store,
            &mut scope.env,
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
                self.eval_context
                    .get_current_scope_mut()
                    .insert_expr_value(expr_id, value);
            }
            Err((error, _)) => self.error = Some(Error::EvaluationFailed(error)),
        };
    }

    fn generate_expr_call(&mut self, callee_expr_id: ExprId, args_expr_id: ExprId) {
        let (store_item_id, functor_app, callable_decl) = self.eval_callee_expr(callee_expr_id);

        // We generate instructions differently depending on whether
        match &callable_decl.implementation {
            CallableImpl::Intrinsic => {
                self.generate_expr_call_intrinsic(&callable_decl, args_expr_id);
            }
            CallableImpl::Spec(spec_impl) => {
                self.generate_expr_call_spec(store_item_id, functor_app, spec_impl, args_expr_id);
            }
        };
    }

    fn generate_expr_call_intrinsic(
        &mut self,
        _callable_decl: &CallableDecl,
        _args_expr_id: ExprId,
    ) {
        unimplemented!();
    }

    fn generate_expr_call_spec(
        &mut self,
        global_callable: StoreItemId,
        functor_app: FunctorApp,
        spec_impl: &SpecImpl,
        _args_expr_id: ExprId,
    ) {
        let spec_decl = get_spec_decl(spec_impl, functor_app);

        // We are currently not setting the argument values in a way that supports arbitrary calls, but we'll add that
        // support later.
        let callable_scope = Scope {
            package_id: global_callable.package,
            callable: Some((global_callable.item, functor_app)),
            args_runtime_properties: Vec::new(),
            env: Env::default(),
            expression_value_map: FxHashMap::default(),
        };
        self.eval_context.scopes.push(callable_scope);
        self.visit_block(spec_decl.block);
        let popped_scope = self
            .eval_context
            .scopes
            .pop()
            .expect("there are no callable scopes to pop");
        assert!(popped_scope.package_id == global_callable.package);
        let (popped_callable_id, popped_functor_app) = popped_scope
            .callable
            .expect("callable in scope is not specified");
        assert!(popped_callable_id == global_callable.item);
        assert!(popped_functor_app == functor_app);
    }

    fn generate_instructions(&mut self, expr_id: ExprId) {
        let current_package_id = self.get_current_package_id();
        let store_expr_id = StoreExprId::from((current_package_id, expr_id));
        let expr = self.package_store.get_expr(store_expr_id);
        match &expr.kind {
            ExprKind::Array(_) => todo!(),
            ExprKind::ArrayLit(_) => todo!(),
            ExprKind::ArrayRepeat(_, _) => todo!(),
            ExprKind::Assign(_, _) => todo!(),
            ExprKind::AssignField(_, _, _) => todo!(),
            ExprKind::AssignIndex(_, _, _) => todo!(),
            ExprKind::AssignOp(_, _, _) => todo!(),
            ExprKind::BinOp(_, _, _) => todo!(),
            ExprKind::Block(_) => todo!(),
            ExprKind::Call(callee_expr_id, args_expr_id) => {
                self.generate_expr_call(*callee_expr_id, *args_expr_id);
            }
            ExprKind::Closure(_, _) => {
                panic!("instruction generation for closure expressions is unsupported")
            }
            ExprKind::Fail(_) => panic!("instruction generation for fail expression is invalid"),
            ExprKind::Field(_, _) => todo!(),
            ExprKind::Hole => panic!("instruction generation for hole expressions is invalid"),
            ExprKind::If(_, _, _) => todo!(),
            ExprKind::Index(_, _) => todo!(),
            ExprKind::Lit(_) => panic!("instruction generation for literal expressions is invalid"),
            ExprKind::Range(_, _, _) => {
                panic!("instruction generation for range expressions is invalid")
            }
            ExprKind::Return(_) => todo!(),
            ExprKind::String(_) => {
                panic!("instruction generation for string expressions is invalid")
            }
            ExprKind::Tuple(_) => todo!(),
            ExprKind::UnOp(_, _) => todo!(),
            ExprKind::UpdateField(_, _, _) => todo!(),
            ExprKind::UpdateIndex(_, _, _) => todo!(),
            ExprKind::Var(_, _) => todo!(),
            ExprKind::While(_, _) => todo!(),
        };
    }

    fn get_current_package_id(&self) -> PackageId {
        self.eval_context.get_current_scope().package_id
    }

    fn get_current_scope_exec_graph(&self) -> &Rc<[ExecGraphNode]> {
        if let Some(spec_decl) = self.get_current_scope_spec_decl() {
            &spec_decl.exec_graph
        } else {
            let package_id = self.get_current_package_id();
            let package = self.package_store.get(package_id);
            &package.entry_exec_graph
        }
    }

    fn get_current_scope_spec_decl(&self) -> Option<&SpecDecl> {
        let current_scope = self.eval_context.get_current_scope();
        let Some((local_item_id, functor_app)) = current_scope.callable else {
            return None;
        };
        let store_item_id = StoreItemId::from((current_scope.package_id, local_item_id));
        let global = self
            .package_store
            .get_global(store_item_id)
            .expect("global does not exist");
        let Global::Callable(callable_decl) = global else {
            panic!("global is not a callable");
        };

        let CallableImpl::Spec(spec_impl) = &callable_decl.implementation else {
            panic!("callable does not implement specializations");
        };

        let spec_decl = get_spec_decl(spec_impl, functor_app);
        Some(spec_decl)
    }

    fn is_classical_stmt(&self, stmt_id: StmtId) -> bool {
        let current_package_id = self.get_current_package_id();
        let store_stmt_id = StoreStmtId::from((current_package_id, stmt_id));
        let stmt_generator_set = self.compute_properties.get_stmt(store_stmt_id);
        let callable_scope = self.eval_context.get_current_scope();
        let compute_kind = stmt_generator_set
            .generate_application_compute_kind(&callable_scope.args_runtime_properties);
        matches!(compute_kind, ComputeKind::Classical)
    }

    fn is_qubit_allocation_stmt(&mut self, stmt_id: StmtId) -> bool {
        let current_package_id = self.get_current_package_id();
        let store_stmt_id = StoreStmtId::from((current_package_id, stmt_id));
        let stmt = self.package_store.get_stmt(store_stmt_id);
        if let StmtKind::Local(_, _, expr_id) = &stmt.kind {
            self.is_qubit_allocation_expr(*expr_id)
        } else {
            false
        }
    }

    fn is_qubit_release_stmt(&mut self, stmt_id: StmtId) -> bool {
        let current_package_id = self.get_current_package_id();
        let store_stmt_id = StoreStmtId::from((current_package_id, stmt_id));
        let stmt = self.package_store.get_stmt(store_stmt_id);
        if let StmtKind::Local(_, _, expr_id) = &stmt.kind {
            self.is_qubit_release_expr(*expr_id)
        } else {
            false
        }
    }

    fn is_qubit_allocation_expr(&mut self, expr_id: ExprId) -> bool {
        let current_package_id = self.get_current_package_id();
        let store_expr_id = StoreExprId::from((current_package_id, expr_id));
        let expr = self.package_store.get_expr(store_expr_id);
        if let ExprKind::Call(callee_expr_id, _) = &expr.kind {
            let (_, _, callable_decl) = self.eval_callee_expr(*callee_expr_id);
            callable_decl
                .name
                .name
                .to_string()
                .eq("__quantum__rt__qubit_allocate")
        } else {
            false
        }
    }

    fn is_qubit_release_expr(&mut self, expr_id: ExprId) -> bool {
        let current_package_id = self.get_current_package_id();
        let store_expr_id = StoreExprId::from((current_package_id, expr_id));
        let expr = self.package_store.get_expr(store_expr_id);
        if let ExprKind::Call(callee_expr_id, _) = &expr.kind {
            let (_, _, callable_decl) = self.eval_callee_expr(*callee_expr_id);
            callable_decl
                .name
                .name
                .to_string()
                .eq("__quantum__rt__qubit_release")
        } else {
            false
        }
    }
}

impl<'a> Visitor<'a> for PartialEvaluator<'a> {
    fn get_block(&self, id: BlockId) -> &'a Block {
        let block_id = StoreBlockId::from((self.get_current_package_id(), id));
        self.package_store.get_block(block_id)
    }

    fn get_expr(&self, id: ExprId) -> &'a Expr {
        let expr_id = StoreExprId::from((self.get_current_package_id(), id));
        self.package_store.get_expr(expr_id)
    }

    fn get_pat(&self, id: PatId) -> &'a Pat {
        let pat_id = StorePatId::from((self.get_current_package_id(), id));
        self.package_store.get_pat(pat_id)
    }

    fn get_stmt(&self, id: StmtId) -> &'a Stmt {
        let stmt_id = StoreStmtId::from((self.get_current_package_id(), id));
        self.package_store.get_stmt(stmt_id)
    }

    fn visit_expr(&mut self, expr_id: ExprId) {
        if self.error.is_some() {
            return;
        }

        // If the expression can be classically evaluated, do it. Otherwise, generate instructions for it.
        let current_package_id = self.get_current_package_id();
        let store_expr_id = StoreExprId::from((current_package_id, expr_id));
        let expr_generator_set = self.compute_properties.get_expr(store_expr_id);
        let callable_scope = self.eval_context.get_current_scope();
        let compute_kind = expr_generator_set
            .generate_application_compute_kind(&callable_scope.args_runtime_properties);
        if matches!(compute_kind, ComputeKind::Classical) {
            self.eval_classical_expr(expr_id);
        } else {
            self.generate_instructions(expr_id);
        }
    }

    fn visit_stmt(&mut self, stmt_id: StmtId) {
        if self.is_classical_stmt(stmt_id)
            || self.is_qubit_allocation_stmt(stmt_id)
            || self.is_qubit_release_stmt(stmt_id)
        {
            // Evaluate the statement.
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
    current_block: rir::BlockId,
    scopes: Vec<Scope>,
}

impl EvaluationContext {
    fn new(entry_package_id: PackageId, initial_block: rir::BlockId) -> Self {
        let entry_callable_scope = Scope {
            package_id: entry_package_id,
            callable: None,
            args_runtime_properties: Vec::new(),
            env: Env::default(),
            expression_value_map: FxHashMap::default(),
        };
        Self {
            current_block: initial_block,
            scopes: vec![entry_callable_scope],
        }
    }

    fn get_current_scope(&self) -> &Scope {
        self.scopes
            .last()
            .expect("the evaluation context does not have a current scope")
    }

    fn get_current_scope_mut(&mut self) -> &mut Scope {
        self.scopes
            .last_mut()
            .expect("the evaluation context does not have a current scope")
    }
}

struct Scope {
    package_id: PackageId,
    callable: Option<(LocalItemId, FunctorApp)>,
    args_runtime_properties: Vec<ValueKind>,
    env: Env,
    expression_value_map: FxHashMap<ExprId, Value>,
}

impl Scope {
    fn insert_expr_value(&mut self, expr_id: ExprId, value: Value) {
        self.expression_value_map.insert(expr_id, value);
    }
}

fn get_spec_decl(spec_impl: &SpecImpl, functor_app: FunctorApp) -> &SpecDecl {
    if !functor_app.adjoint && functor_app.controlled == 0 {
        &spec_impl.body
    } else if functor_app.adjoint && functor_app.controlled == 0 {
        spec_impl
            .adj
            .as_ref()
            .expect("adjoint specialization does not exist")
    } else if !functor_app.adjoint && functor_app.controlled > 0 {
        spec_impl
            .ctl
            .as_ref()
            .expect("controlled specialization does not exist")
    } else {
        spec_impl
            .ctl_adj
            .as_ref()
            .expect("controlled adjoint specialization does not exits")
    }
}
