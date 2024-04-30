// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The Q# partial evaluator residualizes a Q# program, producing RIR from FIR.
//! It does this by evaluating all purely classical expressions and generating RIR instructions for expressions that are
//! not purely classical.

mod evaluation_context;
mod management;

use evaluation_context::{
    Arg, BlockNode, BranchControlFlow, EvalControlFlow, EvaluationContext, Scope,
};
use management::{QuantumIntrinsicsChecker, ResourceManager};
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_data_structures::{functors::FunctorApp, target::TargetCapabilityFlags};
use qsc_eval::val::VarTy;
use qsc_eval::{
    self, exec_graph_section,
    output::GenericReceiver,
    val::{self, Value, Var},
    State, StepAction, StepResult, Variable,
};
use qsc_fir::fir::LocalVarId;
use qsc_fir::{
    fir::{
        self, BinOp, Block, BlockId, CallableDecl, CallableImpl, ExecGraphNode, Expr, ExprId,
        ExprKind, Global, Ident, Mutability, PackageId, PackageStore, PackageStoreLookup, Pat,
        PatId, PatKind, Res, SpecDecl, SpecImpl, Stmt, StmtId, StmtKind, StoreBlockId, StoreExprId,
        StoreItemId, StorePatId, StoreStmtId,
    },
    ty::{Prim, Ty},
};
use qsc_rca::{ComputeKind, ComputePropertiesLookup, PackageStoreComputeProperties};
use qsc_rir::{
    builder,
    rir::{
        self, Callable, CallableId, CallableType, ConditionCode, Instruction, Literal, Operand,
        Program,
    },
};
use rustc_hash::FxHashMap;
use std::{collections::hash_map::Entry, rc::Rc, result::Result};
use thiserror::Error;

/// Partially evaluates a program with the specified entry expression.
pub fn partially_evaluate(
    package_store: &PackageStore,
    compute_properties: &PackageStoreComputeProperties,
    entry: &ProgramEntry,
    capabilities: TargetCapabilityFlags,
) -> Result<Program, Error> {
    let partial_evaluator =
        PartialEvaluator::new(package_store, compute_properties, entry, capabilities);
    partial_evaluator.eval()
}

/// A partial evaluation error.
#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("partial evaluation failed with error {0}")]
    #[diagnostic(code("Qsc.PartialEval.EvaluationFailed"))]
    EvaluationFailed(String, #[label] Span),

    #[error("unsupported Result literal in output")]
    #[diagnostic(help(
        "Result literals `One` and `Zero` cannot be included in generated QIR output recording."
    ))]
    #[diagnostic(code("Qsc.PartialEval.OutputResultLiteral"))]
    OutputResultLiteral(#[label] Span),

    #[error("an unexpected error occurred related to: {0}")]
    #[diagnostic(code("Qsc.PartialEval.Unexpected"))]
    #[diagnostic(help(
        "this is probably a bug. please consider reporting this as an issue to the development team"
    ))]
    Unexpected(String, #[label] Span),

    #[error("failed to evaluate: {0} not yet implemented")]
    #[diagnostic(code("Qsc.PartialEval.Unimplemented"))]
    Unimplemented(String, #[label] Span),
}

/// An entry to the program to be partially evaluated.
pub struct ProgramEntry {
    /// The execution graph that corresponds to the entry expression.
    pub exec_graph: Rc<[ExecGraphNode]>,
    /// The entry expression unique identifier within a package store.
    pub expr: fir::StoreExprId,
}

struct PartialEvaluator<'a> {
    package_store: &'a PackageStore,
    compute_properties: &'a PackageStoreComputeProperties,
    resource_manager: ResourceManager,
    backend: QuantumIntrinsicsChecker,
    callables_map: FxHashMap<Rc<str>, CallableId>,
    eval_context: EvaluationContext,
    program: Program,
    entry: &'a ProgramEntry,
}

impl<'a> PartialEvaluator<'a> {
    fn new(
        package_store: &'a PackageStore,
        compute_properties: &'a PackageStoreComputeProperties,
        entry: &'a ProgramEntry,
        capabilities: TargetCapabilityFlags,
    ) -> Self {
        // Create the entry-point callable.
        let mut resource_manager = ResourceManager::default();
        let mut program = Program::new();
        program.config.capabilities = capabilities;
        let entry_block_id = resource_manager.next_block();
        program.blocks.insert(entry_block_id, rir::Block::default());
        let entry_point_id = resource_manager.next_callable();
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
        let context = EvaluationContext::new(entry.expr.package, entry_block_id);
        Self {
            package_store,
            compute_properties,
            eval_context: context,
            resource_manager,
            backend: QuantumIntrinsicsChecker::default(),
            callables_map: FxHashMap::default(),
            program,
            entry,
        }
    }

    fn bind_value_to_pat(&mut self, mutability: Mutability, pat_id: PatId, value: Value) {
        let pat = self.get_pat(pat_id);
        match &pat.kind {
            PatKind::Bind(ident) => {
                self.bind_value_to_ident(mutability, ident, value);
            }
            PatKind::Tuple(pats) => {
                let tup = value.unwrap_tuple();
                assert!(pats.len() == tup.len());
                for (pat_id, value) in pats.iter().zip(tup.iter()) {
                    self.bind_value_to_pat(mutability, *pat_id, value.clone());
                }
            }
            PatKind::Discard => {
                // Nothing to bind to.
            }
        }
    }

    fn bind_value_to_ident(&mut self, mutability: Mutability, ident: &Ident, value: Value) {
        // We do slightly different things depending on the mutability of the identifier.
        match mutability {
            Mutability::Immutable => self.bind_value_to_immutable_ident(ident, value),
            Mutability::Mutable => self.bind_value_to_mutable_ident(ident, value),
        };
    }

    fn bind_value_to_immutable_ident(&mut self, ident: &Ident, value: Value) {
        // Just insert the value into the hybrid vars map.
        self.eval_context
            .get_current_scope_mut()
            .hybrid_vars
            .insert(ident.id, value);
    }

    fn bind_value_to_mutable_ident(&mut self, ident: &Ident, value: Value) {
        let maybe_var_type = try_get_eval_var_type(&value);
        if let Some(var_type) = maybe_var_type {
            // Get a variable to store into.
            let value_operand = map_eval_value_to_rir_operand(&value);
            let eval_var = self.get_or_create_variable(ident.id, var_type);
            let rir_var = map_eval_var_to_rir_var(eval_var);

            // Insert a store instruction.
            let store_ins = Instruction::Store(value_operand, rir_var);
            self.get_current_rir_block_mut().0.push(store_ins);
        } else {
            // Insert the value into the hybrid vars map.
            self.eval_context
                .get_current_scope_mut()
                .hybrid_vars
                .insert(ident.id, value);
        }
    }

