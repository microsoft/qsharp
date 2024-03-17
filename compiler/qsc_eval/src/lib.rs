// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

pub mod backend;
pub mod debug;
mod error;
mod intrinsic;
pub mod lower;
pub mod output;
pub mod state;
pub mod val;

use crate::val::Value;
use backend::Backend;
use debug::{map_fir_package_to_hir, CallStack, Frame};
use error::PackageSpan;
use miette::Diagnostic;
use num_bigint::BigInt;
use output::Receiver;
use qsc_data_structures::{functors::FunctorApp, index_map::IndexMap, span::Span};
use qsc_fir::fir::{
    self, BinOp, CallableImpl, CfgNode, Expr, ExprId, ExprKind, Field, Functor, Global, Lit,
    LocalItemId, LocalVarId, PackageId, PackageStoreLookup, PatId, PatKind, PrimField, Res, StmtId,
    StoreItemId, StringComponent, UnOp,
};
use qsc_fir::ty::Ty;
use rand::{rngs::StdRng, SeedableRng};
use std::ops;
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    iter,
    ops::Neg,
    rc::Rc,
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("array too large")]
    #[diagnostic(code("Qsc.Eval.ArrayTooLarge"))]
    ArrayTooLarge(#[label("this array has too many items")] PackageSpan),

    #[error("invalid array length: {0}")]
    #[diagnostic(code("Qsc.Eval.InvalidArrayLength"))]
    InvalidArrayLength(i64, #[label("cannot be used as a length")] PackageSpan),

    #[error("division by zero")]
    #[diagnostic(code("Qsc.Eval.DivZero"))]
    DivZero(#[label("cannot divide by zero")] PackageSpan),

    #[error("empty range")]
    #[diagnostic(code("Qsc.Eval.EmptyRange"))]
    EmptyRange(#[label("the range cannot be empty")] PackageSpan),

    #[error("value cannot be used as an index: {0}")]
    #[diagnostic(code("Qsc.Eval.InvalidIndex"))]
    InvalidIndex(i64, #[label("invalid index")] PackageSpan),

    #[error("integer too large for operation")]
    #[diagnostic(code("Qsc.Eval.IntTooLarge"))]
    IntTooLarge(i64, #[label("this value is too large")] PackageSpan),

    #[error("index out of range: {0}")]
    #[diagnostic(code("Qsc.Eval.IndexOutOfRange"))]
    IndexOutOfRange(i64, #[label("out of range")] PackageSpan),

    #[error("intrinsic callable `{0}` failed: {1}")]
    #[diagnostic(code("Qsc.Eval.IntrinsicFail"))]
    IntrinsicFail(String, String, #[label] PackageSpan),

    #[error("invalid rotation angle: {0}")]
    #[diagnostic(code("Qsc.Eval.InvalidRotationAngle"))]
    InvalidRotationAngle(f64, #[label("invalid rotation angle")] PackageSpan),

    #[error("negative integers cannot be used here: {0}")]
    #[diagnostic(code("Qsc.Eval.InvalidNegativeInt"))]
    InvalidNegativeInt(i64, #[label("invalid negative integer")] PackageSpan),

    #[error("output failure")]
    #[diagnostic(code("Qsc.Eval.OutputFail"))]
    OutputFail(#[label("failed to generate output")] PackageSpan),

    #[error("qubits in invocation are not unique")]
    #[diagnostic(code("Qsc.Eval.QubitUniqueness"))]
    QubitUniqueness(#[label] PackageSpan),

    #[error("qubits are not separable")]
    #[diagnostic(help("subset of qubits provided as arguments must not be entangled with any qubits outside of the subset"))]
    #[diagnostic(code("Qsc.Eval.QubitsNotSeparable"))]
    QubitsNotSeparable(#[label] PackageSpan),

    #[error("range with step size of zero")]
    #[diagnostic(code("Qsc.Eval.RangeStepZero"))]
    RangeStepZero(#[label("invalid range")] PackageSpan),

    #[error("Qubit{0} released while not in |0⟩ state")]
    #[diagnostic(help("qubits should be returned to the |0⟩ state before being released to satisfy the assumption that allocated qubits start in the |0⟩ state"))]
    #[diagnostic(code("Qsc.Eval.ReleasedQubitNotZero"))]
    ReleasedQubitNotZero(usize, #[label("Qubit{0}")] PackageSpan),

    #[error("name is not bound")]
    #[diagnostic(code("Qsc.Eval.UnboundName"))]
    UnboundName(#[label] PackageSpan),

    #[error("unknown intrinsic `{0}`")]
    #[diagnostic(code("Qsc.Eval.UnknownIntrinsic"))]
    UnknownIntrinsic(
        String,
        #[label("callable has no implementation")] PackageSpan,
    ),

    #[error("unsupported return type for intrinsic `{0}`")]
    #[diagnostic(help("intrinsic callable return type should be `Unit`"))]
    #[diagnostic(code("Qsc.Eval.UnsupportedIntrinsicType"))]
    UnsupportedIntrinsicType(String, #[label] PackageSpan),

    #[error("program failed: {0}")]
    #[diagnostic(code("Qsc.Eval.UserFail"))]
    UserFail(String, #[label("explicit fail")] PackageSpan),
}

impl Error {
    #[must_use]
    pub fn span(&self) -> &PackageSpan {
        match self {
            Error::ArrayTooLarge(span)
            | Error::DivZero(span)
            | Error::EmptyRange(span)
            | Error::IndexOutOfRange(_, span)
            | Error::InvalidIndex(_, span)
            | Error::IntrinsicFail(_, _, span)
            | Error::IntTooLarge(_, span)
            | Error::InvalidRotationAngle(_, span)
            | Error::InvalidNegativeInt(_, span)
            | Error::OutputFail(span)
            | Error::QubitUniqueness(span)
            | Error::QubitsNotSeparable(span)
            | Error::RangeStepZero(span)
            | Error::ReleasedQubitNotZero(_, span)
            | Error::UnboundName(span)
            | Error::UnknownIntrinsic(_, span)
            | Error::UnsupportedIntrinsicType(_, span)
            | Error::UserFail(_, span)
            | Error::InvalidArrayLength(_, span) => span,
        }
    }
}

/// A specialization that may be implemented for an operation.
enum Spec {
    /// The default specialization.
    Body,
    /// The adjoint specialization.
    Adj,
    /// The controlled specialization.
    Ctl,
    /// The controlled adjoint specialization.
    CtlAdj,
}

impl Display for Spec {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Spec::Body => f.write_str("body"),
            Spec::Adj => f.write_str("adjoint"),
            Spec::Ctl => f.write_str("controlled"),
            Spec::CtlAdj => f.write_str("controlled adjoint"),
        }
    }
}

/// Utility function to identify a subset of a control flow graph corresponding to a given
/// range.
#[must_use]
pub fn sub_cfg(cfg: &Rc<[CfgNode]>, range: ops::Range<usize>) -> Rc<[CfgNode]> {
    let start: u32 = range
        .start
        .try_into()
        .expect("cfg ranges should fit into u32");
    cfg[range]
        .iter()
        .map(|node| match node {
            CfgNode::Jump(idx) => CfgNode::Jump(idx - start),
            CfgNode::JumpIf(idx) => CfgNode::JumpIf(idx - start),
            CfgNode::JumpIfNot(idx) => CfgNode::JumpIfNot(idx - start),
            _ => *node,
        })
        .collect::<Vec<_>>()
        .into()
}

/// Evaluates the given code with the given context.
/// # Errors
/// Returns the first error encountered during execution.
/// # Panics
/// On internal error where no result is returned.
pub fn eval(
    package: PackageId,
    seed: Option<u64>,
    cfg: Rc<[CfgNode]>,
    globals: &impl PackageStoreLookup,
    env: &mut Env,
    sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
    receiver: &mut impl Receiver,
) -> Result<Value, (Error, Vec<Frame>)> {
    let mut state = State::new(package, cfg, seed);
    let res = state.eval(globals, env, sim, receiver, &[], StepAction::Continue)?;
    let StepResult::Return(value) = res else {
        panic!("eval should always return a value");
    };
    Ok(value)
}

/// The type of step action to take during evaluation
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StepAction {
    Next,
    In,
    Out,
    Continue,
}

// The result of an evaluation step.
#[derive(Clone, Debug)]
pub enum StepResult {
    BreakpointHit(StmtId),
    Next,
    StepIn,
    StepOut,
    Return(Value),
}

trait AsIndex {
    type Output;

    fn as_index(&self, index_source: PackageSpan) -> Self::Output;
}

impl AsIndex for i64 {
    type Output = Result<usize, Error>;

    fn as_index(&self, index_source: PackageSpan) -> Self::Output {
        match (*self).try_into() {
            Ok(index) => Ok(index),
            Err(_) => Err(Error::InvalidIndex(*self, index_source)),
        }
    }
}

#[derive(Debug, Clone)]
struct Variable {
    name: Rc<str>,
    value: Value,
    span: Span,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub value: Value,
    pub name: Rc<str>,
    pub type_name: String,
    pub span: Span,
}

struct Range {
    step: i64,
    end: i64,
    curr: i64,
}

impl Iterator for Range {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr;
        self.curr += self.step;
        if (self.step > 0 && curr <= self.end) || (self.step < 0 && curr >= self.end) {
            Some(curr)
        } else {
            None
        }
    }
}

impl Range {
    fn new(start: i64, step: i64, end: i64) -> Self {
        Range {
            step,
            end,
            curr: start,
        }
    }
}

pub struct Env(Vec<Scope>);

impl Env {
    #[must_use]
    fn get(&self, id: LocalVarId) -> Option<&Variable> {
        self.0.iter().rev().find_map(|scope| scope.bindings.get(id))
    }

    fn get_mut(&mut self, id: LocalVarId) -> Option<&mut Variable> {
        self.0
            .iter_mut()
            .rev()
            .find_map(|scope| scope.bindings.get_mut(id))
    }

    fn push_scope(&mut self, frame_id: usize) {
        let scope = Scope {
            frame_id,
            ..Default::default()
        };
        self.0.push(scope);
    }

    fn leave_scope(&mut self) {
        // Only pop the scope if there is more than one scope in the stack,
        // because the global/top-level scope cannot be exited.
        if self.0.len() > 1 {
            self.0
                .pop()
                .expect("scope should have more than one entry.");
        }
    }

    #[must_use]
    pub fn get_variables_in_top_frame(&self) -> Vec<VariableInfo> {
        if let Some(scope) = self.0.last() {
            self.get_variables_in_frame(scope.frame_id)
        } else {
            vec![]
        }
    }

    #[must_use]
    pub fn get_variables_in_frame(&self, frame_id: usize) -> Vec<VariableInfo> {
        let candidate_scopes: Vec<_> = self
            .0
            .iter()
            .filter(|scope| scope.frame_id == frame_id)
            .map(|scope| scope.bindings.iter())
            .collect();

        let variables_by_scope: Vec<Vec<VariableInfo>> = candidate_scopes
            .into_iter()
            .map(|bindings| {
                bindings
                    .map(|(_, var)| VariableInfo {
                        name: var.name.clone(),
                        type_name: var.value.type_name().to_string(),
                        value: var.value.clone(),
                        span: var.span,
                    })
                    .collect()
            })
            .collect();
        variables_by_scope.into_iter().flatten().collect::<Vec<_>>()
    }
}

#[derive(Default)]
struct Scope {
    bindings: IndexMap<LocalVarId, Variable>,
    frame_id: usize,
}

impl Default for Env {
    #[must_use]
    fn default() -> Self {
        Self(vec![Scope::default()])
    }
}

pub struct State {
    cfg_stack: Vec<Rc<[CfgNode]>>,
    idx: u32,
    idx_stack: Vec<u32>,
    curr_val: Option<Value>,
    val_stack: Vec<Vec<Value>>,
    package: PackageId,
    call_stack: CallStack,
    current_span: Span,
    rng: RefCell<StdRng>,
}

impl State {
    #[must_use]
    pub fn new(package: PackageId, cfg: Rc<[CfgNode]>, classical_seed: Option<u64>) -> Self {
        let rng = match classical_seed {
            Some(seed) => RefCell::new(StdRng::seed_from_u64(seed)),
            None => RefCell::new(StdRng::from_entropy()),
        };
        Self {
            cfg_stack: vec![cfg],
            idx: 0,
            idx_stack: Vec::new(),
            curr_val: None,
            val_stack: vec![Vec::new()],
            package,
            call_stack: CallStack::default(),
            current_span: Span::default(),
            rng,
        }
    }

    fn push_frame(&mut self, cfg: Rc<[CfgNode]>, id: StoreItemId, functor: FunctorApp) {
        self.call_stack.push_frame(Frame {
            span: self.current_span,
            id,
            caller: self.package,
            functor,
        });
        self.cfg_stack.push(cfg);
        self.val_stack.push(Vec::new());
        self.idx_stack.push(self.idx);
        self.idx = 0;
        self.package = id.package;
    }

    fn leave_frame(&mut self) {
        if let Some(frame) = self.call_stack.pop_frame() {
            self.package = frame.caller;
            self.val_stack.pop();
            self.idx = self
                .idx_stack
                .pop()
                .expect("should have at least one index");
        }
        self.cfg_stack.pop();
    }

    fn push_scope(&mut self, env: &mut Env) {
        env.push_scope(self.call_stack.len());
    }

    fn take_curr_val(&mut self) -> Value {
        self.curr_val.take().expect("value should be present")
    }

    fn set_curr_val(&mut self, val: Value) {
        self.curr_val = Some(val);
    }

    fn pop_val(&mut self) -> Value {
        self.val_stack
            .last_mut()
            .expect("should have at least one value frame")
            .pop()
            .expect("value should be present")
    }

    fn pop_vals(&mut self, len: usize) -> Vec<Value> {
        let last = self
            .val_stack
            .last_mut()
            .expect("should have at least one value frame");
        last.drain(last.len() - len..).collect()
    }

    fn push_val(&mut self) {
        let val = self.take_curr_val();
        self.val_stack
            .last_mut()
            .expect("should have at least one value frame")
            .push(val);
    }

    #[must_use]
    pub fn get_stack_frames(&self) -> Vec<Frame> {
        let mut frames = self.call_stack.clone().into_frames();

        let mut span = self.current_span;
        for frame in frames.iter_mut().rev() {
            std::mem::swap(&mut frame.span, &mut span);
        }
        frames
    }

    /// # Errors
    /// Returns the first error encountered during execution.
    /// # Panics
    /// When returning a value in the middle of execution.
    pub fn eval(
        &mut self,
        globals: &impl PackageStoreLookup,
        env: &mut Env,
        sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
        out: &mut impl Receiver,
        breakpoints: &[StmtId],
        step: StepAction,
    ) -> Result<StepResult, (Error, Vec<Frame>)> {
        let current_frame = self.call_stack.len();

        while !self.cfg_stack.is_empty() {
            let cfg = self
                .cfg_stack
                .last()
                .expect("should have at least one stack frame");
            let res = match cfg.get(self.idx as usize) {
                Some(CfgNode::Bind(pat)) => {
                    self.idx += 1;
                    self.eval_bind(env, globals, *pat);
                    continue;
                }
                Some(CfgNode::Expr(expr)) => {
                    self.idx += 1;
                    self.eval_expr(env, sim, globals, out, *expr)
                        .map_err(|e| (e, self.get_stack_frames()))?;
                    continue;
                }
                Some(CfgNode::Stmt(stmt)) => {
                    self.idx += 1;
                    self.current_span = globals.get_stmt((self.package, *stmt).into()).span;

                    if let Some(bp) = breakpoints.iter().find(|&bp| *bp == *stmt) {
                        StepResult::BreakpointHit(*bp)
                    } else {
                        if self.current_span == Span::default() {
                            // if there is no span, we are in generated code, so we should skip
                            continue;
                        }
                        // no breakpoint, but we may stop here
                        if step == StepAction::In {
                            StepResult::StepIn
                        } else if step == StepAction::Next && current_frame >= self.call_stack.len()
                        {
                            StepResult::Next
                        } else if step == StepAction::Out && current_frame > self.call_stack.len() {
                            StepResult::StepOut
                        } else {
                            continue;
                        }
                    }
                }
                Some(CfgNode::Jump(idx)) => {
                    self.idx = *idx;
                    continue;
                }
                Some(CfgNode::JumpIf(idx)) => {
                    let cond = self.curr_val == Some(Value::Bool(true));
                    if cond {
                        self.idx = *idx;
                    } else {
                        self.idx += 1;
                    }
                    continue;
                }
                Some(CfgNode::JumpIfNot(idx)) => {
                    let cond = self.curr_val == Some(Value::Bool(true));
                    if cond {
                        self.idx += 1;
                    } else {
                        self.idx = *idx;
                    }
                    continue;
                }
                Some(CfgNode::Store) => {
                    self.push_val();
                    self.idx += 1;
                    continue;
                }
                Some(CfgNode::Unit) => {
                    self.idx += 1;
                    self.set_curr_val(Value::unit());
                    continue;
                }
                Some(CfgNode::Ret) => {
                    self.leave_frame();
                    env.leave_scope();
                    continue;
                }
                None => {
                    // We have reached the end of the current cfg without reaching an explicit return node,
                    // usually indicating the partial execution of a single sub-expression.
                    // This means we should leave the current frame but not the current environment scope,
                    // so bound variables are still accessible after completion.
                    self.leave_frame();
                    assert!(self.cfg_stack.is_empty());
                    continue;
                }
            };

            if let StepResult::Return(_) = res {
                panic!("unexpected return");
            }

            return Ok(res);
        }

        Ok(StepResult::Return(self.get_result()))
    }

    pub fn get_result(&mut self) -> Value {
        // Some executions don't have any statements to execute,
        // such as a fragment that has only item definitions.
        // In that case, the values are empty and the result is unit.
        self.curr_val.take().unwrap_or_else(Value::unit)
    }

    #[allow(clippy::similar_names)]
    fn eval_expr(
        &mut self,
        env: &mut Env,
        sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
        globals: &impl PackageStoreLookup,
        out: &mut impl Receiver,
        expr: ExprId,
    ) -> Result<(), Error> {
        let expr = globals.get_expr((self.package, expr).into());
        self.current_span = expr.span;

        match &expr.kind {
            ExprKind::Array(arr) => self.eval_arr(arr.len()),
            ExprKind::ArrayLit(arr) => self.eval_arr_lit(arr, globals),
            ExprKind::ArrayRepeat(..) => self.eval_arr_repeat(expr.span)?,
            ExprKind::Assign(lhs, _) => self.eval_assign(env, globals, *lhs)?,
            ExprKind::AssignOp(op, lhs, rhs) => {
                let rhs_span = globals.get_expr((self.package, *rhs).into()).span;
                let (is_array, is_unique) =
                    is_updatable_in_place(env, globals.get_expr((self.package, *lhs).into()));
                if is_array {
                    if is_unique {
                        self.eval_array_append_in_place(env, globals, *lhs)?;
                        return Ok(());
                    }
                    let rhs_val = self.take_curr_val();
                    self.eval_expr(env, sim, globals, out, *lhs)?;
                    self.push_val();
                    self.set_curr_val(rhs_val);
                }
                self.eval_binop(*op, rhs_span)?;
                self.eval_assign(env, globals, *lhs)?;
            }
            ExprKind::AssignField(record, field, _) => {
                self.eval_update_field(field.clone());
                self.eval_assign(env, globals, *record)?;
            }
            ExprKind::AssignIndex(lhs, mid, _) => {
                let mid_span = globals.get_expr((self.package, *mid).into()).span;
                let (_, is_unique) =
                    is_updatable_in_place(env, globals.get_expr((self.package, *lhs).into()));
                if is_unique {
                    self.eval_update_index_in_place(env, globals, *lhs, mid_span)?;
                    return Ok(());
                }
                self.push_val();
                self.eval_expr(env, sim, globals, out, *lhs)?;
                self.eval_update_index(mid_span)?;
                self.eval_assign(env, globals, *lhs)?;
            }
            ExprKind::BinOp(op, _, rhs) => {
                let rhs_span = globals.get_expr((self.package, *rhs).into()).span;
                self.eval_binop(*op, rhs_span)?;
            }
            ExprKind::Block(..) => panic!("block expr should be handled by control flow"),
            ExprKind::Call(callee_expr, args_expr) => {
                let callable_span = globals.get_expr((self.package, *callee_expr).into()).span;
                let args_span = globals.get_expr((self.package, *args_expr).into()).span;
                self.eval_call(env, sim, globals, callable_span, args_span, out)?;
            }
            ExprKind::Closure(args, callable) => {
                let closure = resolve_closure(env, self.package, expr.span, args, *callable)?;
                self.set_curr_val(closure);
            }
            ExprKind::Fail(..) => {
                return Err(Error::UserFail(
                    self.take_curr_val().unwrap_string().to_string(),
                    self.to_global_span(expr.span),
                ));
            }
            ExprKind::Field(_, field) => self.eval_field(field.clone()),
            ExprKind::Hole => panic!("hole expr should be disallowed by passes"),
            ExprKind::If(..) => {
                panic!("if expr should be handled by control flow")
            }
            ExprKind::Index(_, rhs) => {
                let rhs_span = globals.get_expr((self.package, *rhs).into()).span;
                self.eval_index(rhs_span)?;
            }
            ExprKind::Lit(lit) => {
                self.set_curr_val(lit_to_val(lit));
            }
            ExprKind::Range(start, step, end) => {
                self.eval_range(start.is_some(), step.is_some(), end.is_some());
            }
            ExprKind::Return(..) => panic!("return expr should be handled by control flow"),
            ExprKind::String(components) => self.collect_string(components),
            ExprKind::UpdateIndex(_, mid, _) => {
                let mid_span = globals.get_expr((self.package, *mid).into()).span;
                self.eval_update_index(mid_span)?;
            }
            ExprKind::Tuple(tup) => self.eval_tup(tup.len()),
            ExprKind::UnOp(op, _) => self.eval_unop(*op),
            ExprKind::UpdateField(_, field, _) => {
                self.eval_update_field(field.clone());
            }
            ExprKind::Var(res, _) => {
                self.set_curr_val(resolve_binding(env, self.package, *res, expr.span)?);
            }
            ExprKind::While(..) => {
                panic!("while expr should be handled by control flow")
            }
        }

        Ok(())
    }

    fn collect_string(&mut self, components: &[StringComponent]) {
        if let [StringComponent::Lit(str)] = components {
            self.set_curr_val(Value::String(Rc::clone(str)));
            return;
        }

        let mut string = String::new();
        for component in components.iter().rev() {
            match component {
                StringComponent::Expr(..) => {
                    let expr_str = format!("{}", self.pop_val());
                    string.insert_str(0, &expr_str);
                }
                StringComponent::Lit(lit) => {
                    string.insert_str(0, lit);
                }
            }
        }
        self.set_curr_val(Value::String(Rc::from(string)));
    }

    fn eval_arr(&mut self, len: usize) {
        let arr = self.pop_vals(len);
        self.set_curr_val(Value::Array(arr.into()));
    }

    fn eval_arr_lit(&mut self, arr: &Vec<ExprId>, globals: &impl PackageStoreLookup) {
        let mut new_arr: Rc<Vec<Value>> = Rc::new(Vec::with_capacity(arr.len()));
        for id in arr {
            let ExprKind::Lit(lit) = &globals.get_expr((self.package, *id).into()).kind else {
                panic!("expr kind should be lit")
            };
            Rc::get_mut(&mut new_arr)
                .expect("array should be uniquely referenced")
                .push(lit_to_val(lit));
        }
        self.set_curr_val(Value::Array(new_arr));
    }

    fn eval_array_append_in_place(
        &mut self,
        env: &mut Env,
        globals: &impl PackageStoreLookup,
        lhs: ExprId,
    ) -> Result<(), Error> {
        let lhs = globals.get_expr((self.package, lhs).into());
        let rhs = self.take_curr_val();
        match (&lhs.kind, rhs) {
            (&ExprKind::Var(Res::Local(id), _), rhs) => match env.get_mut(id) {
                Some(var) => {
                    var.value.append_array(rhs);
                }
                None => return Err(Error::UnboundName(self.to_global_span(lhs.span))),
            },
            _ => unreachable!("unassignable array update pattern should be disallowed by compiler"),
        }
        Ok(())
    }

    fn eval_arr_repeat(&mut self, span: Span) -> Result<(), Error> {
        let size_val = self.take_curr_val().unwrap_int();
        let item_val = self.pop_val();
        let s = match size_val.try_into() {
            Ok(i) => Ok(i),
            Err(_) => Err(Error::InvalidArrayLength(
                size_val,
                self.to_global_span(span),
            )),
        }?;
        self.set_curr_val(Value::Array(vec![item_val; s].into()));
        Ok(())
    }

    fn eval_assign(
        &mut self,
        env: &mut Env,
        globals: &impl PackageStoreLookup,
        lhs: ExprId,
    ) -> Result<(), Error> {
        let rhs = self.take_curr_val();
        self.update_binding(env, globals, lhs, rhs)
    }

    fn eval_bind(&mut self, env: &mut Env, globals: &impl PackageStoreLookup, pat: PatId) {
        let val = self.take_curr_val();
        self.bind_value(env, globals, pat, val);
    }

    fn eval_binop(&mut self, op: BinOp, span: Span) -> Result<(), Error> {
        match op {
            BinOp::Add => self.eval_binop_simple(eval_binop_add),
            BinOp::AndB => self.eval_binop_simple(eval_binop_andb),
            BinOp::Div => self.eval_binop_with_error(span, eval_binop_div)?,
            BinOp::Eq => {
                let rhs_val = self.take_curr_val();
                let lhs_val = self.pop_val();
                self.set_curr_val(Value::Bool(lhs_val == rhs_val));
            }
            BinOp::Exp => self.eval_binop_with_error(span, eval_binop_exp)?,
            BinOp::Gt => self.eval_binop_simple(eval_binop_gt),
            BinOp::Gte => self.eval_binop_simple(eval_binop_gte),
            BinOp::Lt => self.eval_binop_simple(eval_binop_lt),
            BinOp::Lte => self.eval_binop_simple(eval_binop_lte),
            BinOp::Mod => self.eval_binop_with_error(span, eval_binop_mod)?,
            BinOp::Mul => self.eval_binop_simple(eval_binop_mul),
            BinOp::Neq => {
                let rhs_val = self.take_curr_val();
                let lhs_val = self.pop_val();
                self.set_curr_val(Value::Bool(lhs_val != rhs_val));
            }
            BinOp::OrB => self.eval_binop_simple(eval_binop_orb),
            BinOp::Shl => self.eval_binop_with_error(span, eval_binop_shl)?,
            BinOp::Shr => self.eval_binop_with_error(span, eval_binop_shr)?,
            BinOp::Sub => self.eval_binop_simple(eval_binop_sub),
            BinOp::XorB => self.eval_binop_simple(eval_binop_xorb),

            // Logical operators should be handled by control flow
            BinOp::AndL | BinOp::OrL => {}
        }
        Ok(())
    }

    fn eval_binop_simple(&mut self, binop_func: impl FnOnce(Value, Value) -> Value) {
        let rhs_val = self.take_curr_val();
        let lhs_val = self.pop_val();
        self.set_curr_val(binop_func(lhs_val, rhs_val));
    }

    fn eval_binop_with_error(
        &mut self,
        span: Span,
        binop_func: impl FnOnce(Value, Value, PackageSpan) -> Result<Value, Error>,
    ) -> Result<(), Error> {
        let span = self.to_global_span(span);
        let rhs_val = self.take_curr_val();
        let lhs_val = self.pop_val();
        self.set_curr_val(binop_func(lhs_val, rhs_val, span)?);
        Ok(())
    }

    fn eval_call(
        &mut self,
        env: &mut Env,
        sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
        globals: &impl PackageStoreLookup,
        callable_span: Span,
        arg_span: Span,
        out: &mut impl Receiver,
    ) -> Result<(), Error> {
        let arg = self.take_curr_val();
        let (callee_id, functor, fixed_args) = match self.pop_val() {
            Value::Closure(inner) => (inner.id, inner.functor, Some(inner.fixed_args)),
            Value::Global(id, functor) => (id, functor, None),
            _ => panic!("value is not callable"),
        };

        let arg_span = self.to_global_span(arg_span);

        let callee = match globals.get_global(callee_id) {
            Some(Global::Callable(callable)) => callable,
            Some(Global::Udt) => {
                self.set_curr_val(arg);
                return Ok(());
            }
            None => return Err(Error::UnboundName(self.to_global_span(callable_span))),
        };

        let callee_span = self.to_global_span(callee.span);

        let spec = spec_from_functor_app(functor);
        match &callee.implementation {
            CallableImpl::Intrinsic => {
                self.push_frame(Vec::new().into(), callee_id, functor);

                let name = &callee.name.name;
                let val = intrinsic::call(
                    name,
                    callee_span,
                    arg,
                    arg_span,
                    sim,
                    &mut self.rng.borrow_mut(),
                    out,
                )?;
                if val == Value::unit() && callee.output != Ty::UNIT {
                    return Err(Error::UnsupportedIntrinsicType(
                        callee.name.name.to_string(),
                        callee_span,
                    ));
                }
                self.set_curr_val(val);
                self.leave_frame();
                Ok(())
            }
            CallableImpl::Spec(specialized_implementation) => {
                let spec_decl = match spec {
                    Spec::Body => Some(&specialized_implementation.body),
                    Spec::Adj => specialized_implementation.adj.as_ref(),
                    Spec::Ctl => specialized_implementation.ctl.as_ref(),
                    Spec::CtlAdj => specialized_implementation.ctl_adj.as_ref(),
                }
                .expect("missing specialization should be a compilation error");
                self.push_frame(spec_decl.cfg.clone(), callee_id, functor);
                self.push_scope(env);

                self.bind_args_for_spec(
                    env,
                    globals,
                    callee.input,
                    spec_decl.input,
                    arg,
                    functor.controlled,
                    fixed_args,
                );
                Ok(())
            }
        }
    }

    fn eval_field(&mut self, field: Field) {
        let record = self.take_curr_val();
        let val = match (record, field) {
            (Value::Range(inner), Field::Prim(PrimField::Start)) => Value::Int(
                inner
                    .start
                    .expect("range access should be validated by compiler"),
            ),
            (Value::Range(inner), Field::Prim(PrimField::Step)) => Value::Int(inner.step),
            (Value::Range(inner), Field::Prim(PrimField::End)) => Value::Int(
                inner
                    .end
                    .expect("range access should be validated by compiler"),
            ),
            (record, Field::Path(path)) => {
                follow_field_path(record, &path.indices).expect("field path should be valid")
            }
            _ => panic!("invalid field access"),
        };
        self.set_curr_val(val);
    }

    fn eval_index(&mut self, span: Span) -> Result<(), Error> {
        let index_val = self.take_curr_val();
        let arr = self.pop_val().unwrap_array();
        match &index_val {
            Value::Int(i) => self.set_curr_val(index_array(&arr, *i, self.to_global_span(span))?),
            Value::Range(inner) => {
                self.set_curr_val(slice_array(
                    &arr,
                    inner.start,
                    inner.step,
                    inner.end,
                    self.to_global_span(span),
                )?);
            }
            _ => panic!("array should only be indexed by Int or Range"),
        }
        Ok(())
    }

    fn eval_range(&mut self, has_start: bool, has_step: bool, has_end: bool) {
        let end = if has_end {
            Some(self.take_curr_val().unwrap_int())
        } else {
            None
        };
        let step = if has_step {
            self.pop_val().unwrap_int()
        } else {
            val::DEFAULT_RANGE_STEP
        };
        let start = if has_start {
            Some(self.pop_val().unwrap_int())
        } else {
            None
        };
        self.set_curr_val(Value::Range(val::Range { start, step, end }.into()));
    }

    fn eval_update_index(&mut self, span: Span) -> Result<(), Error> {
        let values = self.take_curr_val().unwrap_array();
        let update = self.pop_val();
        let index = self.pop_val();
        let span = self.to_global_span(span);
        match index {
            Value::Int(index) => self.eval_update_index_single(&values, index, update, span),
            Value::Range(inner) => self.eval_update_index_range(
                &values,
                inner.start,
                inner.step,
                inner.end,
                update,
                span,
            ),
            _ => unreachable!("array should only be indexed by Int or Range"),
        }
    }

    fn eval_update_index_single(
        &mut self,
        values: &[Value],
        index: i64,
        update: Value,
        span: PackageSpan,
    ) -> Result<(), Error> {
        if index < 0 {
            return Err(Error::InvalidNegativeInt(index, span));
        }
        let i = index.as_index(span)?;
        let mut values = values.to_vec();
        match values.get_mut(i) {
            Some(value) => {
                *value = update;
            }
            None => return Err(Error::IndexOutOfRange(index, span)),
        }
        self.set_curr_val(Value::Array(values.into()));
        Ok(())
    }

    fn eval_update_index_range(
        &mut self,
        values: &[Value],
        start: Option<i64>,
        step: i64,
        end: Option<i64>,
        update: Value,
        span: PackageSpan,
    ) -> Result<(), Error> {
        let range = make_range(values, start, step, end, span)?;
        let mut values = values.to_vec();
        let update = update.unwrap_array();
        for (idx, update) in range.into_iter().zip(update.iter()) {
            let i = idx.as_index(span)?;
            match values.get_mut(i) {
                Some(value) => {
                    *value = update.clone();
                }
                None => return Err(Error::IndexOutOfRange(idx, span)),
            }
        }
        self.set_curr_val(Value::Array(values.into()));
        Ok(())
    }

    fn eval_update_index_in_place(
        &mut self,
        env: &mut Env,
        globals: &impl PackageStoreLookup,
        lhs: ExprId,
        span: Span,
    ) -> Result<(), Error> {
        let update = self.take_curr_val();
        let index = self.pop_val();
        let span = self.to_global_span(span);
        match index {
            Value::Int(index) => {
                if index < 0 {
                    return Err(Error::InvalidNegativeInt(index, span));
                }
                let i = index.as_index(span)?;
                self.update_array_index_single(env, globals, lhs, span, i, update)
            }
            range @ Value::Range(..) => {
                self.update_array_index_range(env, globals, lhs, span, &range, update)
            }
            _ => unreachable!("array should only be indexed by Int or Range"),
        }
    }

    fn eval_tup(&mut self, len: usize) {
        let tup = self.pop_vals(len);
        self.set_curr_val(Value::Tuple(tup.into()));
    }

    fn eval_unop(&mut self, op: UnOp) {
        let val = self.take_curr_val();
        match op {
            UnOp::Functor(functor) => match val {
                Value::Closure(inner) => {
                    self.set_curr_val(Value::Closure(
                        val::Closure {
                            functor: update_functor_app(functor, inner.functor),
                            ..*inner
                        }
                        .into(),
                    ));
                }
                Value::Global(id, app) => {
                    self.set_curr_val(Value::Global(id, update_functor_app(functor, app)));
                }
                _ => panic!("value should be callable"),
            },
            UnOp::Neg => match val {
                Value::BigInt(v) => self.set_curr_val(Value::BigInt(v.neg())),
                Value::Double(v) => self.set_curr_val(Value::Double(v.neg())),
                Value::Int(v) => self.set_curr_val(Value::Int(v.wrapping_neg())),
                _ => panic!("value should be number"),
            },
            UnOp::NotB => match val {
                Value::Int(v) => self.set_curr_val(Value::Int(!v)),
                Value::BigInt(v) => self.set_curr_val(Value::BigInt(!v)),
                _ => panic!("value should be Int or BigInt"),
            },
            UnOp::NotL => match val {
                Value::Bool(b) => self.set_curr_val(Value::Bool(!b)),
                _ => panic!("value should be bool"),
            },
            UnOp::Pos => match val {
                Value::BigInt(_) | Value::Int(_) | Value::Double(_) => self.set_curr_val(val),
                _ => panic!("value should be number"),
            },
            UnOp::Unwrap => self.set_curr_val(val),
        }
    }

    fn eval_update_field(&mut self, field: Field) {
        let record = self.take_curr_val();
        let value = self.pop_val();
        let update = match (record, field) {
            (Value::Range(mut inner), Field::Prim(PrimField::Start)) => {
                inner.start = Some(value.unwrap_int());
                Value::Range(inner)
            }
            (Value::Range(mut inner), Field::Prim(PrimField::Step)) => {
                inner.step = value.unwrap_int();
                Value::Range(inner)
            }
            (Value::Range(mut inner), Field::Prim(PrimField::End)) => {
                inner.end = Some(value.unwrap_int());
                Value::Range(inner)
            }
            (record, Field::Path(path)) => update_field_path(&record, &path.indices, &value)
                .expect("field path should be valid"),
            _ => panic!("invalid field access"),
        };
        self.set_curr_val(update);
    }

    fn bind_value(&self, env: &mut Env, globals: &impl PackageStoreLookup, pat: PatId, val: Value) {
        let pat = globals.get_pat((self.package, pat).into());
        match &pat.kind {
            PatKind::Bind(variable) => {
                let scope = env.0.last_mut().expect("binding should have a scope");
                scope.bindings.insert(
                    variable.id,
                    Variable {
                        name: variable.name.clone(),
                        value: val,
                        span: variable.span,
                    },
                );
            }
            PatKind::Discard => {}
            PatKind::Tuple(tup) => {
                let val_tup = val.unwrap_tuple();
                for (pat, val) in tup.iter().zip(val_tup.iter()) {
                    self.bind_value(env, globals, *pat, val.clone());
                }
            }
        }
    }

    #[allow(clippy::similar_names)]
    fn update_binding(
        &self,
        env: &mut Env,
        globals: &impl PackageStoreLookup,
        lhs: ExprId,
        rhs: Value,
    ) -> Result<(), Error> {
        let lhs = globals.get_expr((self.package, lhs).into());
        match (&lhs.kind, rhs) {
            (ExprKind::Hole, _) => {}
            (&ExprKind::Var(Res::Local(id), _), rhs) => match env.get_mut(id) {
                Some(var) => {
                    var.value = rhs;
                }
                None => return Err(Error::UnboundName(self.to_global_span(lhs.span))),
            },
            (ExprKind::Tuple(var_tup), Value::Tuple(tup)) => {
                for (expr, val) in var_tup.iter().zip(tup.iter()) {
                    self.update_binding(env, globals, *expr, val.clone())?;
                }
            }
            _ => unreachable!("unassignable pattern should be disallowed by compiler"),
        }
        Ok(())
    }

    fn update_array_index_single(
        &mut self,
        env: &mut Env,
        globals: &impl PackageStoreLookup,
        lhs: ExprId,
        span: PackageSpan,
        index: usize,
        rhs: Value,
    ) -> Result<(), Error> {
        let lhs = globals.get_expr((self.package, lhs).into());
        match &lhs.kind {
            &ExprKind::Var(Res::Local(id), _) => match env.get_mut(id) {
                Some(var) => {
                    var.value.update_array(index, rhs).map_err(|idx| {
                        Error::IndexOutOfRange(idx.try_into().expect("index should be valid"), span)
                    })?;
                }
                None => return Err(Error::UnboundName(self.to_global_span(lhs.span))),
            },
            _ => unreachable!("unassignable array update pattern should be disallowed by compiler"),
        }
        Ok(())
    }

    #[allow(clippy::similar_names)] // `env` and `end` are similar but distinct
    fn update_array_index_range(
        &mut self,
        env: &mut Env,
        globals: &impl PackageStoreLookup,
        lhs: ExprId,
        range_span: PackageSpan,
        range: &Value,
        update: Value,
    ) -> Result<(), Error> {
        let lhs = globals.get_expr((self.package, lhs).into());
        match &lhs.kind {
            &ExprKind::Var(Res::Local(id), _) => match env.get_mut(id) {
                Some(var) => {
                    let rhs = update.unwrap_array();
                    let Value::Array(arr) = &mut var.value else {
                        panic!("variable should be an array");
                    };
                    let Value::Range(inner) = range else {
                        unreachable!("range should be a Value::Range");
                    };
                    let range = make_range(arr, inner.start, inner.step, inner.end, range_span)?;
                    for (idx, rhs) in range.into_iter().zip(rhs.iter()) {
                        if idx < 0 {
                            return Err(Error::InvalidNegativeInt(idx, range_span));
                        }
                        let i = idx.as_index(range_span)?;
                        var.value.update_array(i, rhs.clone()).map_err(|idx| {
                            Error::IndexOutOfRange(
                                idx.try_into().expect("index should be valid"),
                                range_span,
                            )
                        })?;
                    }
                }
                None => return Err(Error::UnboundName(self.to_global_span(lhs.span))),
            },
            _ => unreachable!("unassignable array update pattern should be disallowed by compiler"),
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn bind_args_for_spec(
        &self,
        env: &mut Env,
        globals: &impl PackageStoreLookup,
        decl_pat: PatId,
        spec_pat: Option<PatId>,
        args_val: Value,
        ctl_count: u8,
        fixed_args: Option<Rc<[Value]>>,
    ) {
        match spec_pat {
            Some(spec_pat) => {
                assert!(
                    ctl_count > 0,
                    "spec pattern tuple used without controlled functor"
                );

                let mut tup = args_val;
                let mut ctls = vec![];
                for _ in 0..ctl_count {
                    let [c, rest] = &*tup.unwrap_tuple() else {
                        panic!("tuple should be arity 2");
                    };
                    ctls.extend_from_slice(&c.clone().unwrap_array());
                    tup = rest.clone();
                }

                self.bind_value(env, globals, spec_pat, Value::Array(ctls.into()));
                self.bind_value(env, globals, decl_pat, merge_fixed_args(fixed_args, tup));
            }
            None => self.bind_value(
                env,
                globals,
                decl_pat,
                merge_fixed_args(fixed_args, args_val),
            ),
        }
    }

    fn to_global_span(&self, span: Span) -> PackageSpan {
        PackageSpan {
            package: map_fir_package_to_hir(self.package),
            span,
        }
    }
}

fn merge_fixed_args(fixed_args: Option<Rc<[Value]>>, arg: Value) -> Value {
    if let Some(fixed_args) = fixed_args {
        Value::Tuple(fixed_args.iter().cloned().chain(iter::once(arg)).collect())
    } else {
        arg
    }
}

fn resolve_binding(env: &Env, package: PackageId, res: Res, span: Span) -> Result<Value, Error> {
    Ok(match res {
        Res::Err => panic!("resolution error"),
        Res::Item(item) => Value::Global(
            StoreItemId {
                package: item.package.unwrap_or(package),
                item: item.item,
            },
            FunctorApp::default(),
        ),
        Res::Local(id) => env
            .get(id)
            .ok_or(Error::UnboundName(PackageSpan {
                package: map_fir_package_to_hir(package),
                span,
            }))?
            .value
            .clone(),
    })
}

fn spec_from_functor_app(functor: FunctorApp) -> Spec {
    match (functor.adjoint, functor.controlled) {
        (false, 0) => Spec::Body,
        (true, 0) => Spec::Adj,
        (false, _) => Spec::Ctl,
        (true, _) => Spec::CtlAdj,
    }
}

fn resolve_closure(
    env: &Env,
    package: PackageId,
    span: Span,
    args: &[LocalVarId],
    callable: LocalItemId,
) -> Result<Value, Error> {
    let args: Option<_> = args
        .iter()
        .map(|&arg| Some(env.get(arg)?.value.clone()))
        .collect();
    let args: Vec<_> = args.ok_or(Error::UnboundName(PackageSpan {
        package: map_fir_package_to_hir(package),
        span,
    }))?;
    let callable = StoreItemId {
        package,
        item: callable,
    };
    Ok(Value::Closure(
        val::Closure {
            fixed_args: args.into(),
            id: callable,
            functor: FunctorApp::default(),
        }
        .into(),
    ))
}

fn lit_to_val(lit: &Lit) -> Value {
    match lit {
        Lit::BigInt(v) => Value::BigInt(v.clone()),
        Lit::Bool(v) => Value::Bool(*v),
        Lit::Double(v) => Value::Double(*v),
        Lit::Int(v) => Value::Int(*v),
        Lit::Pauli(v) => Value::Pauli(*v),
        Lit::Result(fir::Result::Zero) => Value::RESULT_ZERO,
        Lit::Result(fir::Result::One) => Value::RESULT_ONE,
    }
}

fn index_array(arr: &[Value], index: i64, span: PackageSpan) -> Result<Value, Error> {
    let i = index.as_index(span)?;
    match arr.get(i) {
        Some(v) => Ok(v.clone()),
        None => Err(Error::IndexOutOfRange(index, span)),
    }
}

fn slice_array(
    arr: &[Value],
    start: Option<i64>,
    step: i64,
    end: Option<i64>,
    span: PackageSpan,
) -> Result<Value, Error> {
    let range = make_range(arr, start, step, end, span)?;
    let mut slice = vec![];
    for i in range {
        slice.push(index_array(arr, i, span)?);
    }

    Ok(Value::Array(slice.into()))
}

fn make_range(
    arr: &[Value],
    start: Option<i64>,
    step: i64,
    end: Option<i64>,
    span: PackageSpan,
) -> Result<Range, Error> {
    if step == 0 {
        Err(Error::RangeStepZero(span))
    } else {
        let len: i64 = match arr.len().try_into() {
            Ok(len) => Ok(len),
            Err(_) => Err(Error::ArrayTooLarge(span)),
        }?;
        let (start, end) = if step > 0 {
            (start.unwrap_or(0), end.unwrap_or(len - 1))
        } else {
            (start.unwrap_or(len - 1), end.unwrap_or(0))
        };
        Ok(Range::new(start, step, end))
    }
}

fn eval_binop_add(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
        Value::Array(arr) => {
            let rhs_arr = rhs_val.unwrap_array();
            let items: Vec<_> = arr.iter().cloned().chain(rhs_arr.iter().cloned()).collect();
            Value::Array(items.into())
        }
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            Value::BigInt(val + rhs)
        }
        Value::Double(val) => {
            let rhs = rhs_val.unwrap_double();
            Value::Double(val + rhs)
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Int(val.wrapping_add(rhs))
        }
        Value::String(val) => {
            let rhs = rhs_val.unwrap_string();
            Value::String((val.to_string() + &rhs).into())
        }
        _ => panic!("value is not addable: {}", lhs_val.type_name()),
    }
}

fn eval_binop_andb(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            Value::BigInt(val & rhs)
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Int(val & rhs)
        }
        _ => panic!("value type does not support andb"),
    }
}

fn eval_binop_div(lhs_val: Value, rhs_val: Value, rhs_span: PackageSpan) -> Result<Value, Error> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            if rhs == BigInt::from(0) {
                Err(Error::DivZero(rhs_span))
            } else {
                Ok(Value::BigInt(val / rhs))
            }
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            if rhs == 0 {
                Err(Error::DivZero(rhs_span))
            } else {
                Ok(Value::Int(val.wrapping_div(rhs)))
            }
        }
        Value::Double(val) => {
            let rhs = rhs_val.unwrap_double();
            Ok(Value::Double(val / rhs))
        }
        _ => panic!("value should support div"),
    }
}

fn eval_binop_exp(lhs_val: Value, rhs_val: Value, rhs_span: PackageSpan) -> Result<Value, Error> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs_val = rhs_val.unwrap_int();
            if rhs_val < 0 {
                Err(Error::InvalidNegativeInt(rhs_val, rhs_span))
            } else {
                let rhs_val: u32 = match rhs_val.try_into() {
                    Ok(v) => Ok(v),
                    Err(_) => Err(Error::IntTooLarge(rhs_val, rhs_span)),
                }?;
                Ok(Value::BigInt(val.pow(rhs_val)))
            }
        }
        Value::Double(val) => Ok(Value::Double(val.powf(rhs_val.unwrap_double()))),
        Value::Int(val) => {
            let rhs_val = rhs_val.unwrap_int();
            if rhs_val < 0 {
                Err(Error::InvalidNegativeInt(rhs_val, rhs_span))
            } else {
                let result: i64 = match rhs_val.try_into() {
                    Ok(v) => val
                        .checked_pow(v)
                        .ok_or(Error::IntTooLarge(rhs_val, rhs_span)),
                    Err(_) => Err(Error::IntTooLarge(rhs_val, rhs_span)),
                }?;
                Ok(Value::Int(result))
            }
        }
        _ => panic!("value should support exp"),
    }
}

fn eval_binop_gt(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            Value::Bool(val > rhs)
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Bool(val > rhs)
        }
        Value::Double(val) => {
            let rhs = rhs_val.unwrap_double();
            Value::Bool(val > rhs)
        }
        _ => panic!("value doesn't support binop gt"),
    }
}

fn eval_binop_gte(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            Value::Bool(val >= rhs)
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Bool(val >= rhs)
        }
        Value::Double(val) => {
            let rhs = rhs_val.unwrap_double();
            Value::Bool(val >= rhs)
        }
        _ => panic!("value doesn't support binop gte"),
    }
}

fn eval_binop_lt(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            Value::Bool(val < rhs)
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Bool(val < rhs)
        }
        Value::Double(val) => {
            let rhs = rhs_val.unwrap_double();
            Value::Bool(val < rhs)
        }
        _ => panic!("value doesn't support binop lt"),
    }
}

