// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

pub mod debug;
mod intrinsic;
pub mod output;
pub mod val;

use crate::val::{FunctorApp, Value};
use debug::{CallStack, Frame};
use miette::Diagnostic;
use num_bigint::BigInt;
use output::Receiver;
use qir_backend::__quantum__rt__initialize;
use qsc_data_structures::span::Span;
use qsc_hir::hir::{
    self, BinOp, Block, CallableBody, CallableDecl, Expr, ExprKind, Field, Functor, Lit,
    Mutability, NodeId, PackageId, Pat, PatKind, PrimField, Res, Spec, SpecBody, SpecGen, Stmt,
    StmtKind, StringComponent, TernOp, UnOp,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    convert::AsRef,
    fmt::Write,
    ops::Neg,
    ptr::null_mut,
    rc::Rc,
};
use thiserror::Error;
use val::GlobalId;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("array too large")]
    ArrayTooLarge(#[label("this array has too many items")] Span),

    #[error("invalid array length: {0}")]
    Count(i64, #[label("cannot be used as a length")] Span),

    #[error("division by zero")]
    DivZero(#[label("cannot divide by zero")] Span),

    #[error("empty range")]
    EmptyRange(#[label("the range cannot be empty")] Span),

    #[error("value cannot be used as an index: {0}")]
    IndexVal(i64, #[label("invalid index")] Span),

    #[error("integer too large for operation")]
    IntTooLarge(i64, #[label("this value is too large")] Span),

    #[error("missing specialization: {0}")]
    MissingSpec(Spec, #[label("callable has no {0} specialization")] Span),

    #[error("reassigning immutable variable")]
    Mutability(#[label("variable declared as immutable")] Span),

    #[error("index out of range: {0}")]
    OutOfRange(i64, #[label("out of range")] Span),

    #[error("negative integers cannot be used here: {0}")]
    Negative(i64, #[label("invalid negative integer")] Span),

    #[error("output failure")]
    Output(#[label("failed to generate output")] Span),

    #[error("qubits in gate invocation are not unique")]
    QubitUniqueness(#[label] Span),

    #[error("range with step size of zero")]
    RangeStepZero(#[label("invalid range")] Span),

    #[error("Qubit{0} released while not in |0⟩ state")]
    ReleasedQubitNotZero(usize),

    #[error("invalid left-hand side of assignment")]
    #[diagnostic(help("the left-hand side must be a variable or tuple of variables"))]
    Unassignable(#[label("not assignable")] Span),

    #[error("symbol is not bound")]
    Unbound(#[label] Span),

    #[error("unknown intrinsic")]
    UnknownIntrinsic(#[label("callable has no implementation")] Span),

    #[error("program failed: {0}")]
    UserFail(String, #[label("explicit fail")] Span),
}

/// Evaluates the given statement with the given context.
/// # Errors
/// Returns the first error encountered during execution.
pub fn eval_stmt<'a>(
    stmt: &'a Stmt,
    globals: &'a impl GlobalLookup<'a>,
    package: PackageId,
    env: &'a mut Env,
    out: &'a mut dyn Receiver,
) -> Result<Value, (Error, CallStack)> {
    let mut state = State::new(globals, package, env, out);
    state.push_stmt(stmt);
    state.eval()
}

/// Evaluates the given expression with the given context.
/// # Errors
/// Returns the first error encountered during execution.
pub fn eval_expr<'a>(
    expr: &'a Expr,
    globals: &'a impl GlobalLookup<'a>,
    package: PackageId,
    env: &'a mut Env,
    out: &'a mut dyn Receiver,
) -> Result<Value, (Error, CallStack)> {
    let mut state = State::new(globals, package, env, out);
    state.push_expr(expr);
    state.eval()
}

pub fn init() {
    __quantum__rt__initialize(null_mut());
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
            Err(_) => Err(Error::IndexVal(*self, span)),
        }
    }
}

#[derive(Debug)]
struct Variable {
    value: Value,
    mutability: Mutability,
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

pub trait GlobalLookup<'a> {
    fn get(&self, id: GlobalId) -> Option<Global<'a>>;
}

impl<'a, F: Fn(GlobalId) -> Option<Global<'a>>> GlobalLookup<'a> for F {
    fn get(&self, id: GlobalId) -> Option<Global<'a>> {
        self(id)
    }
}

#[derive(Default)]
pub struct Env(Vec<Scope>);

impl Env {
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
}

#[derive(Default)]
struct Scope {
    bindings: HashMap<NodeId, Variable>,
}

impl Env {
    #[must_use]
    pub fn with_empty_scope() -> Self {
        Self(vec![Scope::default()])
    }
}

enum Cont<'a> {
    Action(Action<'a>),
    Expr(&'a Expr),
    Frame(usize),
    Scope,
    Stmt(&'a Stmt),
}

#[derive(Copy, Clone)]
enum Action<'a> {
    Array(usize),
    ArrayRepeat(Span),
    Assign(&'a Expr),
    Bind(&'a Pat, Mutability),
    BinOp(BinOp, Span, Option<&'a Expr>),
    Call(Span, Span),
    Consume,
    Fail(Span),
    Field(&'a Field),
    If(&'a Block, Option<&'a Expr>),
    Index(Span),
    Range(bool, bool, bool),
    Return,
    StringConcat(usize),
    StringLit(&'a Rc<str>),
    TernOp(TernOp, &'a Expr, &'a Expr),
    Tuple(usize),
    UnOp(UnOp),
    UpdateField(&'a Field),
    While(&'a Expr, &'a Block),
}

pub(crate) struct State<'a, G> {
    stack: Vec<Cont<'a>>,
    vals: Vec<Value>,
    package: PackageId,
    globals: &'a G,
    env: &'a mut Env,
    out: &'a mut dyn Receiver,
    call_stack: CallStack,
}

impl<'a, G: GlobalLookup<'a>> State<'a, G> {
    fn new(
        globals: &'a G,
        package: PackageId,
        env: &'a mut Env,
        out: &'a mut dyn Receiver,
    ) -> Self {
        Self {
            stack: Vec::new(),
            vals: Vec::new(),
            package,
            globals,
            env,
            out,
            call_stack: CallStack::default(),
        }
    }

    fn pop_cont(&mut self) -> Option<Cont<'a>> {
        self.stack.pop()
    }

    fn push_action(&mut self, action: Action<'a>) {
        self.stack.push(Cont::Action(action));
    }

    fn push_expr(&mut self, expr: &'a Expr) {
        self.stack.push(Cont::Expr(expr));
    }

    fn push_frame(&mut self, span: Option<Span>, id: GlobalId, functor: FunctorApp) {
        self.call_stack.push_frame(Frame {
            span,
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

    fn push_scope(&mut self) {
        self.env.0.push(Scope::default());
        self.stack.push(Cont::Scope);
    }

    fn leave_scope(&mut self) {
        self.env
            .0
            .pop()
            .expect("scope should be entered first before leaving");
    }

    fn push_stmt(&mut self, stmt: &'a Stmt) {
        self.stack.push(Cont::Stmt(stmt));
    }

    fn push_block(&mut self, block: &'a Block) {
        self.push_scope();
        for stmt in block.stmts.iter().rev() {
            self.push_stmt(stmt);
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

    pub(crate) fn eval(&mut self) -> Result<Value, (Error, CallStack)> {
        while let Some(cont) = self.pop_cont() {
            let res = match cont {
                Cont::Action(action) => self.cont_action(action),
                Cont::Expr(expr) => self.cont_expr(expr),
                Cont::Frame(len) => {
                    self.leave_frame(len);
                    Ok(())
                }
                Cont::Scope => {
                    self.leave_scope();
                    Ok(())
                }
                Cont::Stmt(stmt) => {
                    self.cont_stmt(stmt);
                    Ok(())
                }
            };
            if let Err(e) = res {
                return Err((e, self.call_stack.clone()));
            }
        }

        Ok(self.pop_val())
    }

    fn cont_expr(&mut self, expr: &'a Expr) -> Result<(), Error> {
        match &expr.kind {
            ExprKind::Array(arr) => self.cont_arr(arr),
            ExprKind::ArrayRepeat(item, size) => self.cont_arr_repeat(item, size),
            ExprKind::Assign(lhs, rhs) => self.cont_assign(lhs, rhs),
            ExprKind::AssignOp(op, lhs, rhs) => self.cont_assign_op(*op, lhs, rhs),
            ExprKind::AssignField(record, field, replace) => {
                self.cont_assign_field(record, field, replace);
            }
            ExprKind::AssignIndex(lhs, mid, rhs) => self.cont_assign_index(lhs, mid, rhs),
            ExprKind::BinOp(op, lhs, rhs) => self.cont_binop(*op, rhs, lhs),
            ExprKind::Block(block) => self.push_block(block),
            ExprKind::Call(callee_expr, args_expr) => self.cont_call(callee_expr, args_expr),
            ExprKind::Conjugate(..) => panic!("conjugate should be eliminated by passes"),
            ExprKind::Err => panic!("error expr should not be present"),
            ExprKind::Fail(fail_expr) => self.cont_fail(expr.span, fail_expr),
            ExprKind::Field(expr, field) => self.cont_field(expr, field),
            ExprKind::For(..) => panic!("for-loop should be eliminated by passes"),
            ExprKind::Hole => panic!("hole expr should be disallowed by passes"),
            ExprKind::If(cond_expr, then_block, else_expr) => {
                self.cont_if(cond_expr, then_block, else_expr.as_ref().map(AsRef::as_ref));
            }
            ExprKind::Index(arr, index) => self.cont_index(arr, index),
            ExprKind::Lambda(..) => panic!("lambda expr should be disallowed by passes"),
            ExprKind::Lit(lit) => self.push_val(lit_to_val(lit)),
            ExprKind::Range(start, step, end) => self.cont_range(start, step, end),
            ExprKind::Repeat(..) => panic!("repeat-loop should be eliminated by passes"),
            ExprKind::Return(expr) => self.cont_ret(expr),
            ExprKind::String(components) => self.cont_string(components),
            ExprKind::TernOp(op, lhs, mid, rhs) => self.cont_ternop(*op, lhs, mid, rhs),
            ExprKind::Tuple(tup) => self.cont_tup(tup),
            ExprKind::UnOp(op, expr) => self.cont_unop(*op, expr),
            ExprKind::UpdateField(record, field, replace) => {
                self.cont_update_field(record, field, replace);
            }
            ExprKind::Var(res) => {
                let val = resolve_binding(self.env, self.package, *res, expr.span)?;
                self.push_val(val);
            }
            ExprKind::While(cond_expr, block) => self.cont_while(cond_expr, block),
        }
        Ok(())
    }

    fn cont_tup(&mut self, tup: &'a Vec<Expr>) {
        self.push_action(Action::Tuple(tup.len()));
        for tup_expr in tup.iter().rev() {
            self.push_expr(tup_expr);
        }
    }

    fn cont_arr(&mut self, arr: &'a Vec<Expr>) {
        self.push_action(Action::Array(arr.len()));
        for entry in arr.iter().rev() {
            self.push_expr(entry);
        }
    }

    fn cont_arr_repeat(&mut self, item: &'a Expr, size: &'a Expr) {
        self.push_action(Action::ArrayRepeat(size.span));
        self.push_expr(size);
        self.push_expr(item);
    }

    fn cont_ret(&mut self, expr: &'a Expr) {
        self.push_action(Action::Return);
        self.push_expr(expr);
    }

    fn cont_if(&mut self, cond_expr: &'a Expr, then_block: &'a Block, else_expr: Option<&'a Expr>) {
        self.push_action(Action::If(then_block, else_expr));
        self.push_expr(cond_expr);
    }

    fn cont_fail(&mut self, span: Span, fail_expr: &'a Expr) {
        self.push_action(Action::Fail(span));
        self.push_expr(fail_expr);
    }

    fn cont_assign(&mut self, lhs: &'a Expr, rhs: &'a Expr) {
        self.push_action(Action::Assign(lhs));
        self.push_expr(rhs);
        self.push_val(Value::unit());
    }

    fn cont_assign_op(&mut self, op: BinOp, lhs: &'a Expr, rhs: &'a Expr) {
        self.push_action(Action::Assign(lhs));
        self.cont_binop(op, rhs, lhs);
        self.push_val(Value::unit());
    }

    fn cont_assign_field(&mut self, record: &'a Expr, field: &'a Field, replace: &'a Expr) {
        self.push_action(Action::Assign(record));
        self.cont_update_field(record, field, replace);
        self.push_val(Value::unit());
    }

    fn cont_assign_index(&mut self, lhs: &'a Expr, mid: &'a Expr, rhs: &'a Expr) {
        self.push_action(Action::Assign(lhs));
        self.cont_ternop(TernOp::UpdateIndex, lhs, mid, rhs);
        self.push_val(Value::unit());
    }

    fn cont_field(&mut self, expr: &'a Expr, field: &'a Field) {
        self.push_action(Action::Field(field));
        self.push_expr(expr);
    }

    fn cont_index(&mut self, arr: &'a Expr, index: &'a Expr) {
        self.push_action(Action::Index(index.span));
        self.push_expr(index);
        self.push_expr(arr);
    }

    fn cont_range(
        &mut self,
        start: &'a Option<Box<Expr>>,
        step: &'a Option<Box<Expr>>,
        end: &'a Option<Box<Expr>>,
    ) {
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

    fn cont_string(&mut self, components: &'a [StringComponent]) {
        if let [StringComponent::Lit(str)] = components {
            self.push_val(Value::String(Rc::clone(str)));
            return;
        }

        self.push_action(Action::StringConcat(components.len()));
        for component in components.iter().rev() {
            match component {
                StringComponent::Expr(expr) => self.push_expr(expr),
                StringComponent::Lit(lit) => self.push_action(Action::StringLit(lit)),
            }
        }
    }

    fn cont_while(&mut self, cond_expr: &'a Expr, block: &'a Block) {
        self.push_action(Action::While(cond_expr, block));
        self.push_expr(cond_expr);
    }

    fn cont_call(&mut self, callee: &'a Expr, args: &'a Expr) {
        self.push_action(Action::Call(callee.span, args.span));
        self.push_expr(args);
        self.push_expr(callee);
    }

    fn cont_binop(&mut self, op: BinOp, rhs: &'a Expr, lhs: &'a Expr) {
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
                self.push_action(Action::BinOp(op, rhs.span, None));
                self.push_expr(rhs);
                self.push_expr(lhs);
            }
            BinOp::AndL | BinOp::OrL => {
                self.push_action(Action::BinOp(op, rhs.span, Some(rhs)));
                self.push_expr(lhs);
            }
        }
    }

    fn cont_ternop(&mut self, op: TernOp, lhs: &'a Expr, mid: &'a Expr, rhs: &'a Expr) {
        match op {
            TernOp::Cond => {
                self.push_action(Action::TernOp(op, mid, rhs));
                self.push_expr(lhs);
            }
            TernOp::UpdateIndex => {
                self.push_action(Action::TernOp(op, mid, rhs));
                self.push_expr(lhs);
                self.push_expr(rhs);
                self.push_expr(mid);
            }
        }
    }

    fn cont_unop(&mut self, op: UnOp, expr: &'a Expr) {
        self.push_action(Action::UnOp(op));
        self.push_expr(expr);
    }

    fn cont_update_field(&mut self, record: &'a Expr, field: &'a Field, replace: &'a Expr) {
        self.push_action(Action::UpdateField(field));
        self.push_expr(replace);
        self.push_expr(record);
    }

    fn cont_stmt(&mut self, stmt: &'a Stmt) {
        match &stmt.kind {
            StmtKind::Expr(expr) => self.push_expr(expr),
            StmtKind::Item(..) => self.push_val(Value::unit()),
            StmtKind::Local(mutability, pat, expr) => {
                self.push_action(Action::Bind(pat, *mutability));
                self.push_expr(expr);
                self.push_val(Value::unit());
            }
            StmtKind::Qubit(..) => panic!("qubit use-stmt should be eliminated by passes"),
            StmtKind::Semi(expr) => {
                self.push_action(Action::Consume);
                self.push_expr(expr);
                self.push_val(Value::unit());
            }
        }
    }

    fn cont_action(&mut self, action: Action<'a>) -> Result<(), Error> {
        match action {
            Action::Array(len) => self.eval_arr(len),
            Action::ArrayRepeat(span) => self.eval_arr_repeat(span)?,
            Action::Assign(lhs) => self.eval_assign(lhs)?,
            Action::BinOp(op, span, rhs) => self.eval_binop(op, span, rhs)?,
            Action::Bind(pat, mutability) => self.eval_bind(pat, mutability),
            Action::Call(callee_span, args_span) => self.eval_call(callee_span, args_span)?,
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
            Action::If(then_block, else_expr) => self.eval_if(then_block, else_expr),
            Action::Index(span) => self.eval_index(span)?,
            Action::Range(has_start, has_step, has_end) => {
                self.eval_range(has_start, has_step, has_end);
            }
            Action::Return => self.eval_ret(),
            Action::StringConcat(len) => self.eval_string_concat(len),
            Action::StringLit(str) => self.push_val(Value::String(Rc::clone(str))),
            Action::TernOp(op, mid, rhs) => self.eval_ternop(op, mid, rhs)?,
            Action::Tuple(len) => self.eval_tup(len),
            Action::UnOp(op) => self.eval_unop(op),
            Action::UpdateField(field) => self.eval_update_field(field),
            Action::While(cond_expr, block) => self.eval_while(cond_expr, block),
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
            Err(_) => Err(Error::Count(size_val, span)),
        }?;
        self.push_val(Value::Array(vec![item_val; s].into()));
        Ok(())
    }

    fn eval_assign(&mut self, lhs: &'a Expr) -> Result<(), Error> {
        let rhs = self.pop_val();
        update_binding(self.env, lhs, rhs)
    }

    fn eval_bind(&mut self, pat: &'a Pat, mutability: Mutability) {
        let val = self.pop_val();
        bind_value(self.env, pat, val, mutability);
    }

    fn eval_binop(&mut self, op: BinOp, span: Span, rhs: Option<&'a Expr>) -> Result<(), Error> {
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

    fn eval_call(&mut self, callee_span: Span, args_span: Span) -> Result<(), Error> {
        let args_val = self.pop_val();
        let call_val = self.pop_val();
        let (call, functor) = value_to_call_id(&call_val);
        let decl = match self.globals.get(call) {
            Some(Global::Callable(decl)) => Ok(decl),
            Some(Global::Udt) => {
                self.push_val(args_val);
                return Ok(());
            }
            None => Err(Error::Unbound(callee_span)),
        }?;
        let spec = spec_from_functor_app(functor);

        self.push_frame(Some(callee_span), call, functor);

        self.push_scope();

        match (&decl.body, spec) {
            (CallableBody::Block(body_block), Spec::Body) => {
                bind_value(self.env, &decl.input, args_val, Mutability::Immutable);
                self.push_block(body_block);
                Ok(())
            }
            (CallableBody::Specs(spec_decls), spec) => {
                let spec_decl = spec_decls
                    .iter()
                    .find(|spec_decl| spec_decl.spec == spec)
                    .map_or_else(
                        || Err(Error::MissingSpec(spec, callee_span)),
                        |spec_decl| Ok(&spec_decl.body),
                    )?;
                match spec_decl {
                    SpecBody::Impl(pat, body_block) => {
                        bind_args_for_spec(
                            self.env,
                            &decl.input,
                            pat,
                            args_val,
                            functor.controlled,
                        );
                        self.push_block(body_block);
                        Ok(())
                    }
                    SpecBody::Gen(SpecGen::Intrinsic) => {
                        let val = intrinsic::call(
                            &decl.name.name,
                            callee_span,
                            args_val,
                            args_span,
                            self.out,
                        )?;
                        self.push_val(val);
                        Ok(())
                    }
                    SpecBody::Gen(_) => Err(Error::MissingSpec(spec, callee_span)),
                }
            }
            _ => Err(Error::MissingSpec(spec, callee_span)),
        }
    }

    fn eval_field(&mut self, field: &'a Field) {
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

    fn eval_if(&mut self, then_block: &'a Block, else_expr: Option<&'a Expr>) {
        if self.pop_val().unwrap_bool() {
            self.push_block(then_block);
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

    fn eval_ret(&mut self) {
        while let Some(cont) = self.pop_cont() {
            match cont {
                Cont::Frame(len) => {
                    self.leave_frame(len);
                    break;
                }
                Cont::Scope => self.leave_scope(),
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

    fn eval_ternop(&mut self, op: TernOp, mid: &'a Expr, rhs: &'a Expr) -> Result<(), Error> {
        match op {
            TernOp::Cond => {
                if self.pop_val().unwrap_bool() {
                    self.push_expr(mid);
                } else {
                    self.push_expr(rhs);
                }
            }
            TernOp::UpdateIndex => {
                let values = self.pop_val().unwrap_array();
                let update = self.pop_val();
                let index = self.pop_val().unwrap_int();
                if index < 0 {
                    return Err(Error::Negative(index, mid.span));
                }
                let i = index.as_index(mid.span)?;
                let mut values = values.iter().cloned().collect::<Vec<_>>();
                match values.get_mut(i) {
                    Some(value) => {
                        *value = update;
                    }
                    None => return Err(Error::OutOfRange(index, mid.span)),
                }
                self.push_val(Value::Array(values.into()));
            }
        }
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
                Value::Closure => panic!("closure should be disallowed by passes"),
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

    fn eval_update_field(&mut self, field: &'a Field) {
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

    fn eval_while(&mut self, cond_expr: &'a Expr, block: &'a Block) {
        if self.pop_val().unwrap_bool() {
            self.cont_while(cond_expr, block);
            self.push_action(Action::Consume);
            self.push_val(Value::unit());
            self.push_block(block);
        } else {
            self.push_val(Value::unit());
        }
    }
}

fn bind_value(env: &mut Env, pat: &Pat, val: Value, mutability: Mutability) {
    match &pat.kind {
        PatKind::Bind(variable) => {
            let scope = env.0.last_mut().expect("binding should have a scope");
            match scope.bindings.entry(variable.id) {
                Entry::Vacant(entry) => entry.insert(Variable {
                    value: val,
                    mutability,
                }),
                Entry::Occupied(_) => panic!("duplicate binding"),
            };
        }
        PatKind::Discard => {}
        PatKind::Elided => panic!("elision used in binding"),
        PatKind::Tuple(tup) => {
            let val_tup = val.unwrap_tuple();
            for (pat, val) in tup.iter().zip(val_tup.iter()) {
                bind_value(env, pat, val.clone(), mutability);
            }
        }
    }
}

fn resolve_binding(
    env: &mut Env,
    package: PackageId,
    res: Res,
    span: Span,
) -> Result<Value, Error> {
    Ok(match res {
        Res::Err => panic!("resolution error"),
        Res::Item(item) => Value::Global(
            GlobalId {
                package: item.package.unwrap_or(package),
                item: item.item,
            },
            FunctorApp::default(),
        ),
        Res::Local(node) => env.get(node).ok_or(Error::Unbound(span))?.value.clone(),
    })
}

#[allow(clippy::similar_names)]
fn update_binding(env: &mut Env, lhs: &Expr, rhs: Value) -> Result<(), Error> {
    match (&lhs.kind, rhs) {
        (ExprKind::Hole, _) => {}
        (&ExprKind::Var(Res::Local(node)), rhs) => match env.get_mut(node) {
            Some(var) if var.is_mutable() => {
                var.value = rhs;
            }
            Some(_) => return Err(Error::Mutability(lhs.span)),
            None => return Err(Error::Unbound(lhs.span)),
        },
        (ExprKind::Tuple(var_tup), Value::Tuple(tup)) => {
            for (expr, val) in var_tup.iter().zip(tup.iter()) {
                update_binding(env, expr, val.clone())?;
            }
        }
        _ => return Err(Error::Unassignable(lhs.span)),
    }
    Ok(())
}

fn bind_args_for_spec(
    env: &mut Env,
    decl_pat: &Pat,
    spec_pat: &Pat,
    args_val: Value,
    ctl_count: u8,
) {
    match &spec_pat.kind {
        PatKind::Bind(_) | PatKind::Discard => {
            panic!("spec pattern should be elided or elided tuple, found bind/discard")
        }
        PatKind::Elided => bind_value(env, decl_pat, args_val, Mutability::Immutable),
        PatKind::Tuple(pats) => {
            assert_eq!(pats.len(), 2, "spec pattern tuple should have 2 elements");
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

            bind_value(
                env,
                &pats[0],
                Value::Array(ctls.into()),
                Mutability::Immutable,
            );
            bind_value(env, decl_pat, tup, Mutability::Immutable);
        }
    }
}

fn spec_from_functor_app(functor: FunctorApp) -> Spec {
    match (functor.adjoint, functor.controlled) {
        (false, 0) => Spec::Body,
        (true, 0) => Spec::Adj,
        (false, _) => Spec::Ctl,
        (true, _) => Spec::CtlAdj,
    }
}

fn value_to_call_id(val: &Value) -> (GlobalId, FunctorApp) {
    match val {
        Value::Closure => panic!("closure not supported"),
        Value::Global(global, functor) => (*global, *functor),
        _ => panic!("value is not call id"),
    }
}

fn lit_to_val(lit: &Lit) -> Value {
    match lit {
        Lit::BigInt(v) => Value::BigInt(v.clone()),
        Lit::Bool(v) => Value::Bool(*v),
        Lit::Double(v) => Value::Double(*v),
        Lit::Int(v) => Value::Int(*v),
        Lit::Pauli(v) => Value::Pauli(*v),
        Lit::Result(hir::Result::Zero) => Value::Result(false),
        Lit::Result(hir::Result::One) => Value::Result(true),
    }
}

fn index_array(arr: &[Value], index: i64, span: Span) -> Result<Value, Error> {
    let i = index.as_index(span)?;
    match arr.get(i) {
        Some(v) => Ok(v.clone()),
        None => Err(Error::OutOfRange(index, span)),
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
                Err(Error::Negative(rhs_val, rhs_span))
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
                Err(Error::Negative(rhs_val, rhs_span))
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