    fn create_intrinsic_callable(
        &self,
        store_item_id: StoreItemId,
        callable_decl: &CallableDecl,
    ) -> Callable {
        let callable_package = self.package_store.get(store_item_id.package);
        let name = callable_decl.name.name.to_string();
        let input_type: Vec<rir::Ty> = callable_package
            .derive_callable_input_params(callable_decl)
            .iter()
            .map(|input_param| map_fir_type_to_rir_type(&input_param.ty))
            .collect();
        let output_type = if callable_decl.output == Ty::UNIT {
            None
        } else {
            Some(map_fir_type_to_rir_type(&callable_decl.output))
        };
        let body = None;
        let call_type = if name.eq("__quantum__qis__reset__body") {
            CallableType::Reset
        } else {
            CallableType::Regular
        };
        Callable {
            name,
            input_type,
            output_type,
            body,
            call_type,
        }
    }

    fn create_program_block(&mut self) -> rir::BlockId {
        let block_id = self.resource_manager.next_block();
        self.program.blocks.insert(block_id, rir::Block::default());
        block_id
    }

    fn eval(mut self) -> Result<Program, Error> {
        // Evaluate the entry-point expression.
        let ret_val = self.try_eval_expr(self.entry.expr.expr)?.into_value();
        let output_recording: Vec<Instruction> = self
            .generate_output_recording_instructions(
                ret_val,
                &self.get_expr(self.entry.expr.expr).ty,
            )
            .map_err(|()| Error::OutputResultLiteral(self.get_expr(self.entry.expr.expr).span))?;

        // Insert the return expression and return the generated program.
        let current_block = self.get_current_rir_block_mut();
        current_block.0.extend(output_recording);
        current_block.0.push(Instruction::Return);

        // Set the number of qubits and results used by the program.
        self.program.num_qubits = self
            .resource_manager
            .qubit_count()
            .try_into()
            .expect("qubits count should fit into a u32");
        self.program.num_results = self
            .resource_manager
            .result_register_count()
            .try_into()
            .expect("results count should fit into a u32");

        Ok(self.program)
    }

    fn eval_classical_expr(&mut self, expr_id: ExprId) -> Result<EvalControlFlow, Error> {
        let current_package_id = self.get_current_package_id();
        let store_expr_id = StoreExprId::from((current_package_id, expr_id));
        let expr = self.package_store.get_expr(store_expr_id);
        let scope_exec_graph = self.get_current_scope_exec_graph().clone();
        let scope = self.eval_context.get_current_scope_mut();
        let exec_graph = exec_graph_section(&scope_exec_graph, expr.exec_graph_range.clone());
        let mut state = State::new(current_package_id, exec_graph, None);
        let eval_result = state.eval(
            self.package_store,
            &mut scope.env,
            &mut self.backend,
            &mut GenericReceiver::new(&mut std::io::sink()),
            &[],
            StepAction::Continue,
        );
        match eval_result {
            Ok(step_result) => {
                let StepResult::Return(value) = step_result else {
                    panic!("evaluating a classical expression should always return a value");
                };

                // Figure out the control flow kind.
                let scope = self.eval_context.get_current_scope();
                let eval_control_flow = if scope.has_classical_evaluator_returned() {
                    EvalControlFlow::Return(value)
                } else {
                    EvalControlFlow::Continue(value)
                };
                Ok(eval_control_flow)
            }
            Err((error, _)) => Err(Error::EvaluationFailed(error.to_string(), expr.span)),
        }
    }

    fn eval_classical_stmt(&mut self, stmt_id: StmtId) -> Result<EvalControlFlow, Error> {
        let current_package_id = self.get_current_package_id();
        let stmt = self.get_stmt(stmt_id);
        let scope_exec_graph = self.get_current_scope_exec_graph().clone();
        let scope = self.eval_context.get_current_scope_mut();
        let exec_graph = exec_graph_section(&scope_exec_graph, stmt.exec_graph_range.clone());
        let mut state = State::new(current_package_id, exec_graph, None);
        let eval_result = state.eval(
            self.package_store,
            &mut scope.env,
            &mut self.backend,
            &mut GenericReceiver::new(&mut std::io::sink()),
            &[],
            StepAction::Continue,
        );
        match eval_result {
            Ok(step_result) => {
                let StepResult::Return(value) = step_result else {
                    panic!("evaluating a classical expression should always return a value");
                };

                // Figure out the control flow kind.
                let scope = self.eval_context.get_current_scope();
                let eval_control_flow = if scope.has_classical_evaluator_returned() {
                    EvalControlFlow::Return(value)
                } else {
                    EvalControlFlow::Continue(value)
                };
                Ok(eval_control_flow)
            }
            Err((error, _)) => Err(Error::EvaluationFailed(error.to_string(), stmt.span)),
        }
    }