fn eval_binop_lte(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            Value::Bool(val <= rhs)
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Bool(val <= rhs)
        }
        Value::Double(val) => {
            let rhs = rhs_val.unwrap_double();
            Value::Bool(val <= rhs)
        }
        _ => panic!("value doesn't support binop lte"),
    }
}

fn eval_binop_mod(lhs_val: Value, rhs_val: Value, rhs_span: PackageSpan) -> Result<Value, Error> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            if rhs == BigInt::from(0) {
                Err(Error::DivZero(rhs_span))
            } else {
                Ok(Value::BigInt(val % rhs))
            }
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            if rhs == 0 {
                Err(Error::DivZero(rhs_span))
            } else {
                Ok(Value::Int(val.wrapping_rem(rhs)))
            }
        }
        Value::Double(val) => {
            let rhs = rhs_val.unwrap_double();
            if rhs == 0.0 {
                Err(Error::DivZero(rhs_span))
            } else {
                Ok(Value::Double(val % rhs))
            }
        }
        _ => panic!("value should support mod"),
    }
}

fn eval_binop_mul(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            Value::BigInt(val * rhs)
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Int(val.wrapping_mul(rhs))
        }
        Value::Double(val) => {
            let rhs = rhs_val.unwrap_double();
            Value::Double(val * rhs)
        }
        _ => panic!("value should support mul"),
    }
}

