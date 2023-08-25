// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

pub mod backend;
pub mod debug;
mod intrinsic;
pub mod lower;
pub mod output;
pub mod val;

use crate::val::{FunctorApp, Value};
use backend::Backend;
use debug::{CallStack, Frame};
use miette::Diagnostic;
use num_bigint::BigInt;
use output::Receiver;
use qsc_data_structures::span::Span;
use qsc_fir::fir::{
    self, BinOp, CallableDecl, ExprKind, Field, Functor, Lit, LocalItemId, Mutability, NodeId,
    PackageId, PatKind, PrimField, Res, SpecBody, SpecGen, StmtKind, StringComponent, UnOp,
};
use qsc_fir::fir::{BlockId, ExprId, PatId, StmtId};
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::{self, Display, Formatter, Write},
    iter,
    ops::Neg,
    rc::Rc,
};
use thiserror::Error;
use val::GlobalId;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("array too large")]
    #[diagnostic(code("Qsc.Eval.ArrayTooLarge"))]
    ArrayTooLarge(#[label("this array has too many items")] Span),

    #[error("invalid array length: {0}")]
    #[diagnostic(code("Qsc.Eval.InvalidArrayLength"))]
    InvalidArrayLength(i64, #[label("cannot be used as a length")] Span),

    #[error("division by zero")]
    #[diagnostic(code("Qsc.Eval.DivZero"))]
    DivZero(#[label("cannot divide by zero")] Span),

    #[error("empty range")]
    #[diagnostic(code("Qsc.Eval.EmptyRange"))]
    EmptyRange(#[label("the range cannot be empty")] Span),

    #[error("value cannot be used as an index: {0}")]
    #[diagnostic(code("Qsc.Eval.InvalidIndex"))]
    InvalidIndex(i64, #[label("invalid index")] Span),

    #[error("integer too large for operation")]
    #[diagnostic(code("Qsc.Eval.IntTooLarge"))]
    IntTooLarge(i64, #[label("this value is too large")] Span),

    #[error("missing specialization: {0}")]
    #[diagnostic(code("Qsc.Eval.MissingSpec"))]
    MissingSpec(String, #[label("callable has no {0} specialization")] Span),

    #[error("index out of range: {0}")]
    #[diagnostic(code("Qsc.Eval.IndexOutOfRange"))]
    IndexOutOfRange(i64, #[label("out of range")] Span),

    #[error("negative integers cannot be used here: {0}")]
    #[diagnostic(code("Qsc.Eval.InvalidNegativeInt"))]
    InvalidNegativeInt(i64, #[label("invalid negative integer")] Span),

    #[error("output failure")]
    #[diagnostic(code("Qsc.Eval.OutputFail"))]
    OutputFail(#[label("failed to generate output")] Span),

    #[error("qubits in gate invocation are not unique")]
    #[diagnostic(code("Qsc.Eval.QubitUniqueness"))]
    QubitUniqueness(#[label] Span),

    #[error("range with step size of zero")]
    #[diagnostic(code("Qsc.Eval.RangeStepZero"))]
    RangeStepZero(#[label("invalid range")] Span),

    #[error("Qubit{0} released while not in |0⟩ state")]
    #[diagnostic(code("Qsc.Eval.ReleasedQubitNotZero"))]
    ReleasedQubitNotZero(usize),

    #[error("name is not bound")]
    #[diagnostic(code("Qsc.Eval.UnboundName"))]
    UnboundName(#[label] Span),

    #[error("unknown intrinsic `{0}`")]
    #[diagnostic(code("Qsc.Eval.UnknownIntrinsic"))]
    UnknownIntrinsic(String, #[label("callable has no implementation")] Span),

    #[error("program failed: {0}")]
    #[diagnostic(code("Qsc.Eval.UserFail"))]
    UserFail(String, #[label("explicit fail")] Span),
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

/// Evaluates the given expr with the given context.
/// # Errors
/// Returns the first error encountered during execution.
/// # Panics
/// On internal error where no result is returned.
pub fn eval_expr(
    state: &mut State,
    expr: ExprId,
    globals: &impl NodeLookup,
    env: &mut Env,
    sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
    out: &mut impl Receiver,
) -> Result<Value, (Error, Vec<Frame>)> {
    state.push_expr(expr);
    let res = state.eval(globals, env, sim, out, &[], StepAction::Continue)?;
    let StepResult::Return(value) = res else {
        panic!("eval_expr should always return a value");
    };
    Ok(value)
}

/// Evaluates the given stmt with the given context.
/// # Errors
/// Returns the first error encountered during execution.
/// # Panics
/// On internal error where no result is returned.
pub fn eval_stmt(
    stmt: StmtId,
    globals: &impl NodeLookup,
    env: &mut Env,
    sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
    package: PackageId,
    receiver: &mut impl Receiver,
) -> Result<Value, (Error, Vec<Frame>)> {
    let mut state = State::new(package);
    state.push_stmt(stmt);
    let res = state.eval(globals, env, sim, receiver, &[], StepAction::Continue)?;
    let StepResult::Return(value) = res else {
        panic!("eval_stmt should always return a value");
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

pub fn eval_push_expr(state: &mut State, expr: ExprId) {
    state.push_expr(expr);
}

trait AsIndex {
    type Output;

    fn as_index(&self, span: Span) -> Self::Output;
}

impl AsIndex for i64 {
    type Output = Result<usize, Error>;

    fn as_index(&self, span: Span) -> Self::Output {
        match (*self).try_into() {
            Ok(index) => Ok(index),
            Err(_) => Err(Error::InvalidIndex(*self, span)),
        }
    }
}

#[derive(Debug, Clone)]
struct Variable {
    name: Rc<str>,
    value: Value,
    mutability: Mutability,
    span: Span,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub value: Value,
    pub name: Rc<str>,
    pub type_name: String,
    pub id: NodeId,
    pub mutability: Mutability,
    pub span: Span,
}

impl Variable {
    fn is_mutable(&self) -> bool {
        self.mutability == Mutability::Mutable
    }
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

pub enum Global<'a> {
    Callable(&'a CallableDecl),
    Udt,
}

pub trait NodeLookup {
    fn get(&self, id: GlobalId) -> Option<Global>;
    fn get_block(&self, package: PackageId, id: BlockId) -> &qsc_fir::fir::Block;
    fn get_expr(&self, package: PackageId, id: ExprId) -> &qsc_fir::fir::Expr;
    fn get_pat(&self, package: PackageId, id: PatId) -> &qsc_fir::fir::Pat;
    fn get_stmt(&self, package: PackageId, id: StmtId) -> &qsc_fir::fir::Stmt;
}

#[derive(Default)]
pub struct Env(Vec<Scope>);

impl Env {
    #[must_use]
    fn get(&self, id: NodeId) -> Option<&Variable> {
        self.0
            .iter()
            .rev()
            .find_map(|scope| scope.bindings.get(&id))
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut Variable> {
        self.0
            .iter_mut()
            .rev()
            .find_map(|scope| scope.bindings.get_mut(&id))
    }

    fn push_scope(&mut self, frame_id: usize) {
        let scope = Scope {
            frame_id,
            ..Default::default()
        };
        self.0.push(scope);
    }

    fn leave_scope(&mut self) {
        self.0
            .pop()
            .expect("scope should be entered first before leaving");
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
                    .map(|(id, var)| VariableInfo {
                        id: *id,
                        name: var.name.clone(),
                        type_name: var.value.type_name().to_string(),
                        value: var.value.clone(),
                        mutability: var.mutability,
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
    bindings: HashMap<NodeId, Variable>,
    frame_id: usize,
}

impl Env {
    #[must_use]
    pub fn with_empty_scope() -> Self {
        Self(vec![Scope::default()])
    }
}

enum Cont {
    Action(Action),
    Expr(ExprId),
    Frame(usize),
    Scope,
    Stmt(StmtId),
}

#[derive(Clone)]
enum Action {
    Array(usize),
    ArrayRepeat(Span),
    Assign(ExprId),
    Bind(PatId, Mutability),
    BinOp(BinOp, Span, Option<ExprId>),
    Call(Span, Span),
    Consume,
    Fail(Span),
    Field(Field),
    If(ExprId, Option<ExprId>),
    Index(Span),
    Range(bool, bool, bool),
    Return,
    StringConcat(usize),
    StringLit(Rc<str>),
    UpdateIndex(Span),
    Tuple(usize),
    UnOp(UnOp),
    UpdateField(Field),
    While(ExprId, BlockId),
}

pub struct State {
    stack: Vec<Cont>,
    vals: Vec<Value>,
    package: PackageId,
    call_stack: CallStack,
    current_span: Span,
}

impl State {
    #[must_use]
    pub fn new(package: PackageId) -> Self {
        Self {
            stack: Vec::new(),
            vals: Vec::new(),
            package,
            call_stack: CallStack::default(),
            current_span: Span::default(),
        }
    }

    fn pop_cont(&mut self) -> Option<Cont> {
        self.stack.pop()
    }

    fn push_action(&mut self, action: Action) {
        self.stack.push(Cont::Action(action));
    }

    fn push_expr(&mut self, expr: ExprId) {
        self.stack.push(Cont::Expr(expr));
    }

    fn push_frame(&mut self, id: GlobalId, functor: FunctorApp) {
        self.call_stack.push_frame(Frame {
            span: self.current_span,
            id,
            caller: self.package,
            functor,
        });
        self.stack.push(Cont::Frame(self.vals.len()));
        self.package = id.package;
    }

    fn leave_frame(&mut self, len: usize) {
        let frame = self
            .call_stack
            .pop_frame()
            .expect("frame should be present");
        self.package = frame.caller;
        let frame_val = self.pop_val();
        self.vals.drain(len..);
        self.push_val(frame_val);
    }

    fn push_scope(&mut self, env: &mut Env) {
        env.push_scope(self.call_stack.len());
        self.stack.push(Cont::Scope);
    }

    fn push_stmt(&mut self, stmt: StmtId) {
        self.stack.push(Cont::Stmt(stmt));
    }

    fn push_block(&mut self, env: &mut Env, globals: &impl NodeLookup, block: BlockId) {
        let block = globals.get_block(self.package, block);
        self.push_scope(env);
        for stmt in block.stmts.iter().rev() {
            self.push_stmt(*stmt);
            self.push_action(Action::Consume);
        }
        if block.stmts.is_empty() {
            self.push_val(Value::unit());
        } else {
            self.pop_cont();
        }
    }

    fn pop_val(&mut self) -> Value {
        self.vals.pop().expect("value should be present")
    }

    fn pop_vals(&mut self, len: usize) -> Vec<Value> {
        self.vals.drain(self.vals.len() - len..).collect()
    }

    fn push_val(&mut self, val: Value) {
        self.vals.push(val);
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
        globals: &impl NodeLookup,
        env: &mut Env,
        sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
        out: &mut impl Receiver,
        breakpoints: &[StmtId],
        step: StepAction,
    ) -> Result<StepResult, (Error, Vec<Frame>)> {
        let current_frame = self.call_stack.len();

        while let Some(cont) = self.pop_cont() {
            let res = match cont {
                Cont::Action(action) => {
                    self.cont_action(env, sim, globals, action, out)
                        .map_err(|e| (e, self.get_stack_frames()))?;
                    continue;
                }
                Cont::Expr(expr) => {
                    self.cont_expr(env, globals, expr)
                        .map_err(|e| (e, self.get_stack_frames()))?;
                    continue;
                }
                Cont::Frame(len) => {
                    self.leave_frame(len);
                    continue;
                }
                Cont::Scope => {
                    env.leave_scope();
                    continue;
                }
                Cont::Stmt(stmt) => {
                    self.cont_stmt(globals, stmt);
                    if let Some(bp) = breakpoints.iter().find(|&bp| *bp == stmt) {
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
            };

            if let StepResult::Return(_) = res {
                panic!("unexpected return");
            }

            return Ok(res);
        }

        Ok(StepResult::Return(self.get_result()))
    }

    pub fn get_result(&mut self) -> Value {
        self.pop_val()
    }

    #[allow(clippy::similar_names)]
    fn cont_expr(
        &mut self,
        env: &mut Env,
        globals: &impl NodeLookup,
        expr: ExprId,
    ) -> Result<(), Error> {
        let expr = globals.get_expr(self.package, expr);
        self.current_span = expr.span;

        match &expr.kind {
            ExprKind::Array(arr) => self.cont_arr(arr),
            ExprKind::ArrayRepeat(item, size) => self.cont_arr_repeat(globals, *item, *size),
            ExprKind::Assign(lhs, rhs) => self.cont_assign(*lhs, *rhs),
            ExprKind::AssignOp(op, lhs, rhs) => self.cont_assign_op(globals, *op, *lhs, *rhs),
            ExprKind::AssignField(record, field, replace) => {
                self.cont_assign_field(*record, field, *replace);
            }
            ExprKind::AssignIndex(lhs, mid, rhs) => {
                self.cont_assign_index(globals, *lhs, *mid, *rhs);
            }
            ExprKind::BinOp(op, lhs, rhs) => self.cont_binop(globals, *op, *rhs, *lhs),
            ExprKind::Block(block) => self.push_block(env, globals, *block),
            ExprKind::Call(callee_expr, args_expr) => {
                self.cont_call(globals, *callee_expr, *args_expr);
            }
            ExprKind::Closure(args, callable) => {
                let closure = resolve_closure(env, self.package, expr.span, args, *callable)?;
                self.push_val(closure);
            }
            ExprKind::Fail(fail_expr) => self.cont_fail(expr.span, *fail_expr),
            ExprKind::Field(expr, field) => self.cont_field(*expr, field),
            ExprKind::Hole => panic!("hole expr should be disallowed by passes"),
            ExprKind::If(cond_expr, then_expr, else_expr) => {
                self.cont_if(*cond_expr, *then_expr, *else_expr);
            }
            ExprKind::Index(arr, index) => self.cont_index(globals, *arr, *index),
            ExprKind::Lit(lit) => self.push_val(lit_to_val(lit)),
            ExprKind::Range(start, step, end) => self.cont_range(*start, *step, *end),
            ExprKind::Return(expr) => self.cont_ret(*expr),
            ExprKind::String(components) => self.cont_string(components),
            ExprKind::UpdateIndex(lhs, mid, rhs) => self.update_index(globals, *lhs, *mid, *rhs),
            ExprKind::Tuple(tup) => self.cont_tup(tup),
            ExprKind::UnOp(op, expr) => self.cont_unop(*op, *expr),
            ExprKind::UpdateField(record, field, replace) => {
                self.cont_update_field(*record, field, *replace);
            }
            ExprKind::Var(res, _) => {
                self.push_val(resolve_binding(env, self.package, *res, expr.span)?);
            }
            ExprKind::While(cond_expr, block) => self.cont_while(*cond_expr, *block),
        }

        Ok(())
    }

    fn cont_tup(&mut self, tup: &Vec<ExprId>) {
        self.push_action(Action::Tuple(tup.len()));
        for tup_expr in tup.iter().rev() {
            self.push_expr(*tup_expr);
        }
    }

    fn cont_arr(&mut self, arr: &Vec<ExprId>) {
        self.push_action(Action::Array(arr.len()));
        for entry in arr.iter().rev() {
            self.push_expr(*entry);
        }
    }

    fn cont_arr_repeat(&mut self, globals: &impl NodeLookup, item: ExprId, size: ExprId) {
        let size_expr = globals.get_expr(self.package, size);
        self.push_action(Action::ArrayRepeat(size_expr.span));
        self.push_expr(size);
        self.push_expr(item);
    }

    fn cont_ret(&mut self, expr: ExprId) {
        self.push_action(Action::Return);
        self.push_expr(expr);
    }

    fn cont_if(&mut self, cond_expr: ExprId, then_expr: ExprId, else_expr: Option<ExprId>) {
        self.push_action(Action::If(then_expr, else_expr));
        self.push_expr(cond_expr);
    }

    fn cont_fail(&mut self, span: Span, fail_expr: ExprId) {
        self.push_action(Action::Fail(span));
        self.push_expr(fail_expr);
    }

    fn cont_assign(&mut self, lhs: ExprId, rhs: ExprId) {
        self.push_action(Action::Assign(lhs));
        self.push_expr(rhs);
        self.push_val(Value::unit());
    }

    fn cont_assign_op(&mut self, globals: &impl NodeLookup, op: BinOp, lhs: ExprId, rhs: ExprId) {
        self.push_action(Action::Assign(lhs));
        self.cont_binop(globals, op, rhs, lhs);
        self.push_val(Value::unit());
    }

    fn cont_assign_field(&mut self, record: ExprId, field: &Field, replace: ExprId) {
        self.push_action(Action::Assign(record));
        self.cont_update_field(record, field, replace);
        self.push_val(Value::unit());
    }

    fn cont_assign_index(
        &mut self,
        globals: &impl NodeLookup,
        lhs: ExprId,
        mid: ExprId,
        rhs: ExprId,
    ) {
        self.push_action(Action::Assign(lhs));
        self.update_index(globals, lhs, mid, rhs);
        self.push_val(Value::unit());
    }

    fn cont_field(&mut self, expr: ExprId, field: &Field) {
        self.push_action(Action::Field(field.clone()));
        self.push_expr(expr);
    }

    fn cont_index(&mut self, globals: &impl NodeLookup, arr: ExprId, index: ExprId) {
        let index_expr = globals.get_expr(self.package, index);
        self.push_action(Action::Index(index_expr.span));
        self.push_expr(index);
        self.push_expr(arr);
    }

    fn cont_range(&mut self, start: Option<ExprId>, step: Option<ExprId>, end: Option<ExprId>) {
        self.push_action(Action::Range(
            start.is_some(),
            step.is_some(),
            end.is_some(),
        ));
        if let Some(end) = end {
            self.push_expr(end);
        }
        if let Some(step) = step {
            self.push_expr(step);
        }
        if let Some(start) = start {
            self.push_expr(start);
        }
    }

    fn cont_string(&mut self, components: &[StringComponent]) {
        if let [StringComponent::Lit(str)] = components {
            self.push_val(Value::String(Rc::clone(str)));
            return;
        }

        self.push_action(Action::StringConcat(components.len()));
        for component in components.iter().rev() {
            match component {
                StringComponent::Expr(expr) => self.push_expr(*expr),
                StringComponent::Lit(lit) => self.push_action(Action::StringLit(lit.clone())),
            }
        }
    }

    fn cont_while(&mut self, cond_expr: ExprId, block: BlockId) {
        self.push_action(Action::While(cond_expr, block));
        self.push_expr(cond_expr);
    }

    fn cont_call(&mut self, globals: &impl NodeLookup, callee: ExprId, args: ExprId) {
        let callee_expr = globals.get_expr(self.package, callee);
        let args_expr = globals.get_expr(self.package, args);
        self.push_action(Action::Call(callee_expr.span, args_expr.span));
        self.push_expr(args);
        self.push_expr(callee);
    }

    fn cont_binop(&mut self, globals: &impl NodeLookup, op: BinOp, rhs: ExprId, lhs: ExprId) {
        let rhs_expr = globals.get_expr(self.package, rhs);
        match op {
            BinOp::Add
            | BinOp::AndB
            | BinOp::Div
            | BinOp::Eq
            | BinOp::Exp
            | BinOp::Gt
            | BinOp::Gte
            | BinOp::Lt
            | BinOp::Lte
            | BinOp::Mod
            | BinOp::Mul
            | BinOp::Neq
            | BinOp::OrB
            | BinOp::Shl
            | BinOp::Shr
            | BinOp::Sub
            | BinOp::XorB => {
                self.push_action(Action::BinOp(op, rhs_expr.span, None));
                self.push_expr(rhs);
                self.push_expr(lhs);
            }
            BinOp::AndL | BinOp::OrL => {
                self.push_action(Action::BinOp(op, rhs_expr.span, Some(rhs)));
                self.push_expr(lhs);
            }
        }
    }

    fn update_index(&mut self, globals: &impl NodeLookup, lhs: ExprId, mid: ExprId, rhs: ExprId) {
        let mid_expr = globals.get_expr(self.package, mid);
        self.push_action(Action::UpdateIndex(mid_expr.span));
        self.push_expr(lhs);
        self.push_expr(rhs);
        self.push_expr(mid);
    }

    fn cont_unop(&mut self, op: UnOp, expr: ExprId) {
        self.push_action(Action::UnOp(op));
        self.push_expr(expr);
    }

    fn cont_update_field(&mut self, record: ExprId, field: &Field, replace: ExprId) {
        self.push_action(Action::UpdateField(field.clone()));
        self.push_expr(replace);
        self.push_expr(record);
    }

    fn cont_stmt(&mut self, globals: &impl NodeLookup, stmt: StmtId) {
        let stmt = globals.get_stmt(self.package, stmt);
        self.current_span = stmt.span;

        match &stmt.kind {
            StmtKind::Expr(expr) => self.push_expr(*expr),
            StmtKind::Item(..) => self.push_val(Value::unit()),
            StmtKind::Local(mutability, pat, expr) => {
                self.push_action(Action::Bind(*pat, *mutability));
                self.push_expr(*expr);
                self.push_val(Value::unit());
            }
            StmtKind::Qubit(..) => panic!("qubit use-stmt should be eliminated by passes"),
            StmtKind::Semi(expr) => {
                self.push_action(Action::Consume);
                self.push_expr(*expr);
                self.push_val(Value::unit());
            }
        }
    }

    fn cont_action(
        &mut self,
        env: &mut Env,
        sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
        globals: &impl NodeLookup,
        action: Action,
        out: &mut impl Receiver,
    ) -> Result<(), Error> {
        match action {
            Action::Array(len) => self.eval_arr(len),
            Action::ArrayRepeat(span) => self.eval_arr_repeat(span)?,
            Action::Assign(lhs) => self.eval_assign(env, globals, lhs)?,
            Action::BinOp(op, span, rhs) => self.eval_binop(op, span, rhs)?,
            Action::Bind(pat, mutability) => self.eval_bind(env, globals, pat, mutability),
            Action::Call(callee_span, args_span) => {
                self.eval_call(env, sim, globals, callee_span, args_span, out)?;
            }
            Action::Consume => {
                self.pop_val();
            }
            Action::Fail(span) => {
                return Err(Error::UserFail(
                    self.pop_val().unwrap_string().to_string(),
                    span,
                ));
            }
            Action::Field(field) => self.eval_field(field),
            Action::If(then_expr, else_expr) => self.eval_if(then_expr, else_expr),
            Action::Index(span) => self.eval_index(span)?,
            Action::Range(has_start, has_step, has_end) => {
                self.eval_range(has_start, has_step, has_end);
            }
            Action::Return => self.eval_ret(env),
            Action::StringConcat(len) => self.eval_string_concat(len),
            Action::StringLit(str) => self.push_val(Value::String(str)),
            Action::UpdateIndex(span) => self.eval_update_index(span)?,
            Action::Tuple(len) => self.eval_tup(len),
            Action::UnOp(op) => self.eval_unop(op),
            Action::UpdateField(field) => self.eval_update_field(field),
            Action::While(cond_expr, block) => self.eval_while(env, globals, cond_expr, block),
        }
        Ok(())
    }

    fn eval_arr(&mut self, len: usize) {
        let arr = self.pop_vals(len);
        self.push_val(Value::Array(arr.into()));
    }

    fn eval_arr_repeat(&mut self, span: Span) -> Result<(), Error> {
        let size_val = self.pop_val().unwrap_int();
        let item_val = self.pop_val();
        let s = match size_val.try_into() {
            Ok(i) => Ok(i),
            Err(_) => Err(Error::InvalidArrayLength(size_val, span)),
        }?;
        self.push_val(Value::Array(vec![item_val; s].into()));
        Ok(())
    }

    fn eval_assign(
        &mut self,
        env: &mut Env,
        globals: &impl NodeLookup,
        lhs: ExprId,
    ) -> Result<(), Error> {
        let rhs = self.pop_val();
        self.update_binding(env, globals, lhs, rhs)
    }

    fn eval_bind(
        &mut self,
        env: &mut Env,
        globals: &impl NodeLookup,
        pat: PatId,
        mutability: Mutability,
    ) {
        let val = self.pop_val();
        self.bind_value(env, globals, pat, val, mutability);
    }

    fn eval_binop(&mut self, op: BinOp, span: Span, rhs: Option<ExprId>) -> Result<(), Error> {
        match op {
            BinOp::Add => self.eval_binop_simple(eval_binop_add),
            BinOp::AndB => self.eval_binop_simple(eval_binop_andb),
            BinOp::AndL => {
                if self.pop_val().unwrap_bool() {
                    self.push_expr(rhs.expect("rhs should be provided with binop andl"));
                } else {
                    self.push_val(Value::Bool(false));
                }
            }
            BinOp::Div => self.eval_binop_with_error(span, eval_binop_div)?,
            BinOp::Eq => {
                let rhs_val = self.pop_val();
                let lhs_val = self.pop_val();
                self.push_val(Value::Bool(lhs_val == rhs_val));
            }
            BinOp::Exp => self.eval_binop_with_error(span, eval_binop_exp)?,
            BinOp::Gt => self.eval_binop_simple(eval_binop_gt),
            BinOp::Gte => self.eval_binop_simple(eval_binop_gte),
            BinOp::Lt => self.eval_binop_simple(eval_binop_lt),
            BinOp::Lte => self.eval_binop_simple(eval_binop_lte),
            BinOp::Mod => self.eval_binop_simple(eval_binop_mod),
            BinOp::Mul => self.eval_binop_simple(eval_binop_mul),
            BinOp::Neq => {
                let rhs_val = self.pop_val();
                let lhs_val = self.pop_val();
                self.push_val(Value::Bool(lhs_val != rhs_val));
            }
            BinOp::OrB => self.eval_binop_simple(eval_binop_orb),
            BinOp::OrL => {
                if self.pop_val().unwrap_bool() {
                    self.push_val(Value::Bool(true));
                } else {
                    self.push_expr(rhs.expect("rhs should be provided with binop andl"));
                }
            }
            BinOp::Shl => self.eval_binop_simple(eval_binop_shl),
            BinOp::Shr => self.eval_binop_simple(eval_binop_shr),
            BinOp::Sub => self.eval_binop_simple(eval_binop_sub),
            BinOp::XorB => self.eval_binop_simple(eval_binop_xorb),
        }
        Ok(())
    }

    fn eval_binop_simple(&mut self, binop_func: impl FnOnce(Value, Value) -> Value) {
        let rhs_val = self.pop_val();
        let lhs_val = self.pop_val();
        self.push_val(binop_func(lhs_val, rhs_val));
    }

    fn eval_binop_with_error(
        &mut self,
        span: Span,
        binop_func: impl FnOnce(Value, Value, Span) -> Result<Value, Error>,
    ) -> Result<(), Error> {
        let rhs_val = self.pop_val();
        let lhs_val = self.pop_val();
        self.push_val(binop_func(lhs_val, rhs_val, span)?);
        Ok(())
    }

    fn eval_call(
        &mut self,
        env: &mut Env,
        sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
        globals: &impl NodeLookup,
        callee_span: Span,
        arg_span: Span,
        out: &mut impl Receiver,
    ) -> Result<(), Error> {
        let arg = self.pop_val();
        let (callee_id, functor, fixed_args) = match self.pop_val() {
            Value::Closure(fixed_args, id, functor) => (id, functor, Some(fixed_args)),
            Value::Global(id, functor) => (id, functor, None),
            _ => panic!("value is not callable"),
        };

        let callee = match globals.get(callee_id) {
            Some(Global::Callable(callable)) => callable,
            Some(Global::Udt) => {
                self.push_val(arg);
                return Ok(());
            }
            None => return Err(Error::UnboundName(callee_span)),
        };

        let spec = spec_from_functor_app(functor);
        self.push_frame(callee_id, functor);
        self.push_scope(env);
        let block_body = &match spec {
            Spec::Body => Some(&callee.body),
            Spec::Adj => callee.adj.as_ref(),
            Spec::Ctl => callee.ctl.as_ref(),
            Spec::CtlAdj => callee.ctl_adj.as_ref(),
        }
        .ok_or(Error::MissingSpec(spec.to_string(), callee_span))?
        .body;
        match block_body {
            SpecBody::Impl(input, body_block) => {
                self.bind_args_for_spec(
                    env,
                    globals,
                    callee.input,
                    *input,
                    arg,
                    functor.controlled,
                    fixed_args,
                );
                self.push_block(env, globals, *body_block);
                Ok(())
            }
            SpecBody::Gen(SpecGen::Intrinsic) => {
                let name = &callee.name.name;
                let val = intrinsic::call(name, callee_span, arg, arg_span, sim, out)?;
                self.push_val(val);
                Ok(())
            }
            SpecBody::Gen(_) => Err(Error::MissingSpec(spec.to_string(), callee_span)),
        }
    }

    fn eval_field(&mut self, field: Field) {
        let record = self.pop_val();
        let val = match (record, field) {
            (Value::Range(Some(start), _, _), Field::Prim(PrimField::Start)) => Value::Int(start),
            (Value::Range(_, step, _), Field::Prim(PrimField::Step)) => Value::Int(step),
            (Value::Range(_, _, Some(end)), Field::Prim(PrimField::End)) => Value::Int(end),
            (record, Field::Path(path)) => {
                follow_field_path(record, &path.indices).expect("field path should be valid")
            }
            _ => panic!("invalid field access"),
        };
        self.push_val(val);
    }

    fn eval_if(&mut self, then_expr: ExprId, else_expr: Option<ExprId>) {
        if self.pop_val().unwrap_bool() {
            self.push_expr(then_expr);
        } else if let Some(else_expr) = else_expr {
            self.push_expr(else_expr);
        } else {
            self.push_val(Value::unit());
        }
    }

    fn eval_index(&mut self, span: Span) -> Result<(), Error> {
        let index_val = self.pop_val();
        let arr = self.pop_val().unwrap_array();
        match &index_val {
            Value::Int(i) => self.push_val(index_array(&arr, *i, span)?),
            &Value::Range(start, step, end) => {
                self.push_val(slice_array(&arr, start, step, end, span)?);
            }
            _ => panic!("array should only be indexed by Int or Range"),
        }
        Ok(())
    }

    fn eval_range(&mut self, has_start: bool, has_step: bool, has_end: bool) {
        let end = if has_end {
            Some(self.pop_val().unwrap_int())
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
        self.push_val(Value::Range(start, step, end));
    }

    fn eval_ret(&mut self, env: &mut Env) {
        while let Some(cont) = self.pop_cont() {
            match cont {
                Cont::Frame(len) => {
                    self.leave_frame(len);
                    break;
                }
                Cont::Scope => env.leave_scope(),
                _ => {}
            }
        }
    }

    fn eval_string_concat(&mut self, len: usize) {
        let mut string = String::new();
        for component in self.pop_vals(len) {
            write!(string, "{component}").expect("string should be writable");
        }
        self.push_val(Value::String(string.into()));
    }

    fn eval_update_index(&mut self, span: Span) -> Result<(), Error> {
        let values = self.pop_val().unwrap_array();
        let update = self.pop_val();
        let index = self.pop_val().unwrap_int();
        if index < 0 {
            return Err(Error::InvalidNegativeInt(index, span));
        }
        let i = index.as_index(span)?;
        let mut values = values.iter().cloned().collect::<Vec<_>>();
        match values.get_mut(i) {
            Some(value) => {
                *value = update;
            }
            None => return Err(Error::IndexOutOfRange(index, span)),
        }
        self.push_val(Value::Array(values.into()));
        Ok(())
    }

    fn eval_tup(&mut self, len: usize) {
        let tup = self.pop_vals(len);
        self.push_val(Value::Tuple(tup.into()));
    }

    fn eval_unop(&mut self, op: UnOp) {
        let val = self.pop_val();
        match op {
            UnOp::Functor(functor) => match val {
                Value::Closure(args, id, app) => {
                    self.push_val(Value::Closure(args, id, update_functor_app(functor, app)));
                }
                Value::Global(id, app) => {
                    self.push_val(Value::Global(id, update_functor_app(functor, app)));
                }
                _ => panic!("value should be callable"),
            },
            UnOp::Neg => match val {
                Value::BigInt(v) => self.push_val(Value::BigInt(v.neg())),
                Value::Double(v) => self.push_val(Value::Double(v.neg())),
                Value::Int(v) => self.push_val(Value::Int(v.wrapping_neg())),
                _ => panic!("value should be number"),
            },
            UnOp::NotB => match val {
                Value::Int(v) => self.push_val(Value::Int(!v)),
                Value::BigInt(v) => self.push_val(Value::BigInt(!v)),
                _ => panic!("value should be Int or BigInt"),
            },
            UnOp::NotL => match val {
                Value::Bool(b) => self.push_val(Value::Bool(!b)),
                _ => panic!("value should be bool"),
            },
            UnOp::Pos => match val {
                Value::BigInt(_) | Value::Int(_) | Value::Double(_) => self.push_val(val),
                _ => panic!("value should be number"),
            },
            UnOp::Unwrap => self.push_val(val),
        }
    }

    fn eval_update_field(&mut self, field: Field) {
        let value = self.pop_val();
        let record = self.pop_val();
        let update = match (record, field) {
            (Value::Range(_, step, end), Field::Prim(PrimField::Start)) => {
                Value::Range(Some(value.unwrap_int()), step, end)
            }
            (Value::Range(start, _, end), Field::Prim(PrimField::Step)) => {
                Value::Range(start, value.unwrap_int(), end)
            }
            (Value::Range(start, step, _), Field::Prim(PrimField::End)) => {
                Value::Range(start, step, Some(value.unwrap_int()))
            }
            (record, Field::Path(path)) => update_field_path(&record, &path.indices, &value)
                .expect("field path should be valid"),
            _ => panic!("invalid field access"),
        };
        self.push_val(update);
    }

    fn eval_while(
        &mut self,
        env: &mut Env,
        globals: &impl NodeLookup,
        cond_expr: ExprId,
        block: BlockId,
    ) {
        if self.pop_val().unwrap_bool() {
            self.cont_while(cond_expr, block);
            self.push_action(Action::Consume);
            self.push_val(Value::unit());
            self.push_block(env, globals, block);
        } else {
            self.push_val(Value::unit());
        }
    }

    fn bind_value(
        &self,
        env: &mut Env,
        globals: &impl NodeLookup,
        pat: PatId,
        val: Value,
        mutability: Mutability,
    ) {
        let pat = globals.get_pat(self.package, pat);
        match &pat.kind {
            PatKind::Bind(variable) => {
                let scope = env.0.last_mut().expect("binding should have a scope");
                match scope.bindings.entry(variable.id) {
                    Entry::Vacant(entry) => entry.insert(Variable {
                        name: variable.name.clone(),
                        value: val,
                        mutability,
                        span: variable.span,
                    }),
                    Entry::Occupied(_) => panic!("duplicate binding"),
                };
            }
            PatKind::Discard => {}
            PatKind::Tuple(tup) => {
                let val_tup = val.unwrap_tuple();
                for (pat, val) in tup.iter().zip(val_tup.iter()) {
                    self.bind_value(env, globals, *pat, val.clone(), mutability);
                }
            }
        }
    }

    #[allow(clippy::similar_names)]
    fn update_binding(
        &self,
        env: &mut Env,
        globals: &impl NodeLookup,
        lhs: ExprId,
        rhs: Value,
    ) -> Result<(), Error> {
        let lhs = globals.get_expr(self.package, lhs);
        match (&lhs.kind, rhs) {
            (ExprKind::Hole, _) => {}
            (&ExprKind::Var(Res::Local(node), _), rhs) => match env.get_mut(node) {
                Some(var) if var.is_mutable() => {
                    var.value = rhs;
                }
                Some(_) => panic!("update of mutable variable should be disallowed by compiler"),
                None => return Err(Error::UnboundName(lhs.span)),
            },
            (ExprKind::Tuple(var_tup), Value::Tuple(tup)) => {
                for (expr, val) in var_tup.iter().zip(tup.iter()) {
                    self.update_binding(env, globals, *expr, val.clone())?;
                }
            }
            _ => panic!("unassignable pattern should be disallowed by compiler"),
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn bind_args_for_spec(
        &self,
        env: &mut Env,
        globals: &impl NodeLookup,
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

                self.bind_value(
                    env,
                    globals,
                    spec_pat,
                    Value::Array(ctls.into()),
                    Mutability::Immutable,
                );
                self.bind_value(
                    env,
                    globals,
                    decl_pat,
                    merge_fixed_args(fixed_args, tup),
                    Mutability::Immutable,
                );
            }
            None => self.bind_value(
                env,
                globals,
                decl_pat,
                merge_fixed_args(fixed_args, args_val),
                Mutability::Immutable,
            ),
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
            GlobalId {
                package: item.package.unwrap_or(package),
                item: item.item,
            },
            FunctorApp::default(),
        ),
        Res::Local(node) => env.get(node).ok_or(Error::UnboundName(span))?.value.clone(),
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
    args: &[NodeId],
    callable: LocalItemId,
) -> Result<Value, Error> {
    let args: Option<_> = args
        .iter()
        .map(|&arg| Some(env.get(arg)?.value.clone()))
        .collect();
    let args: Vec<_> = args.ok_or(Error::UnboundName(span))?;
    let callable = GlobalId {
        package,
        item: callable,
    };
    Ok(Value::Closure(args.into(), callable, FunctorApp::default()))
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

fn index_array(arr: &[Value], index: i64, span: Span) -> Result<Value, Error> {
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
    span: Span,
) -> Result<Value, Error> {
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

        let range = Range::new(start, step, end);
        let mut slice = vec![];
        for i in range {
            slice.push(index_array(arr, i, span)?);
        }

        Ok(Value::Array(slice.into()))
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
            Value::Int(val + rhs)
        }
        Value::String(val) => {
            let rhs = rhs_val.unwrap_string();
            Value::String((val.to_string() + &rhs).into())
        }
        _ => panic!("value is not addable"),
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

fn eval_binop_div(lhs_val: Value, rhs_val: Value, rhs_span: Span) -> Result<Value, Error> {
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
                Ok(Value::Int(val / rhs))
            }
        }
        Value::Double(val) => {
            let rhs = rhs_val.unwrap_double();
            if rhs == 0.0 {
                Err(Error::DivZero(rhs_span))
            } else {
                Ok(Value::Double(val / rhs))
            }
        }
        _ => panic!("value should support div"),
    }
}

fn eval_binop_exp(lhs_val: Value, rhs_val: Value, rhs_span: Span) -> Result<Value, Error> {
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
                let rhs_val: u32 = match rhs_val.try_into() {
                    Ok(v) => Ok(v),
                    Err(_) => Err(Error::IntTooLarge(rhs_val, rhs_span)),
                }?;
                Ok(Value::Int(val.pow(rhs_val)))
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

fn eval_binop_mod(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs = rhs_val.unwrap_big_int();
            Value::BigInt(val % rhs)
        }
        Value::Int(val) => {
            let rhs = rhs_val.unwrap_int();
            Value::Int(val % rhs)
        }
        Value::Double(val) => {
            let rhs = rhs_val.unwrap_double();
            Value::Double(val % rhs)
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
            Value::Int(val * rhs)
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

fn eval_binop_shl(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
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
            if rhs > 0 {
                Value::Int(val << rhs)
            } else {
                Value::Int(val >> rhs.abs())
            }
        }
        _ => panic!("value should support shl"),
    }
}

fn eval_binop_shr(lhs_val: Value, rhs_val: Value) -> Value {
    match lhs_val {
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
            if rhs > 0 {
                Value::Int(val >> rhs)
            } else {
                Value::Int(val << rhs.abs())
            }
        }
        _ => panic!("value should support shr"),
    }
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
            Value::Int(val - rhs)
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
        let Value::Tuple(items) = value else { return None; };
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