    fn eval_hybrid_expr(&mut self, expr_id: ExprId) -> Result<EvalControlFlow, Error> {
        let current_package_id = self.get_current_package_id();
        let store_expr_id = StoreExprId::from((current_package_id, expr_id));
        let expr = self.package_store.get_expr(store_expr_id);
        match &expr.kind {
            ExprKind::Array(exprs) => self.eval_expr_array(exprs),
            ExprKind::ArrayLit(_) => panic!("array of literal values should always be classical"),
            ExprKind::ArrayRepeat(value_expr_id, size_expr_id) => {
                self.eval_expr_array_repeat(*value_expr_id, *size_expr_id)
            }
            ExprKind::Assign(lhs_expr_id, rhs_expr_id) => {
                self.eval_expr_assign(*lhs_expr_id, *rhs_expr_id)
            }
            ExprKind::AssignField(_, _, _) => Err(Error::Unimplemented(
                "Field Assignment Expr".to_string(),
                expr.span,
            )),
            ExprKind::AssignIndex(_, _, _) => Err(Error::Unimplemented(
                "Assignment Index Expr".to_string(),
                expr.span,
            )),
            ExprKind::AssignOp(_, _, _) => Err(Error::Unimplemented(
                "Assignment Op Expr".to_string(),
                expr.span,
            )),
            ExprKind::BinOp(bin_op, lhs_expr_id, rhs_expr_id) => {
                self.eval_expr_bin_op(expr_id, *bin_op, *lhs_expr_id, *rhs_expr_id)
            }
            ExprKind::Block(block_id) => self.try_eval_block(*block_id),
            ExprKind::Call(callee_expr_id, args_expr_id) => {
                self.eval_expr_call(*callee_expr_id, *args_expr_id)
            }
            ExprKind::Closure(_, _) => {
                panic!("instruction generation for closure expressions is unsupported")
            }
            ExprKind::Fail(_) => panic!("instruction generation for fail expression is invalid"),
            ExprKind::Field(_, _) => Err(Error::Unimplemented("Field Expr".to_string(), expr.span)),
            ExprKind::Hole => panic!("instruction generation for hole expressions is invalid"),
            ExprKind::If(condition_expr_id, body_expr_id, otherwise_expr_id) => self.eval_expr_if(
                expr_id,
                *condition_expr_id,
                *body_expr_id,
                *otherwise_expr_id,
            ),
            ExprKind::Index(_, _) => Err(Error::Unimplemented("Index Expr".to_string(), expr.span)),
            ExprKind::Lit(_) => panic!("instruction generation for literal expressions is invalid"),
            ExprKind::Range(_, _, _) => {
                panic!("instruction generation for range expressions is invalid")
            }
            ExprKind::Return(expr_id) => self.eval_expr_return(*expr_id),
            ExprKind::String(_) => {
                panic!("instruction generation for string expressions is invalid")
            }
            ExprKind::Tuple(exprs) => self.eval_expr_tuple(exprs),
            ExprKind::UnOp(_, _) => Err(Error::Unimplemented("Unary Expr".to_string(), expr.span)),
            ExprKind::UpdateField(_, _, _) => Err(Error::Unimplemented(
                "Updated Field Expr".to_string(),
                expr.span,
            )),
            ExprKind::UpdateIndex(_, _, _) => Err(Error::Unimplemented(
                "Update Index Expr".to_string(),
                expr.span,
            )),
            ExprKind::Var(res, _) => Ok(EvalControlFlow::Continue(self.eval_expr_var(res))),
            ExprKind::While(condition_expr_id, body_block_id) => {
                self.eval_expr_while(*condition_expr_id, *body_block_id)
            }
        }
    }

    fn eval_hybrid_stmt(&mut self, stmt_id: StmtId) -> Result<EvalControlFlow, Error> {
        let stmt = self.get_stmt(stmt_id);
        match stmt.kind {
            StmtKind::Expr(expr_id) => self.try_eval_expr(expr_id),
            StmtKind::Semi(expr_id) => {
                let control_flow = self.try_eval_expr(expr_id)?;
                match control_flow {
                    EvalControlFlow::Continue(_) => Ok(EvalControlFlow::Continue(Value::unit())),
                    EvalControlFlow::Return(_) => Ok(control_flow),
                }
            }
            StmtKind::Local(mutability, pat_id, expr_id) => {
                let control_flow = self.try_eval_expr(expr_id)?;
                match control_flow {
                    EvalControlFlow::Continue(value) => {
                        self.bind_value_to_pat(mutability, pat_id, value);
                        Ok(EvalControlFlow::Continue(Value::unit()))
                    }
                    EvalControlFlow::Return(_) => Ok(control_flow),
                }
            }
            StmtKind::Item(_) => {
                // Do nothing and return a continue unit value.
                Ok(EvalControlFlow::Continue(Value::unit()))
            }
        }
    }

    fn eval_expr_array_repeat(
        &mut self,
        value_expr_id: ExprId,
        size_expr_id: ExprId,
    ) -> Result<EvalControlFlow, Error> {
        // Try to evaluate both the value and size expressions to get their value, short-circuiting execution if any of the
        // expressions is a return.
        let value_control_flow = self.try_eval_expr(value_expr_id)?;
        let EvalControlFlow::Continue(value) = value_control_flow else {
            let value_expr = self.get_expr(value_expr_id);
            return Err(Error::Unexpected(
                "embedded return in array value".to_string(),
                value_expr.span,
            ));
        };
        let size_control_flow = self.try_eval_expr(size_expr_id)?;
        let EvalControlFlow::Continue(size) = size_control_flow else {
            let size_expr = self.get_expr(size_expr_id);
            return Err(Error::Unexpected(
                "embedded return in array size".to_string(),
                size_expr.span,
            ));
        };

        // We assume the size of the array is a classical value because otherwise it would have been rejected before
        // getting to the partial evaluation stage.
        let size = size.unwrap_int();
        let values = vec![value; TryFrom::try_from(size).expect("could not convert size value")];
        Ok(EvalControlFlow::Continue(Value::Array(values.into())))
    }

    fn eval_expr_assign(
        &mut self,
        _lhs_expr_id: ExprId,
        _rhs_expr_id: ExprId,
    ) -> Result<EvalControlFlow, Error> {
        unimplemented!();
    }

    #[allow(clippy::similar_names)]
    fn eval_expr_bin_op(
        &mut self,
        bin_op_expr_id: ExprId,
        bin_op: BinOp,
        lhs_expr_id: ExprId,
        rhs_expr_id: ExprId,
    ) -> Result<EvalControlFlow, Error> {
        // Try to evaluate both the LHS and RHS expressions to get their value, short-circuiting execution if any of the
        // expressions is a return.
        let lhs_control_flow = self.try_eval_expr(lhs_expr_id)?;
        if lhs_control_flow.is_return() {
            let lhs_expr = self.get_expr(lhs_expr_id);
            return Err(Error::Unexpected(
                "embedded return in binary operation".to_string(),
                lhs_expr.span,
            ));
        }
        let rhs_control_flow = self.try_eval_expr(rhs_expr_id)?;
        if rhs_control_flow.is_return() {
            let rhs_expr = self.get_expr(rhs_expr_id);
            return Err(Error::Unexpected(
                "embedded return in binary operation".to_string(),
                rhs_expr.span,
            ));
        }

        // Get the operands to use when generating the binary operation instruction depending on the type of the
        // expression's value.
        let lhs_value = lhs_control_flow.into_value();
        let lhs_operand = if let Value::Result(result) = lhs_value {
            self.eval_result_as_bool_operand(result)
        } else {
            map_eval_value_to_rir_operand(&lhs_value)
        };
        let rhs_value = rhs_control_flow.into_value();
        let rhs_operand = if let Value::Result(result) = rhs_value {
            self.eval_result_as_bool_operand(result)
        } else {
            map_eval_value_to_rir_operand(&rhs_value)
        };

        // Create a variable to store the result of the expression.
        let bin_op_expr = self.get_expr(bin_op_expr_id);
        let variable_id = self.resource_manager.next_var();
        let variable_ty = map_fir_type_to_rir_type(&bin_op_expr.ty);
        let variable = rir::Variable {
            id: variable_id,
            ty: variable_ty,
        };

        // Create the binary operation instruction and add it to the current block.
        let instruction = match bin_op {
            BinOp::Eq => Instruction::Icmp(ConditionCode::Eq, lhs_operand, rhs_operand, variable),
            BinOp::Neq => Instruction::Icmp(ConditionCode::Ne, lhs_operand, rhs_operand, variable),
            _ => {
                return Err(Error::Unimplemented(
                    format!("BinOp Expr ({bin_op:?})"),
                    bin_op_expr.span,
                ))
            }
        };
        let current_block = self.get_current_rir_block_mut();
        current_block.0.push(instruction);

        // Return the variable as a value.
        let value = Value::Var(map_rir_var_to_eval_var(variable));
        Ok(EvalControlFlow::Continue(value))
    }