fn eval_binop_orb(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            Value::BigInt(val | rhs)
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Int(val | rhs)
        }
        _ => panic!("value type does not support orb"),
    }
}

fn eval_binop_shl(lhs_val: Value, rhs_val: Value, rhs_span: PackageSpan) -> Result<Value, Error> {
    Ok(match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_int();
            if rhs > 0 {
                Value::BigInt(val << rhs)
            } else {
                Value::BigInt(val >> rhs.abs())
            }
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Int(if rhs > 0 {
                let shift: u32 = rhs.try_into().or(Err(Error::IntTooLarge(rhs, rhs_span)))?;
                val.checked_shl(shift)
                    .ok_or(Error::IntTooLarge(rhs, rhs_span))?
            } else {
                let shift: u32 = rhs
                    .checked_neg()
                    .ok_or(Error::IntTooLarge(rhs, rhs_span))?
                    .try_into()
                    .or(Err(Error::IntTooLarge(rhs, rhs_span)))?;
                val.checked_shr(shift)
                    .ok_or(Error::IntTooLarge(rhs, rhs_span))?
            })
        }
        _ => panic!("value should support shl"),
    })
}

fn eval_binop_shr(lhs_val: Value, rhs_val: Value, rhs_span: PackageSpan) -> Result<Value, Error> {
    Ok(match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_int();
            if rhs > 0 {
                Value::BigInt(val >> rhs)
            } else {
                Value::BigInt(val << rhs.abs())
            }
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Int(if rhs > 0 {
                let shift: u32 = rhs.try_into().or(Err(Error::IntTooLarge(rhs, rhs_span)))?;
                val.checked_shr(shift)
                    .ok_or(Error::IntTooLarge(rhs, rhs_span))?
            } else {
                let shift: u32 = rhs
                    .checked_neg()
                    .ok_or(Error::IntTooLarge(rhs, rhs_span))?
                    .try_into()
                    .or(Err(Error::IntTooLarge(rhs, rhs_span)))?;
                val.checked_shl(shift)
                    .ok_or(Error::IntTooLarge(rhs, rhs_span))?
            })
        }
        _ => panic!("value should support shr"),
    })
}

