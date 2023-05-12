// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

pub mod debug;
mod intrinsic;
pub mod output;
pub mod val;

use crate::val::{ConversionError, FunctorApp, Value};
use debug::{CallStack, Frame};
use intrinsic::invoke_intrinsic;
use miette::Diagnostic;
use num_bigint::BigInt;
use output::Receiver;
use qir_backend::{
    __quantum__rt__initialize, __quantum__rt__qubit_allocate, __quantum__rt__qubit_release,
    qubit_is_zero,
};
use qsc_data_structures::span::Span;
use qsc_hir::hir::{
    self, BinOp, Block, CallableBody, CallableDecl, Expr, ExprKind, Field, Functor, Lit,
    Mutability, NodeId, PackageId, Pat, PatKind, PrimField, QubitInit, QubitInitKind, Res, Spec,
    SpecBody, SpecGen, Stmt, StmtKind, StringComponent, TernOp, UnOp,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Write,
    mem::take,
    ops::{
        ControlFlow::{self, Break, Continue},
        Neg,
    },
    ptr::null_mut,
    rc::Rc,
};
use thiserror::Error;
use val::{GlobalId, Qubit};

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

    #[error("{0} type does not support equality comparison")]
    Equality(&'static str, #[label("does not support comparison")] Span),

    #[error("value cannot be used as an index: {0}")]
    IndexVal(i64, #[label("invalid index")] Span),

    #[error("integer too large for operation")]
    IntTooLarge(i64, #[label("this value is too large")] Span),

    #[error("missing specialization: {0}")]
    MissingSpec(Spec, #[label("callable has no {0} specialization")] Span),

    #[error("reassigning immutable variable")]
    Mutability(#[label("variable declared as immutable")] Span),

    #[error("iterable ranges cannot be open-ended")]
    OpenEnded(#[label("open-ended range used as iterator")] Span),

    #[error("index out of range: {0}")]
    OutOfRange(i64, #[label("out of range")] Span),

    #[error("negative integers cannot be used here: {0}")]
    Negative(i64, #[label("invalid negative integer")] Span),

    #[error("type {0} is not iterable")]
    NotIterable(&'static str, #[label("not iterable")] Span),

    #[error("output failure")]
    Output(#[label("failed to generate output")] Span),

    #[error("range with step size of zero")]
    RangeStepZero(#[label("invalid range")] Span),

    #[error("Qubit{0} released while not in |0⟩ state")]
    ReleasedQubitNotZero(usize, #[label] Span),

    #[error("mismatched types")]
    Type(
        &'static str,
        &'static str,
        #[label("expected {0}, found {1}")] Span,
    ),

    #[error("mismatched tuples")]
    TupleArity(
        usize,
        usize,
        #[label("expected {0}-tuple, found {1}-tuple")] Span,
    ),

    #[error("invalid left-hand side of assignment")]
    #[diagnostic(help("the left-hand side must be a variable or tuple of variables"))]
    Unassignable(#[label("not assignable")] Span),

    #[error("symbol is not bound")]
    Unbound(#[label] Span),

    #[error("{0} support is not implemented")]
    #[diagnostic(help("this language feature is not yet supported"))]
    Unimplemented(&'static str, #[label("cannot evaluate this")] Span),

    #[error("unknown intrinsic")]
    UnknownIntrinsic(#[label("callable has no implementation")] Span),

    #[error("program failed: {0}")]
    UserFail(String, #[label("explicit fail")] Span),
}

#[derive(Debug)]
enum Reason {
    Error(Error),
    Return(Value),
}

trait WithSpan {
    type Output;

    fn with_span(self, span: Span) -> Self::Output;
}

impl<T> WithSpan for Result<T, ConversionError> {
    type Output = ControlFlow<Reason, T>;

    fn with_span(self, span: Span) -> Self::Output {
        match self {
            Ok(c) => Continue(c),
            Err(e) => Break(Reason::Error(Error::Type(e.expected, e.actual, span))),
        }
    }
}

trait AsIndex {
    type Output;

    fn as_index(&self, span: Span) -> Self::Output;
}

impl AsIndex for i64 {
    type Output = ControlFlow<Reason, usize>;

    fn as_index(&self, span: Span) -> ControlFlow<Reason, usize> {
        match (*self).try_into() {
            Ok(index) => Continue(index),
            Err(_) => Break(Reason::Error(Error::IndexVal(*self, span))),
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

pub trait GlobalLookup<'a> {
    fn get(&self, id: GlobalId) -> Option<Global<'a>>;
}

impl<'a, F: Fn(GlobalId) -> Option<Global<'a>>> GlobalLookup<'a> for F {
    fn get(&self, id: GlobalId) -> Option<Global<'a>> {
        self(id)
    }
}

pub enum Global<'a> {
    Callable(&'a CallableDecl),
    Udt,
}

/// Evaluates the given statement with the given context.
/// # Errors
/// Returns the first error encountered during execution.
pub fn eval_stmt<'a>(
    stmt: &Stmt,
    globals: &'a impl GlobalLookup<'a>,
    package: PackageId,
    env: &mut Env,
    out: &'a mut dyn Receiver,
) -> Result<Value, (Error, CallStack)> {
    let mut eval = Evaluator {
        globals,
        package,
        env: take(env),
        call_stack: CallStack::default(),
        out: Some(out),
    };
    let res = match eval.eval_stmt(stmt) {
        Continue(res) | Break(Reason::Return(res)) => Ok(res),
        Break(Reason::Error(error)) => Err((error, eval.call_stack.clone())),
    };
    *env = take(&mut eval.env);
    res
}

/// Evaluates the given expression with the given context.
/// # Errors
/// Returns the first error encountered during execution.
pub fn eval_expr<'a>(
    expr: &Expr,
    globals: &'a impl GlobalLookup<'a>,
    package: PackageId,
    env: &mut Env,
    out: &'a mut dyn Receiver,
) -> Result<Value, (Error, CallStack)> {
    let mut eval = Evaluator {
        globals,
        package,
        env: take(env),
        call_stack: CallStack::default(),
        out: Some(out),
    };
    let res = match eval.eval_expr(expr) {
        Continue(res) | Break(Reason::Return(res)) => Ok(res),
        Break(Reason::Error(error)) => Err((error, eval.call_stack.clone())),
    };
    *env = take(&mut eval.env);
    res
}

pub fn init() {
    __quantum__rt__initialize(null_mut());
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
    qubits: Vec<(Qubit, Span)>,
}

impl Env {
    #[must_use]
    pub fn with_empty_scope() -> Self {
        Self(vec![Scope::default()])
    }
}

struct Evaluator<'a, G> {
    globals: &'a G,
    package: PackageId,
    env: Env,
    call_stack: CallStack,
    out: Option<&'a mut dyn Receiver>,
}

impl<'a, G: GlobalLookup<'a>> Evaluator<'a, G> {
    #[allow(clippy::too_many_lines)]
    fn eval_expr(&mut self, expr: &Expr) -> ControlFlow<Reason, Value> {
        match &expr.kind {
            ExprKind::Array(arr) => {
                let mut val_arr = vec![];
                for expr in arr {
                    val_arr.push(self.eval_expr(expr)?);
                }
                Continue(Value::Array(val_arr.into()))
            }
            ExprKind::ArrayRepeat(item, size) => {
                let item_val = self.eval_expr(item)?;
                let size_val: i64 = self.eval_expr(size)?.try_into().with_span(size.span)?;
                let s = match size_val.try_into() {
                    Ok(i) => Continue(i),
                    Err(_) => Break(Reason::Error(Error::Count(size_val, size.span))),
                }?;
                Continue(Value::Array(vec![item_val; s].into()))
            }
            ExprKind::Assign(lhs, rhs) => {
                let val = self.eval_expr(rhs)?;
                self.update_binding(lhs, val)
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                let update = self.eval_binop(*op, lhs, rhs)?;
                self.update_binding(lhs, update)
            }
            ExprKind::AssignUpdate(lhs, mid, rhs) => {
                let update = self.eval_ternop_update(lhs, mid, rhs)?;
                self.update_binding(lhs, update)
            }
            ExprKind::BinOp(op, lhs, rhs) => self.eval_binop(*op, lhs, rhs),
            ExprKind::Block(block) => self.eval_block(block),
            ExprKind::Call(call, args) => self.eval_call(call, args),
            ExprKind::Fail(msg) => {
                let msg = self.eval_expr(msg)?.try_into_string().with_span(msg.span)?;
                Break(Reason::Error(Error::UserFail(msg.to_string(), expr.span)))
            }
            ExprKind::Field(record, field) => self.eval_field(record, field),
            ExprKind::For(pat, expr, block) => self.eval_for_loop(pat, expr, block),
            ExprKind::If(cond, then, els) => {
                if self.eval_expr(cond)?.try_into().with_span(cond.span)? {
                    self.eval_block(then)
                } else if let Some(els) = els {
                    self.eval_expr(els)
                } else {
                    Continue(Value::unit())
                }
            }
            ExprKind::Index(arr, index_expr) => {
                let arr = self.eval_expr(arr)?.try_into_array().with_span(arr.span)?;
                let index_val = self.eval_expr(index_expr)?;
                match &index_val {
                    Value::Int(index) => index_array(&arr, *index, index_expr.span),
                    &Value::Range(start, step, end) => {
                        slice_array(&arr, start, step, end, index_expr.span)
                    }
                    _ => Break(Reason::Error(Error::Type(
                        "Int or Range",
                        index_val.type_name(),
                        index_expr.span,
                    ))),
                }
            }
            ExprKind::Lit(lit) => Continue(lit_to_val(lit)),
            ExprKind::Range(start, step, end) => self.eval_range(start, step, end),
            ExprKind::Repeat(repeat, cond, fixup) => self.eval_repeat_loop(repeat, cond, fixup),
            ExprKind::Return(expr) => Break(Reason::Return(self.eval_expr(expr)?)),
            ExprKind::String(components) => self.eval_string(components),
            ExprKind::TernOp(ternop, lhs, mid, rhs) => match *ternop {
                TernOp::Cond => self.eval_ternop_cond(lhs, mid, rhs),
                TernOp::Update => self.eval_ternop_update(lhs, mid, rhs),
            },
            ExprKind::Tuple(tup) => {
                let mut val_tup = vec![];
                for expr in tup {
                    val_tup.push(self.eval_expr(expr)?);
                }
                Continue(Value::Tuple(val_tup.into()))
            }
            ExprKind::UnOp(op, rhs) => self.eval_unop(expr, *op, rhs),
            &ExprKind::Var(res) => match self.resolve_binding(res, expr.span) {
                Ok(val) => Continue(val),
                Err(e) => Break(Reason::Error(e)),
            },
            ExprKind::While(cond, block) => {
                while self.eval_expr(cond)?.try_into().with_span(cond.span)? {
                    self.eval_block(block)?;
                }
                Continue(Value::unit())
            }
            ExprKind::Conjugate(..) => {
                Break(Reason::Error(Error::Unimplemented("conjugate", expr.span)))
            }
            ExprKind::Err => Break(Reason::Error(Error::Unimplemented("error", expr.span))),
            ExprKind::Hole => Break(Reason::Error(Error::Unimplemented("hole", expr.span))),
            ExprKind::Lambda(..) => Break(Reason::Error(Error::Unimplemented("lambda", expr.span))),
        }
    }

    fn eval_range(
        &mut self,
        start: &Option<Box<Expr>>,
        step: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
    ) -> ControlFlow<Reason, Value> {
        let mut to_opt_i64 = |e: &Option<Box<Expr>>| match e {
            Some(expr) => Continue(Some(self.eval_expr(expr)?.try_into().with_span(expr.span)?)),
            None => Continue(None),
        };
        Continue(Value::Range(
            to_opt_i64(start)?,
            to_opt_i64(step)?.unwrap_or(val::DEFAULT_RANGE_STEP),
            to_opt_i64(end)?,
        ))
    }

    fn eval_string(&mut self, components: &[StringComponent]) -> ControlFlow<Reason, Value> {
        if let [StringComponent::Lit(str)] = components {
            return Continue(Value::String(Rc::clone(str)));
        }

        let mut string = String::new();
        for component in components {
            match component {
                StringComponent::Expr(expr) => {
                    let value = self.eval_expr(expr)?;
                    write!(string, "{value}").expect("string should be writable");
                }
                StringComponent::Lit(lit) => string += lit,
            }
        }

        Continue(Value::String(string.into()))
    }

    fn eval_block(&mut self, block: &Block) -> ControlFlow<Reason, Value> {
        self.enter_scope();
        let result = if let Some((last, most)) = block.stmts.split_last() {
            for stmt in most {
                self.eval_stmt(stmt)?;
            }
            self.eval_stmt(last)
        } else {
            Continue(Value::unit())
        };
        self.leave_scope()?;
        result
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> ControlFlow<Reason, Value> {
        match &stmt.kind {
            StmtKind::Item(_) => Continue(Value::unit()),
            StmtKind::Expr(expr) => self.eval_expr(expr),
            StmtKind::Local(mutability, pat, expr) => {
                let val = self.eval_expr(expr)?;
                self.bind_value(pat, val, expr.span, *mutability)?;
                Continue(Value::unit())
            }
            StmtKind::Semi(expr) => {
                self.eval_expr(expr)?;
                Continue(Value::unit())
            }
            StmtKind::Qubit(_, pat, qubit_init, block) => {
                let (qubit_val, qubits) = self.eval_qubit_init(qubit_init)?;
                if let Some(block) = block {
                    self.enter_scope();
                    self.track_qubits(qubits);
                    self.bind_value(pat, qubit_val, stmt.span, Mutability::Immutable)?;
                    self.eval_block(block)?;
                    self.leave_scope()?;
                } else {
                    self.track_qubits(qubits);
                    self.bind_value(pat, qubit_val, stmt.span, Mutability::Immutable)?;
                }
                Continue(Value::unit())
            }
        }
    }

    fn eval_for_loop(
        &mut self,
        pat: &Pat,
        expr: &Expr,
        block: &Block,
    ) -> ControlFlow<Reason, Value> {
        match self.eval_expr(expr)? {
            Value::Array(arr) => self.iterate_for_loop(pat, arr.iter().cloned(), expr.span, block),
            Value::Range(start, step, end) => {
                let start =
                    start.map_or(Break(Reason::Error(Error::OpenEnded(expr.span))), Continue)?;
                let end =
                    end.map_or(Break(Reason::Error(Error::OpenEnded(expr.span))), Continue)?;
                let range = Range::new(start, step, end);
                self.iterate_for_loop(pat, range.map(Value::Int), expr.span, block)
            }
            value => Break(Reason::Error(Error::NotIterable(
                value.type_name(),
                expr.span,
            ))),
        }
    }

    fn iterate_for_loop(
        &mut self,
        pat: &Pat,
        values: impl Iterator<Item = Value>,
        span: Span,
        block: &Block,
    ) -> ControlFlow<Reason, Value> {
        for value in values {
            self.enter_scope();
            self.bind_value(pat, value, span, Mutability::Immutable);
            self.eval_block(block)?;
            self.leave_scope()?;
        }

        Continue(Value::unit())
    }

    fn eval_repeat_loop(
        &mut self,
        repeat: &Block,
        cond: &Expr,
        fixup: &Option<Block>,
    ) -> ControlFlow<Reason, Value> {
        self.enter_scope();

        for stmt in &repeat.stmts {
            self.eval_stmt(stmt)?;
        }
        while !self.eval_expr(cond)?.try_into().with_span(cond.span)? {
            if let Some(block) = fixup.as_ref() {
                self.eval_block(block)?;
            }

            self.leave_scope()?;
            self.enter_scope();

            for stmt in &repeat.stmts {
                self.eval_stmt(stmt)?;
            }
        }

        self.leave_scope()?;
        Continue(Value::unit())
    }

    fn eval_qubit_init(
        &mut self,
        qubit_init: &QubitInit,
    ) -> ControlFlow<Reason, (Value, Vec<(Qubit, Span)>)> {
        match &qubit_init.kind {
            QubitInitKind::Array(count) => {
                let count_val: i64 = self.eval_expr(count)?.try_into().with_span(count.span)?;
                let count: usize = match count_val.try_into() {
                    Ok(i) => Continue(i),
                    Err(_) => Break(Reason::Error(Error::Count(count_val, count.span))),
                }?;
                let mut arr = vec![];
                arr.resize_with(count, || {
                    (Qubit(__quantum__rt__qubit_allocate()), qubit_init.span)
                });

                Continue((
                    Value::Array(arr.iter().copied().map(|q| Value::Qubit(q.0)).collect()),
                    arr,
                ))
            }
            QubitInitKind::Single => {
                let qubit = Qubit(__quantum__rt__qubit_allocate());
                Continue((Value::Qubit(qubit), vec![(qubit, qubit_init.span)]))
            }
            QubitInitKind::Tuple(tup) => {
                let mut tup_vec = vec![];
                let mut qubit_vec = vec![];
                for init in tup {
                    let (t, mut v) = self.eval_qubit_init(init)?;
                    tup_vec.push(t);
                    qubit_vec.append(&mut v);
                }
                Continue((Value::Tuple(tup_vec.into()), qubit_vec))
            }
        }
    }

    fn eval_call(&mut self, callee: &Expr, args: &Expr) -> ControlFlow<Reason, Value> {
        let callee_val = self.eval_expr(callee)?;
        let (callee_id, functor) = value_to_call_id(&callee_val, callee.span)?;
        let spec = spec_from_functor_app(functor);
        let args_val = self.eval_expr(args)?;
        let decl = match self.globals.get(callee_id) {
            Some(Global::Callable(decl)) => Continue(decl),
            Some(Global::Udt) => return Continue(args_val),
            None => Break(Reason::Error(Error::Unbound(callee.span))),
        }?;

        self.push_frame(Frame {
            id: callee_id,
            span: Some(callee.span),
            caller: self.package,
            functor,
        });

        let mut new_self = Self {
            globals: self.globals,
            package: callee_id.package,
            env: Env::default(),
            out: self.out.take(),
            call_stack: CallStack::default(),
        };
        let call_res = new_self.eval_call_spec(
            decl,
            spec,
            args_val,
            args.span,
            callee.span,
            functor.controlled,
        );
        self.out = new_self.out.take();

        match call_res {
            Break(Reason::Return(val)) => {
                self.pop_frame();
                Continue(val)
            }
            Break(Reason::Error(_)) => {
                for frame in new_self.call_stack.clone().into_frames() {
                    self.call_stack.push_frame(frame);
                }

                call_res
            }
            Continue(_) => {
                self.pop_frame();
                call_res
            }
        }
    }

    fn eval_call_spec(
        &mut self,
        decl: &CallableDecl,
        spec: Spec,
        args_val: Value,
        args_span: Span,
        call_span: Span,
        ctl_count: u8,
    ) -> ControlFlow<Reason, Value> {
        self.enter_scope();
        let res = match (&decl.body, spec) {
            (CallableBody::Block(body_block), Spec::Body) => {
                self.bind_value(&decl.input, args_val, args_span, Mutability::Immutable)?;
                self.eval_block(body_block)
            }
            (CallableBody::Specs(spec_decls), spec) => {
                let spec_decl = spec_decls
                    .iter()
                    .find(|spec_decl| spec_decl.spec == spec)
                    .map_or_else(
                        || Break(Reason::Error(Error::MissingSpec(spec, call_span))),
                        |spec_decl| Continue(&spec_decl.body),
                    )?;
                match spec_decl {
                    SpecBody::Impl(pat, body_block) => {
                        self.bind_args_for_spec(&decl.input, pat, args_val, args_span, ctl_count)?;
                        self.eval_block(body_block)
                    }
                    SpecBody::Gen(SpecGen::Intrinsic) => invoke_intrinsic(
                        &decl.name.name,
                        call_span,
                        args_val,
                        args_span,
                        self.out
                            .as_deref_mut()
                            .expect("output receiver should be set"),
                    ),
                    SpecBody::Gen(_) => Break(Reason::Error(Error::MissingSpec(spec, call_span))),
                }
            }
            _ => Break(Reason::Error(Error::MissingSpec(spec, call_span))),
        };
        self.leave_scope()?;
        res
    }

    fn bind_args_for_spec(
        &mut self,
        decl_pat: &Pat,
        spec_pat: &Pat,
        args_val: Value,
        args_span: Span,
        ctl_count: u8,
    ) -> ControlFlow<Reason, ()> {
        match &spec_pat.kind {
            PatKind::Bind(_) | PatKind::Discard => {
                panic!("spec pattern should be elided or elided tuple, found bind/discard")
            }
            PatKind::Elided => {
                self.bind_value(decl_pat, args_val, args_span, Mutability::Immutable)
            }
            PatKind::Tuple(pats) => {
                assert_eq!(pats.len(), 2, "spec pattern tuple should have 2 elements");
                assert!(
                    ctl_count > 0,
                    "spec pattern tuple used without controlled functor"
                );

                let mut tup = args_val;
                let mut ctls = vec![];
                for _ in 0..ctl_count {
                    let tup_nesting = tup.try_into_tuple().with_span(args_span)?;
                    if tup_nesting.len() != 2 {
                        return Break(Reason::Error(Error::TupleArity(
                            2,
                            tup_nesting.len(),
                            args_span,
                        )));
                    }

                    let c = tup_nesting[0].clone();
                    let rest = tup_nesting[1].clone();
                    ctls.extend_from_slice(c.try_into_array().with_span(args_span)?.as_ref());
                    tup = rest;
                }

                self.bind_value(
                    &pats[0],
                    Value::Array(ctls.into()),
                    args_span,
                    Mutability::Immutable,
                )?;
                self.bind_value(decl_pat, tup, args_span, Mutability::Immutable)
            }
        }
    }

    fn eval_unop(&mut self, expr: &Expr, op: UnOp, rhs: &Expr) -> ControlFlow<Reason, Value> {
        let val = self.eval_expr(rhs)?;
        match op {
            UnOp::Neg => match val {
                Value::BigInt(v) => Continue(Value::BigInt(v.neg())),
                Value::Double(v) => Continue(Value::Double(v.neg())),
                Value::Int(v) => Continue(Value::Int(v.wrapping_neg())),
                _ => Break(Reason::Error(Error::Type(
                    "Int, BigInt, or Double",
                    val.type_name(),
                    rhs.span,
                ))),
            },
            UnOp::Pos => match val {
                Value::BigInt(_) | Value::Int(_) | Value::Double(_) => Continue(val),
                _ => Break(Reason::Error(Error::Type(
                    "Int, BigInt, or Double",
                    val.type_name(),
                    rhs.span,
                ))),
            },
            UnOp::NotL => match val {
                Value::Bool(b) => Continue(Value::Bool(!b)),
                _ => Break(Reason::Error(Error::Type(
                    "Bool",
                    val.type_name(),
                    rhs.span,
                ))),
            },
            UnOp::NotB => match val {
                Value::Int(v) => Continue(Value::Int(!v)),
                Value::BigInt(v) => Continue(Value::BigInt(!v)),
                _ => Break(Reason::Error(Error::Type(
                    "Int or BigInt",
                    val.type_name(),
                    rhs.span,
                ))),
            },
            UnOp::Functor(functor) => match val {
                Value::Closure => Break(Reason::Error(Error::Unimplemented("closure", expr.span))),
                Value::Global(id, app) => {
                    Continue(Value::Global(id, update_functor_app(functor, app)))
                }
                _ => Break(Reason::Error(Error::Type(
                    "Callable",
                    val.type_name(),
                    rhs.span,
                ))),
            },
            UnOp::Unwrap => Break(Reason::Error(Error::Unimplemented("unwrap", expr.span))),
        }
    }

    fn eval_binop(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr) -> ControlFlow<Reason, Value> {
        let lhs_val = self.eval_expr(lhs)?;
        match op {
            BinOp::Add => eval_binop_add(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::AndB => eval_binop_andb(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::AndL => self.eval_binop_andl(lhs_val.try_into().with_span(lhs.span)?, rhs),
            BinOp::Div => eval_binop_div(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::Eq => eval_binop_eq(&lhs_val, lhs.span, &self.eval_expr(rhs)?, rhs.span),
            BinOp::Exp => eval_binop_exp(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::Gt => eval_binop_gt(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::Gte => eval_binop_gte(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::Lt => eval_binop_lt(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::Lte => eval_binop_lte(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::Mod => eval_binop_mod(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::Mul => eval_binop_mul(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::Neq => eval_binop_neq(&lhs_val, lhs.span, &self.eval_expr(rhs)?, rhs.span),
            BinOp::OrB => eval_binop_orb(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::OrL => self.eval_binop_orl(lhs_val.try_into().with_span(lhs.span)?, rhs),
            BinOp::Shl => eval_binop_shl(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::Shr => eval_binop_shr(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::Sub => eval_binop_sub(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
            BinOp::XorB => eval_binop_xorb(lhs_val, lhs.span, self.eval_expr(rhs)?, rhs.span),
        }
    }

    fn eval_binop_andl(&mut self, lhs: bool, rhs: &Expr) -> ControlFlow<Reason, Value> {
        Continue(Value::Bool(
            lhs && self.eval_expr(rhs)?.try_into().with_span(rhs.span)?,
        ))
    }

    fn eval_binop_orl(&mut self, lhs: bool, rhs: &Expr) -> ControlFlow<Reason, Value> {
        Continue(Value::Bool(
            lhs || self.eval_expr(rhs)?.try_into().with_span(rhs.span)?,
        ))
    }

    fn eval_ternop_cond(
        &mut self,
        lhs: &Expr,
        mid: &Expr,
        rhs: &Expr,
    ) -> ControlFlow<Reason, Value> {
        if self.eval_expr(lhs)?.try_into().with_span(lhs.span)? {
            self.eval_expr(mid)
        } else {
            self.eval_expr(rhs)
        }
    }

    fn eval_ternop_update(
        &mut self,
        lhs: &Expr,
        mid: &Expr,
        rhs: &Expr,
    ) -> ControlFlow<Reason, Value> {
        let values = self.eval_expr(lhs)?.try_into_array().with_span(lhs.span)?;
        let index: i64 = self.eval_expr(mid)?.try_into().with_span(mid.span)?;
        if index < 0 {
            return Break(Reason::Error(Error::Negative(index, mid.span)));
        }

        let mut values: Vec<_> = values.iter().cloned().collect();
        match values.get_mut(index.as_index(mid.span)?) {
            Some(value) => {
                *value = self.eval_expr(rhs)?;
                Continue(Value::Array(values.into()))
            }
            None => Break(Reason::Error(Error::OutOfRange(index, mid.span))),
        }
    }

    fn eval_field(&mut self, record: &Expr, field: &Field) -> ControlFlow<Reason, Value> {
        match (self.eval_expr(record)?, field) {
            (Value::Array(arr), Field::Prim(PrimField::Length)) => {
                let len: i64 = match arr.len().try_into() {
                    Ok(len) => Continue(len),
                    Err(_) => Break(Reason::Error(Error::ArrayTooLarge(record.span))),
                }?;
                Continue(Value::Int(len))
            }
            (Value::Range(Some(start), _, _), Field::Prim(PrimField::Start)) => {
                Continue(Value::Int(start))
            }
            (Value::Range(_, step, _), Field::Prim(PrimField::Step)) => Continue(Value::Int(step)),
            (Value::Range(_, _, Some(end)), Field::Prim(PrimField::End)) => {
                Continue(Value::Int(end))
            }
            (mut value, Field::Path(path)) => {
                for &index in path {
                    let Value::Tuple(items) = value else { panic!("field path on non-tuple value"); };
                    value = items[index].clone();
                }
                Continue(value)
            }
            _ => panic!("invalid field access"),
        }
    }

    fn enter_scope(&mut self) {
        self.env.0.push(Scope::default());
    }

    fn track_qubits(&mut self, mut qubits: Vec<(Qubit, Span)>) {
        self.env
            .0
            .last_mut()
            .expect("scope should have been entered to track qubits")
            .qubits
            .append(&mut qubits);
    }

    fn leave_scope(&mut self) -> ControlFlow<Reason, ()> {
        for (qubit, span) in self
            .env
            .0
            .pop()
            .expect("scope should be entered first before leaving")
            .qubits
        {
            if !qubit_is_zero(qubit.0) {
                return ControlFlow::Break(Reason::Error(Error::ReleasedQubitNotZero(
                    qubit.0 as usize,
                    span,
                )));
            }
            __quantum__rt__qubit_release(qubit.0);
        }
        ControlFlow::Continue(())
    }

    fn bind_value(
        &mut self,
        pat: &Pat,
        value: Value,
        span: Span,
        mutability: Mutability,
    ) -> ControlFlow<Reason, ()> {
        match &pat.kind {
            PatKind::Bind(variable) => {
                let scope = self.env.0.last_mut().expect("binding should have a scope");
                match scope.bindings.entry(variable.id) {
                    Entry::Vacant(entry) => entry.insert(Variable { value, mutability }),
                    Entry::Occupied(_) => panic!("duplicate binding"),
                };
                Continue(())
            }
            PatKind::Discard => Continue(()),
            PatKind::Elided => panic!("elision used in binding"),
            PatKind::Tuple(tup) => {
                let val_tup = value.try_into_tuple().with_span(span)?;
                if val_tup.len() == tup.len() {
                    for (pat, val) in tup.iter().zip(val_tup.iter()) {
                        self.bind_value(pat, val.clone(), span, mutability)?;
                    }
                    Continue(())
                } else {
                    Break(Reason::Error(Error::TupleArity(
                        tup.len(),
                        val_tup.len(),
                        pat.span,
                    )))
                }
            }
        }
    }

    fn resolve_binding(&mut self, res: Res, span: Span) -> Result<Value, Error> {
        Ok(match res {
            Res::Err => panic!("resolution error"),
            Res::Item(item) => Value::Global(
                GlobalId {
                    package: item.package.unwrap_or(self.package),
                    item: item.item,
                },
                FunctorApp::default(),
            ),
            Res::Local(node) => self
                .env
                .get(node)
                .ok_or(Error::Unbound(span))?
                .value
                .clone(),
        })
    }

    #[allow(clippy::similar_names)]
    fn update_binding(&mut self, lhs: &Expr, rhs: Value) -> ControlFlow<Reason, Value> {
        match (&lhs.kind, rhs) {
            (ExprKind::Hole, _) => Continue(Value::unit()),
            (&ExprKind::Var(Res::Local(node)), rhs) => match self.env.get_mut(node) {
                Some(var) if var.is_mutable() => {
                    var.value = rhs;
                    Continue(Value::unit())
                }
                Some(_) => Break(Reason::Error(Error::Mutability(lhs.span))),
                None => Break(Reason::Error(Error::Unbound(lhs.span))),
            },
            (ExprKind::Tuple(var_tup), Value::Tuple(tup)) => {
                if var_tup.len() == tup.len() {
                    for (expr, val) in var_tup.iter().zip(tup.iter()) {
                        self.update_binding(expr, val.clone())?;
                    }
                    Continue(Value::unit())
                } else {
                    Break(Reason::Error(Error::TupleArity(
                        var_tup.len(),
                        tup.len(),
                        lhs.span,
                    )))
                }
            }
            _ => Break(Reason::Error(Error::Unassignable(lhs.span))),
        }
    }

    fn push_frame(&mut self, frame: Frame) {
        self.call_stack.push_frame(frame);
    }

    fn pop_frame(&mut self) {
        self.call_stack.pop_frame();
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

fn value_to_call_id(val: &Value, span: Span) -> ControlFlow<Reason, (GlobalId, FunctorApp)> {
    match val {
        Value::Closure => Break(Reason::Error(Error::Unimplemented("closure", span))),
        Value::Global(global, functor) => Continue((*global, *functor)),
        _ => Break(Reason::Error(Error::Type(
            "Callable",
            val.type_name(),
            span,
        ))),
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

fn index_array(arr: &[Value], index: i64, span: Span) -> ControlFlow<Reason, Value> {
    match arr.get(index.as_index(span)?) {
        Some(v) => Continue(v.clone()),
        None => Break(Reason::Error(Error::OutOfRange(index, span))),
    }
}

fn slice_array(
    arr: &[Value],
    start: Option<i64>,
    step: i64,
    end: Option<i64>,
    span: Span,
) -> ControlFlow<Reason, Value> {
    if step == 0 {
        Break(Reason::Error(Error::RangeStepZero(span)))
    } else {
        let len: i64 = match arr.len().try_into() {
            Ok(len) => Continue(len),
            Err(_) => Break(Reason::Error(Error::ArrayTooLarge(span))),
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

        Continue(Value::Array(slice.into()))
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

fn eval_binop_add(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::Array(arr) => {
            let rhs_arr = rhs_val.try_into_array().with_span(rhs_span)?;
            let items: Vec<_> = arr.iter().cloned().chain(rhs_arr.iter().cloned()).collect();
            Continue(Value::Array(items.into()))
        }
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::BigInt(val + rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Double(val + rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Int(val + rhs))
        }
        Value::String(val) => {
            let rhs = rhs_val.try_into_string().with_span(rhs_span)?;
            Continue(Value::String((val.to_string() + &rhs).into()))
        }
        _ => Break(Reason::Error(Error::Type(
            "Array, BigInt, Double, Int, or String",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_andb(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::BigInt(val & rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Int(val & rhs))
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_div(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            if rhs == BigInt::from(0) {
                Break(Reason::Error(Error::DivZero(rhs_span)))
            } else {
                Continue(Value::BigInt(val / rhs))
            }
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs == 0 {
                Break(Reason::Error(Error::DivZero(rhs_span)))
            } else {
                Continue(Value::Int(val / rhs))
            }
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs == 0.0 {
                Break(Reason::Error(Error::DivZero(rhs_span)))
            } else {
                Continue(Value::Double(val / rhs))
            }
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt, Double, or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn supports_eq(val: &Value, val_span: Span) -> ControlFlow<Reason, ()> {
    match val {
        Value::Closure | Value::Global(..) => {
            Break(Reason::Error(Error::Equality(val.type_name(), val_span)))
        }
        _ => Continue(()),
    }
}

fn eval_binop_eq(
    lhs_val: &Value,
    lhs_span: Span,
    rhs_val: &Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    supports_eq(lhs_val, lhs_span)?;
    if lhs_val.type_name() == rhs_val.type_name() {
        Continue(Value::Bool(lhs_val == rhs_val))
    } else {
        Break(Reason::Error(Error::Type(
            lhs_val.type_name(),
            rhs_val.type_name(),
            rhs_span,
        )))
    }
}

fn eval_binop_exp(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs_val: i64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs_val < 0 {
                Break(Reason::Error(Error::Negative(rhs_val, rhs_span)))
            } else {
                let rhs_val: u32 = match rhs_val.try_into() {
                    Ok(v) => Continue(v),
                    Err(_) => Break(Reason::Error(Error::IntTooLarge(rhs_val, rhs_span))),
                }?;
                Continue(Value::BigInt(val.pow(rhs_val)))
            }
        }
        Value::Double(val) => Continue(Value::Double(
            val.powf(rhs_val.try_into().with_span(rhs_span)?),
        )),
        Value::Int(val) => {
            let rhs_val: i64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs_val < 0 {
                Break(Reason::Error(Error::Negative(rhs_val, rhs_span)))
            } else {
                let rhs_val: u32 = match rhs_val.try_into() {
                    Ok(v) => Continue(v),
                    Err(_) => Break(Reason::Error(Error::IntTooLarge(rhs_val, rhs_span))),
                }?;
                Continue(Value::Int(val.pow(rhs_val)))
            }
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt, Double, or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_gt(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Bool(val > rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Bool(val > rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Bool(val > rhs))
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt, Double, or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_gte(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Bool(val >= rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Bool(val >= rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Bool(val >= rhs))
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt, Double, or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_lt(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Bool(val < rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Bool(val < rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Bool(val < rhs))
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt, Double, or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_lte(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Bool(val <= rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Bool(val <= rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Bool(val <= rhs))
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt, Double, or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_mod(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::BigInt(val % rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Int(val % rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Double(val % rhs))
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt, Double, or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_mul(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::BigInt(val * rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Int(val * rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Double(val * rhs))
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt, Double, or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_neq(
    lhs_val: &Value,
    lhs_span: Span,
    rhs_val: &Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    supports_eq(lhs_val, lhs_span)?;
    if lhs_val.type_name() == rhs_val.type_name() {
        Continue(Value::Bool(lhs_val != rhs_val))
    } else {
        Break(Reason::Error(Error::Type(
            lhs_val.type_name(),
            rhs_val.type_name(),
            rhs_span,
        )))
    }
}

fn eval_binop_orb(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::BigInt(val | rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Int(val | rhs))
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_shl(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs > 0 {
                Continue(Value::BigInt(val << rhs))
            } else {
                Continue(Value::BigInt(val >> rhs.abs()))
            }
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs > 0 {
                Continue(Value::Int(val << rhs))
            } else {
                Continue(Value::Int(val >> rhs.abs()))
            }
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_shr(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs > 0 {
                Continue(Value::BigInt(val >> rhs))
            } else {
                Continue(Value::BigInt(val << rhs.abs()))
            }
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs > 0 {
                Continue(Value::Int(val >> rhs))
            } else {
                Continue(Value::Int(val << rhs.abs()))
            }
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_sub(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::BigInt(val - rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Int(val - rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Double(val - rhs))
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt, Double, or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn eval_binop_xorb(
    lhs_val: Value,
    lhs_span: Span,
    rhs_val: Value,
    rhs_span: Span,
) -> ControlFlow<Reason, Value> {
    match lhs_val {
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::BigInt(val ^ rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            Continue(Value::Int(val ^ rhs))
        }
        _ => Break(Reason::Error(Error::Type(
            "BigInt or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}