    fn eval_expr_call(
        &mut self,
        callee_expr_id: ExprId,
        args_expr_id: ExprId,
    ) -> Result<EvalControlFlow, Error> {
        // Visit the both the callee and arguments expressions to get their values.
        let callee_control_flow = self.try_eval_expr(callee_expr_id)?;
        if callee_control_flow.is_return() {
            let callee_expr = self.get_expr(callee_expr_id);
            return Err(Error::Unexpected(
                "embedded return in callee".to_string(),
                callee_expr.span,
            ));
        }

        let args_control_flow = self.try_eval_expr(args_expr_id)?;
        if args_control_flow.is_return() {
            let args_expr = self.get_expr(args_expr_id);
            return Err(Error::Unexpected(
                "embedded return in call arguments".to_string(),
                args_expr.span,
            ));
        }

        // Get the callable.
        let (store_item_id, functor_app) = callee_control_flow.into_value().unwrap_global();
        let global = self
            .package_store
            .get_global(store_item_id)
            .expect("global not present");
        let Global::Callable(callable_decl) = global else {
            // Instruction generation for UDTs is not supported.
            panic!("global is not a callable");
        };

        // We generate instructions differently depending on whether we are calling an intrinsic or a specialization
        // with an implementation.
        let value = match &callable_decl.implementation {
            CallableImpl::Intrinsic => self.eval_expr_call_to_intrinsic(
                store_item_id,
                callable_decl,
                args_control_flow.into_value(),
            ),
            CallableImpl::Spec(spec_impl) => self.eval_expr_call_to_spec(
                store_item_id,
                functor_app,
                spec_impl,
                callable_decl.input,
                args_control_flow.into_value(),
            )?,
        };
        Ok(EvalControlFlow::Continue(value))
    }

    fn eval_expr_call_to_intrinsic(
        &mut self,
        store_item_id: StoreItemId,
        callable_decl: &CallableDecl,
        args_value: Value,
    ) -> Value {
        // There are a few special cases regarding intrinsic callables. Identify them and handle them properly.
        match callable_decl.name.name.as_ref() {
            // Qubit allocations and measurements have special handling.
            "__quantum__rt__qubit_allocate" => self.allocate_qubit(),
            "__quantum__rt__qubit_release" => self.release_qubit(args_value),
            "__quantum__qis__m__body" => self.measure_qubit(builder::mz_decl(), args_value),
            "__quantum__qis__mresetz__body" => {
                self.measure_qubit(builder::mresetz_decl(), args_value)
            }
            // The following operations should be conditionally compiled out for all targets for which QIR generation is
            // supported.
            "CheckZero" | "DrawRandomInt" | "DrawRandomDouble" => panic!(
                "`{}` is not a supported by partial evaluation",
                callable_decl.name.name
            ),
            // The following intrinsic operations and functions are no-ops.
            "BeginEstimateCaching" => Value::Bool(true),
            "DumpRegister"
            | "AccountForEstimatesInternal"
            | "BeginRepeatEstimatesInternal"
            | "EndRepeatEstimatesInternal" => Value::unit(),
            _ => self.eval_expr_call_to_intrinsic_qis(store_item_id, callable_decl, args_value),
        }
    }

    fn eval_expr_call_to_intrinsic_qis(
        &mut self,
        store_item_id: StoreItemId,
        callable_decl: &CallableDecl,
        args_value: Value,
    ) -> Value {
        // Intrinsic callables that make it to this point are expected to be unitary.
        assert_eq!(callable_decl.output, Ty::UNIT);

        // Check if the callable is already in the program, and if not add it.
        let callable = self.create_intrinsic_callable(store_item_id, callable_decl);
        let callable_id = self.get_or_insert_callable(callable);

        // Resove the call arguments, create the call instruction and insert it to the current block.
        let args = self.resolve_args(
            (store_item_id.package, callable_decl.input).into(),
            args_value,
        );
        let args_operands = args
            .into_iter()
            .map(|arg| map_eval_value_to_rir_operand(&arg.into_value()))
            .collect();

        let instruction = Instruction::Call(callable_id, args_operands, None);
        let current_block = self.get_current_rir_block_mut();
        current_block.0.push(instruction);
        Value::unit()
    }

    fn eval_expr_call_to_spec(
        &mut self,
        global_callable_id: StoreItemId,
        functor_app: FunctorApp,
        spec_impl: &SpecImpl,
        args_pat: PatId,
        args_value: Value,
    ) -> Result<Value, Error> {
        let spec_decl = get_spec_decl(spec_impl, functor_app);

        // Create new call scope.
        let args = self.resolve_args((global_callable_id.package, args_pat).into(), args_value);
        let call_scope = Scope::new(
            global_callable_id.package,
            Some((global_callable_id.item, functor_app)),
            args,
        );
        self.eval_context.push_scope(call_scope);
        let block_value = self.try_eval_block(spec_decl.block)?.into_value();
        let popped_scope = self.eval_context.pop_scope();
        assert!(
            popped_scope.package_id == global_callable_id.package,
            "scope package ID mismatch"
        );
        let (popped_callable_id, popped_functor_app) = popped_scope
            .callable
            .expect("callable in scope is not specified");
        assert!(
            popped_callable_id == global_callable_id.item,
            "scope callable ID mismatch"
        );
        assert!(popped_functor_app == functor_app, "scope functor mismatch");
        Ok(block_value)
    }