fn eval_binop_sub(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            Value::BigInt(val - rhs)
        }
        Value::Double(val) => {
            let rhs = rhs_val.unwrap_double();
            Value::Double(val - rhs)
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Int(val.wrapping_sub(rhs))
        }
        _ => panic!("value is not subtractable"),
    }
}

fn eval_binop_xorb(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            Value::BigInt(val ^ rhs)
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Int(val ^ rhs)
        }
        _ => panic!("value type does not support xorb"),
    }
}

fn update_functor_app(functor: Functor, app: FunctorApp) -> FunctorApp {
    match functor {
        Functor::Adj => FunctorApp {
            adjoint: !app.adjoint,
            controlled: app.controlled,
        },
        Functor::Ctl => FunctorApp {
            adjoint: app.adjoint,
            controlled: app.controlled + 1,
        },
    }
}

fn follow_field_path(mut value: Value, path: &[usize]) -> Option<Value> {
    for &index in path {
        let Value::Tuple(items) = value else {
            return None;
        };
        value = items[index].clone();
    }
    Some(value)
}

fn update_field_path(record: &Value, path: &[usize], replace: &Value) -> Option<Value> {
    match (record, path) {
        (_, []) => Some(replace.clone()),
        (Value::Tuple(items), &[next_index, ..]) if next_index < items.len() => {
            let update = |(index, item)| {
                if index == next_index {
                    update_field_path(item, &path[1..], replace)
                } else {
                    Some(item.clone())
                }
            };

            let items: Option<_> = items.iter().enumerate().map(update).collect();
            Some(Value::Tuple(items?))
        }
        _ => None,
    }
}

fn is_updatable_in_place(env: &Env, expr: &Expr) -> (bool, bool) {
    match &expr.kind {
        ExprKind::Var(Res::Local(id), _) => match env.get(*id) {
            Some(var) => match &var.value {
                Value::Array(var) => (true, Rc::weak_count(var) + Rc::strong_count(var) == 1),
                _ => (false, false),
            },
            _ => (false, false),
        },
        _ => (false, false),
    }
}