    fn eval_expr_if(
        &mut self,
        if_expr_id: ExprId,
        condition_expr_id: ExprId,
        body_expr_id: ExprId,
        otherwise_expr_id: Option<ExprId>,
    ) -> Result<EvalControlFlow, Error> {
        // Visit the the condition expression to get its value.
        let condition_control_flow = self.try_eval_expr(condition_expr_id)?;
        if condition_control_flow.is_return() {
            let condition_expr = self.get_expr(condition_expr_id);
            return Err(Error::Unexpected(
                "embedded return in if condition".to_string(),
                condition_expr.span,
            ));
        }

        // If the condition value is a Boolean literal, use the value to decide which branch to
        // evaluate.
        let condition_value = condition_control_flow.into_value();
        if let Value::Bool(condition_bool) = condition_value {
            return self.eval_expr_if_with_classical_condition(
                condition_bool,
                body_expr_id,
                otherwise_expr_id,
            );
        }

        // At this point the condition value is not classical, so we need to generate a branching instruction.
        // First, we pop the current block node and generate a new one which the new branches will jump to when their
        // instructions end.
        let current_block_node = self.eval_context.pop_block_node();
        let continuation_block_node_id = self.create_program_block();
        let continuation_block_node = BlockNode {
            id: continuation_block_node_id,
            successor: current_block_node.successor,
        };
        self.eval_context.push_block_node(continuation_block_node);

        // Since the if expression can represent a dynamic value, create a variable to store it if the expression is
        // non-unit.
        let if_expr = self.get_expr(if_expr_id);
        let maybe_if_expr_var = if if_expr.ty == Ty::UNIT {
            None
        } else {
            let variable_id = self.resource_manager.next_var();
            let variable_ty = map_fir_type_to_rir_type(&if_expr.ty);
            Some(rir::Variable {
                id: variable_id,
                ty: variable_ty,
            })
        };

        // Evaluate the body expression.
        let if_true_branch_control_flow =
            self.eval_expr_if_branch(body_expr_id, continuation_block_node_id, maybe_if_expr_var)?;
        let if_true_block_id = match if_true_branch_control_flow {
            BranchControlFlow::Block(block_id) => block_id,
            BranchControlFlow::Return(value) => return Ok(EvalControlFlow::Return(value)),
        };

        // Evaluate the otherwise expression (if any), and determine the block to branch to if the condition is false.
        let if_false_block_id = if let Some(otherwise_expr_id) = otherwise_expr_id {
            let if_false_branch_control_flow = self.eval_expr_if_branch(
                otherwise_expr_id,
                continuation_block_node_id,
                maybe_if_expr_var,
            )?;
            match if_false_branch_control_flow {
                BranchControlFlow::Block(block_id) => block_id,
                BranchControlFlow::Return(value) => return Ok(EvalControlFlow::Return(value)),
            }
        } else {
            continuation_block_node_id
        };

        // Finally, we insert the branch instruction.
        let condition_value_var = condition_value.unwrap_var();
        let condition_rir_var = map_eval_var_to_rir_var(condition_value_var);
        let branch_ins =
            Instruction::Branch(condition_rir_var, if_true_block_id, if_false_block_id);
        self.get_program_block_mut(current_block_node.id)
            .0
            .push(branch_ins);

        // Return the value of the if expression.
        let if_expr_value = if let Some(if_expr_var) = maybe_if_expr_var {
            Value::Var(map_rir_var_to_eval_var(if_expr_var))
        } else {
            Value::unit()
        };
        Ok(EvalControlFlow::Continue(if_expr_value))
    }

    fn eval_expr_if_branch(
        &mut self,
        branch_body_expr_id: ExprId,
        continuation_block_id: rir::BlockId,
        if_expr_var: Option<rir::Variable>,
    ) -> Result<BranchControlFlow, Error> {
        // Create the block node that corresponds to the branch body and push it as the active one.
        let block_node_id = self.create_program_block();
        let block_node = BlockNode {
            id: block_node_id,
            successor: Some(continuation_block_id),
        };
        self.eval_context.push_block_node(block_node);

        // Evaluate the branch body expression.
        let body_control = self.try_eval_expr(branch_body_expr_id)?;
        if body_control.is_return() {
            return Ok(BranchControlFlow::Return(body_control.into_value()));
        }

        // If there is a variable to save the value of the if expression to, add a store instruction.
        if let Some(if_expr_var) = if_expr_var {
            let body_operand = map_eval_value_to_rir_operand(&body_control.into_value());
            let store_ins = Instruction::Store(body_operand, if_expr_var);
            self.get_current_rir_block_mut().0.push(store_ins);
        }

        // Finally, jump to the continuation block and pop the current block node.
        let jump_ins = Instruction::Jump(continuation_block_id);
        self.get_current_rir_block_mut().0.push(jump_ins);
        let _ = self.eval_context.pop_block_node();
        Ok(BranchControlFlow::Block(block_node_id))
    }

    fn eval_expr_if_with_classical_condition(
        &mut self,
        condition_bool: bool,
        body_expr_id: ExprId,
        otherwise_expr_id: Option<ExprId>,
    ) -> Result<EvalControlFlow, Error> {
        if condition_bool {
            self.try_eval_expr(body_expr_id)
        } else if let Some(otherwise_expr_id) = otherwise_expr_id {
            self.try_eval_expr(otherwise_expr_id)
        } else {
            // The classical condition evaluated to false, but there is not otherwise block so there is nothing to
            // evaluate.
            // Return unit since it is the only possibility for if expressions with no otherwise block.
            Ok(EvalControlFlow::Continue(Value::unit()))
        }
    }

    fn eval_expr_return(&mut self, expr_id: ExprId) -> Result<EvalControlFlow, Error> {
        let control_flow = self.try_eval_expr(expr_id)?;
        Ok(EvalControlFlow::Return(control_flow.into_value()))
    }

    fn eval_expr_array(&mut self, exprs: &Vec<ExprId>) -> Result<EvalControlFlow, Error> {
        let mut values = Vec::with_capacity(exprs.len());
        for expr_id in exprs {
            let control_flow = self.try_eval_expr(*expr_id)?;
            if control_flow.is_return() {
                let expr = self.get_expr(*expr_id);
                return Err(Error::Unexpected(
                    "embedded return in array".to_string(),
                    expr.span,
                ));
            }
            values.push(control_flow.into_value());
        }
        Ok(EvalControlFlow::Continue(Value::Array(values.into())))
    }

    fn eval_expr_tuple(&mut self, exprs: &Vec<ExprId>) -> Result<EvalControlFlow, Error> {
        let mut values = Vec::with_capacity(exprs.len());
        for expr_id in exprs {
            let control_flow = self.try_eval_expr(*expr_id)?;
            if control_flow.is_return() {
                let expr = self.get_expr(*expr_id);
                return Err(Error::Unexpected(
                    "embedded return in tuple".to_string(),
                    expr.span,
                ));
            }
            values.push(control_flow.into_value());
        }
        Ok(EvalControlFlow::Continue(Value::Tuple(values.into())))
    }

    fn eval_expr_var(&mut self, res: &Res) -> Value {
        match res {
            Res::Err => panic!("resolution error"),
            Res::Item(item) => Value::Global(
                StoreItemId {
                    package: item.package.unwrap_or(self.get_current_package_id()),
                    item: item.item,
                },
                FunctorApp::default(),
            ),
            Res::Local(local_var_id) => self
                .eval_context
                .get_current_scope()
                .get_local_value(*local_var_id)
                .clone(),
        }
    }

    fn eval_expr_while(
        &mut self,
        condition_expr_id: ExprId,
        body_block_id: BlockId,
    ) -> Result<EvalControlFlow, Error> {
        // Verify assumptions.
        assert!(
            self.is_classical_expr(condition_expr_id),
            "loop conditions must be purely classical"
        );
        let body_block = self.get_block(body_block_id);
        assert_eq!(
            body_block.ty,
            Ty::UNIT,
            "the type of a loop block is expected to be Unit"
        );

        // Evaluate the block until the loop condition is false.
        let mut condition_control_flow = self.try_eval_expr(condition_expr_id)?;
        if condition_control_flow.is_return() {
            let condition_expr = self.get_expr(condition_expr_id);
            return Err(Error::Unexpected(
                "embedded return in loop condition".to_string(),
                condition_expr.span,
            ));
        }
        let mut condition_boolean = condition_control_flow.into_value().unwrap_bool();
        while condition_boolean {
            // Evaluate the loop block.
            let block_control_flow = self.try_eval_block(body_block_id)?;
            if block_control_flow.is_return() {
                return Ok(block_control_flow);
            }

            // Re-evaluate the condition now that the block evaluation is done
            condition_control_flow = self.try_eval_expr(condition_expr_id)?;
            if condition_control_flow.is_return() {
                let condition_expr = self.get_expr(condition_expr_id);
                return Err(Error::Unexpected(
                    "embedded return in loop condition".to_string(),
                    condition_expr.span,
                ));
            }
            condition_boolean = condition_control_flow.into_value().unwrap_bool();
        }

        // We have evaluated the loop so just return unit as the value of this loop expression.
        Ok(EvalControlFlow::Continue(Value::unit()))
    }

    fn eval_result_as_bool_operand(&mut self, result: val::Result) -> Operand {
        match result {
            val::Result::Id(id) => {
                // If this is a result ID, generate the instruction to read it.
                let result_operand = Operand::Literal(Literal::Result(
                    id.try_into().expect("could not convert result ID to u32"),
                ));
                let read_result_callable_id =
                    self.get_or_insert_callable(builder::read_result_decl());
                let variable_id = self.resource_manager.next_var();
                let variable_ty = rir::Ty::Boolean;
                let variable = rir::Variable {
                    id: variable_id,
                    ty: variable_ty,
                };
                let instruction = Instruction::Call(
                    read_result_callable_id,
                    vec![result_operand],
                    Some(variable),
                );
                let current_block = self.get_current_rir_block_mut();
                current_block.0.push(instruction);
                Operand::Variable(variable)
            }
            val::Result::Val(bool) => Operand::Literal(Literal::Bool(bool)),
        }
    }

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

    fn get_current_package_id(&self) -> PackageId {
        self.eval_context.get_current_scope().package_id
    }

    fn get_current_rir_block_mut(&mut self) -> &mut rir::Block {
        self.get_program_block_mut(self.eval_context.get_current_block_id())
    }

    fn get_current_scope_exec_graph(&self) -> &Rc<[ExecGraphNode]> {
        if let Some(spec_decl) = self.get_current_scope_spec_decl() {
            &spec_decl.exec_graph
        } else {
            &self.entry.exec_graph
        }
    }

    fn get_current_scope_spec_decl(&self) -> Option<&SpecDecl> {
        let current_scope = self.eval_context.get_current_scope();
        let (local_item_id, functor_app) = current_scope.callable?;
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

    fn get_expr_compute_kind(&self, expr_id: ExprId) -> ComputeKind {
        let current_package_id = self.get_current_package_id();
        let store_expr_id = StoreExprId::from((current_package_id, expr_id));
        let expr_generator_set = self.compute_properties.get_expr(store_expr_id);
        let callable_scope = self.eval_context.get_current_scope();
        expr_generator_set.generate_application_compute_kind(&callable_scope.args_value_kind)
    }

    fn get_stmt_compute_kind(&self, stmt_id: StmtId) -> ComputeKind {
        let current_package_id = self.get_current_package_id();
        let store_stmt_id = StoreStmtId::from((current_package_id, stmt_id));
        let stmt_generator_set = self.compute_properties.get_stmt(store_stmt_id);
        let callable_scope = self.eval_context.get_current_scope();
        stmt_generator_set.generate_application_compute_kind(&callable_scope.args_value_kind)
    }

    fn get_or_create_variable(&mut self, local_var_id: LocalVarId, var_ty: VarTy) -> Var {
        let current_scope = self.eval_context.get_current_scope_mut();
        let entry = current_scope.hybrid_vars.entry(local_var_id);
        let local_var_value = entry.or_insert(Value::Var({
            let var_id = self.resource_manager.next_var();
            Var {
                id: var_id.into(),
                ty: var_ty,
            }
        }));
        let Value::Var(var) = local_var_value else {
            panic!("value must be a variable");
        };
        *var
    }

    fn get_or_insert_callable(&mut self, callable: Callable) -> CallableId {
        // Check if the callable is already in the program, and if not add it.
        let callable_name = callable.name.clone();
        if let Entry::Vacant(entry) = self.callables_map.entry(callable_name.clone().into()) {
            let callable_id = self.resource_manager.next_callable();
            entry.insert(callable_id);
            self.program.callables.insert(callable_id, callable);
        }

        *self
            .callables_map
            .get(callable_name.as_str())
            .expect("callable not present")
    }

    fn get_program_block_mut(&mut self, id: rir::BlockId) -> &mut rir::Block {
        self.program
            .blocks
            .get_mut(id)
            .expect("program block does not exist")
    }

    fn is_classical_expr(&self, expr_id: ExprId) -> bool {
        let compute_kind = self.get_expr_compute_kind(expr_id);
        matches!(compute_kind, ComputeKind::Classical)
    }

    fn is_classical_stmt(&self, stmt_id: StmtId) -> bool {
        let compute_kind = self.get_stmt_compute_kind(stmt_id);
        matches!(compute_kind, ComputeKind::Classical)
    }

    fn allocate_qubit(&mut self) -> Value {
        let qubit = self.resource_manager.allocate_qubit();
        Value::Qubit(qubit)
    }

    fn measure_qubit(&mut self, measure_callable: Callable, args_value: Value) -> Value {
        // Get the qubit and result IDs to use in the qubit measure instruction.
        let qubit = args_value.unwrap_qubit();
        let qubit_value = Value::Qubit(qubit);
        let qubit_operand = map_eval_value_to_rir_operand(&qubit_value);
        let result_value = Value::Result(self.resource_manager.next_result_register());
        let result_operand = map_eval_value_to_rir_operand(&result_value);

        // Check if the callable has already been added to the program and if not do so now.
        let measure_callable_id = self.get_or_insert_callable(measure_callable);
        let args = vec![qubit_operand, result_operand];
        let instruction = Instruction::Call(measure_callable_id, args, None);
        let current_block = self.get_current_rir_block_mut();
        current_block.0.push(instruction);

        // Return the result value.
        result_value
    }

    fn release_qubit(&mut self, args_value: Value) -> Value {
        let qubit = args_value.unwrap_qubit();
        self.resource_manager.release_qubit(qubit);

        // The value of a qubit release is unit.
        Value::unit()
    }

    fn resolve_args(&self, store_pat_id: StorePatId, value: Value) -> Vec<Arg> {
        let pat = self.package_store.get_pat(store_pat_id);
        match &pat.kind {
            PatKind::Discard => vec![Arg::Discard(value)],
            PatKind::Bind(ident) => {
                let variable = Variable {
                    name: ident.name.clone(),
                    value,
                    span: ident.span,
                };
                vec![Arg::Var(ident.id, variable)]
            }
            PatKind::Tuple(pats) => {
                let values = value.unwrap_tuple();
                assert_eq!(
                    pats.len(),
                    values.len(),
                    "pattern tuple and value tuple have different arity"
                );
                let mut args = Vec::new();
                let pat_value_tuples = pats.iter().zip(values.to_vec());
                for (pat_id, value) in pat_value_tuples {
                    let mut element_args =
                        self.resolve_args((store_pat_id.package, *pat_id).into(), value);
                    args.append(&mut element_args);
                }
                args
            }
        }
    }

    fn try_eval_block(&mut self, block_id: BlockId) -> Result<EvalControlFlow, Error> {
        let block = self.get_block(block_id);
        let mut return_stmt_id = None;
        let mut last_control_flow = EvalControlFlow::Continue(Value::unit());

        // Iterate through the statements until we hit a return or reach the last statement.
        let mut stmts_iter = block.stmts.iter();
        for stmt_id in stmts_iter.by_ref() {
            last_control_flow = self.try_eval_stmt(*stmt_id)?;
            if last_control_flow.is_return() {
                return_stmt_id = Some(*stmt_id);
                break;
            }
        }

        // While we support multiple returns within a callable, disallow situations in which statements are left
        // unprocessed when we are evaluating a branch within a callable scope.
        let remaining_stmt_count = stmts_iter.count();
        let current_scope = self.eval_context.get_current_scope();
        if remaining_stmt_count > 0 && current_scope.is_currently_evaluating_branch() {
            let return_stmt =
                self.get_stmt(return_stmt_id.expect("a return statement ID must have been set"));
            Err(Error::Unexpected(
                "early return".to_string(),
                return_stmt.span,
            ))
        } else {
            Ok(last_control_flow)
        }
    }

    fn try_eval_expr(&mut self, expr_id: ExprId) -> Result<EvalControlFlow, Error> {
        // An expression is evaluated differently depending on whether it is purely classical or hybrid.
        if self.is_classical_expr(expr_id) {
            self.eval_classical_expr(expr_id)
        } else {
            self.eval_hybrid_expr(expr_id)
        }
    }

    fn try_eval_stmt(&mut self, stmt_id: StmtId) -> Result<EvalControlFlow, Error> {
        // If the statement is classical, we can just evaluate it.
        if self.is_classical_stmt(stmt_id) {
            self.eval_classical_stmt(stmt_id)
        } else {
            self.eval_hybrid_stmt(stmt_id)
        }
    }

    fn generate_output_recording_instructions(
        &mut self,
        ret_val: Value,
        ty: &Ty,
    ) -> Result<Vec<Instruction>, ()> {
        let mut instrs = Vec::new();

        match ret_val {
            Value::Result(val::Result::Val(_)) => return Err(()),

            Value::Array(vals) => self.record_array(ty, &mut instrs, &vals)?,
            Value::Tuple(vals) => self.record_tuple(ty, &mut instrs, &vals)?,
            Value::Result(res) => self.record_result(&mut instrs, res),
            Value::Var(var) => self.record_variable(ty, &mut instrs, var),
            Value::Bool(val) => self.record_bool(&mut instrs, val),
            Value::Int(val) => self.record_int(&mut instrs, val),

            Value::BigInt(_)
            | Value::Closure(_)
            | Value::Double(_)
            | Value::Global(_, _)
            | Value::Pauli(_)
            | Value::Qubit(_)
            | Value::Range(_)
            | Value::String(_) => panic!("unsupported value type in output recording"),
        }

        Ok(instrs)
    }

    fn record_int(&mut self, instrs: &mut Vec<Instruction>, val: i64) {
        let int_record_callable_id = self.get_int_record_callable();
        instrs.push(Instruction::Call(
            int_record_callable_id,
            vec![
                Operand::Literal(Literal::Integer(val)),
                Operand::Literal(Literal::Pointer),
            ],
            None,
        ));
    }

    fn record_bool(&mut self, instrs: &mut Vec<Instruction>, val: bool) {
        let bool_record_callable_id = self.get_bool_record_callable();
        instrs.push(Instruction::Call(
            bool_record_callable_id,
            vec![
                Operand::Literal(Literal::Bool(val)),
                Operand::Literal(Literal::Pointer),
            ],
            None,
        ));
    }

    fn record_variable(&mut self, ty: &Ty, instrs: &mut Vec<Instruction>, var: Var) {
        let record_callable_id = match ty {
            Ty::Prim(Prim::Bool) => self.get_bool_record_callable(),
            Ty::Prim(Prim::Int) => self.get_int_record_callable(),
            _ => panic!("unsupported variable type in output recording"),
        };
        instrs.push(Instruction::Call(
            record_callable_id,
            vec![
                Operand::Variable(map_eval_var_to_rir_var(var)),
                Operand::Literal(Literal::Pointer),
            ],
            None,
        ));
    }

    fn record_result(&mut self, instrs: &mut Vec<Instruction>, res: val::Result) {
        let result_record_callable_id = self.get_result_record_callable();
        instrs.push(Instruction::Call(
            result_record_callable_id,
            vec![
                Operand::Literal(Literal::Result(
                    res.unwrap_id()
                        .try_into()
                        .expect("result id should fit into u32"),
                )),
                Operand::Literal(Literal::Pointer),
            ],
            None,
        ));
    }

    fn record_tuple(
        &mut self,
        ty: &Ty,
        instrs: &mut Vec<Instruction>,
        vals: &Rc<[Value]>,
    ) -> Result<(), ()> {
        let Ty::Tuple(elem_tys) = ty else {
            panic!("expected tuple type for tuple value");
        };
        let tuple_record_callable_id = self.get_tuple_record_callable();
        instrs.push(Instruction::Call(
            tuple_record_callable_id,
            vec![
                Operand::Literal(Literal::Integer(
                    vals.len()
                        .try_into()
                        .expect("tuple length should fit into u32"),
                )),
                Operand::Literal(Literal::Pointer),
            ],
            None,
        ));
        for (val, elem_ty) in vals.iter().zip(elem_tys.iter()) {
            instrs.extend(self.generate_output_recording_instructions(val.clone(), elem_ty)?);
        }

        Ok(())
    }

    fn record_array(
        &mut self,
        ty: &Ty,
        instrs: &mut Vec<Instruction>,
        vals: &Rc<Vec<Value>>,
    ) -> Result<(), ()> {
        let Ty::Array(elem_ty) = ty else {
            panic!("expected array type for array value");
        };
        let array_record_callable_id = self.get_array_record_callable();
        instrs.push(Instruction::Call(
            array_record_callable_id,
            vec![
                Operand::Literal(Literal::Integer(
                    vals.len()
                        .try_into()
                        .expect("array length should fit into u32"),
                )),
                Operand::Literal(Literal::Pointer),
            ],
            None,
        ));
        for val in vals.iter() {
            instrs.extend(self.generate_output_recording_instructions(val.clone(), elem_ty)?);
        }

        Ok(())
    }

    fn get_array_record_callable(&mut self) -> CallableId {
        if let Some(id) = self.callables_map.get("__quantum__rt__array_record_output") {
            return *id;
        }

        let callable = builder::array_record_decl();
        let callable_id = self.resource_manager.next_callable();
        self.callables_map
            .insert("__quantum__rt__array_record_output".into(), callable_id);
        self.program.callables.insert(callable_id, callable);
        callable_id
    }

    fn get_tuple_record_callable(&mut self) -> CallableId {
        if let Some(id) = self.callables_map.get("__quantum__rt__tuple_record_output") {
            return *id;
        }

        let callable = builder::tuple_record_decl();
        let callable_id = self.resource_manager.next_callable();
        self.callables_map
            .insert("__quantum__rt__tuple_record_output".into(), callable_id);
        self.program.callables.insert(callable_id, callable);
        callable_id
    }

    fn get_result_record_callable(&mut self) -> CallableId {
        if let Some(id) = self
            .callables_map
            .get("__quantum__rt__result_record_output")
        {
            return *id;
        }

        let callable = builder::result_record_decl();
        let callable_id = self.resource_manager.next_callable();
        self.callables_map
            .insert("__quantum__rt__result_record_output".into(), callable_id);
        self.program.callables.insert(callable_id, callable);
        callable_id
    }

    fn get_bool_record_callable(&mut self) -> CallableId {
        if let Some(id) = self.callables_map.get("__quantum__rt__bool_record_output") {
            return *id;
        }

        let callable = builder::bool_record_decl();
        let callable_id = self.resource_manager.next_callable();
        self.callables_map
            .insert("__quantum__rt__bool_record_output".into(), callable_id);
        self.program.callables.insert(callable_id, callable);
        callable_id
    }

    fn get_int_record_callable(&mut self) -> CallableId {
        if let Some(id) = self.callables_map.get("__quantum__rt__int_record_output") {
            return *id;
        }

        let callable = builder::int_record_decl();
        let callable_id = self.resource_manager.next_callable();
        self.callables_map
            .insert("__quantum__rt__int_record_output".into(), callable_id);
        self.program.callables.insert(callable_id, callable);
        callable_id
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

fn map_eval_value_to_rir_operand(value: &Value) -> Operand {
    match value {
        Value::Bool(b) => Operand::Literal(Literal::Bool(*b)),
        Value::Double(d) => Operand::Literal(Literal::Double(*d)),
        Value::Int(i) => Operand::Literal(Literal::Integer(*i)),
        Value::Qubit(q) => Operand::Literal(Literal::Qubit(
            q.0.try_into().expect("could not convert qubit ID to u32"),
        )),
        Value::Result(r) => match r {
            val::Result::Id(id) => Operand::Literal(Literal::Result(
                (*id)
                    .try_into()
                    .expect("could not convert result ID to u32"),
            )),
            val::Result::Val(bool) => Operand::Literal(Literal::Bool(*bool)),
        },
        Value::Var(var) => Operand::Variable(map_eval_var_to_rir_var(*var)),
        _ => panic!("{value} cannot be mapped to a RIR operand"),
    }
}

fn map_eval_var_to_rir_var(var: Var) -> rir::Variable {
    rir::Variable {
        id: var.id.into(),
        ty: map_eval_var_type_to_rir_type(var.ty),
    }
}

fn map_eval_var_type_to_rir_type(var_ty: VarTy) -> rir::Ty {
    match var_ty {
        VarTy::Boolean => rir::Ty::Boolean,
        VarTy::Integer => rir::Ty::Integer,
        VarTy::Double => rir::Ty::Double,
    }
}

fn map_fir_type_to_rir_type(ty: &Ty) -> rir::Ty {
    let Ty::Prim(prim) = ty else {
        panic!("only some primitive types are supported");
    };

    match prim {
        Prim::BigInt
        | Prim::Pauli
        | Prim::Range
        | Prim::RangeFrom
        | Prim::RangeFull
        | Prim::RangeTo
        | Prim::String => panic!("{prim:?} is not a supported primitive type"),
        Prim::Bool => rir::Ty::Boolean,
        Prim::Double => rir::Ty::Double,
        Prim::Int => rir::Ty::Integer,
        Prim::Qubit => rir::Ty::Qubit,
        Prim::Result => rir::Ty::Result,
    }
}

fn map_rir_var_to_eval_var(var: rir::Variable) -> Var {
    Var {
        id: var.id.into(),
        ty: map_rir_type_to_eval_var_type(var.ty),
    }
}

fn map_rir_type_to_eval_var_type(ty: rir::Ty) -> VarTy {
    match ty {
        rir::Ty::Boolean => VarTy::Boolean,
        rir::Ty::Integer => VarTy::Integer,
        rir::Ty::Double => VarTy::Double,
        _ => panic!("cannot convert RIR type {ty} to evaluator varible type"),
    }
}

fn try_get_eval_var_type(value: &Value) -> Option<VarTy> {
    match value {
        Value::Bool(_) => Some(VarTy::Boolean),
        Value::Int(_) => Some(VarTy::Integer),
        Value::Double(_) => Some(VarTy::Double),
        Value::Var(var) => Some(var.ty),
        _ => None,
    }
}
