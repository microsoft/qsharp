// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

mod intrinsic;
pub mod val;

use crate::val::{ConversionError, FunctorApp, Value};
use intrinsic::invoke_intrinsic;
use miette::Diagnostic;
use num_bigint::BigInt;
use qir_backend::__quantum__rt__qubit_allocate;
use qsc_ast::ast::{
    self, BinOp, Block, CallableBody, CallableDecl, Expr, ExprKind, Functor, Lit, Mutability,
    NodeId, Pat, PatKind, QubitInit, QubitInitKind, Span, Spec, SpecBody, SpecGen, Stmt, StmtKind,
    UnOp,
};
use qsc_frontend::{
    compile::{PackageId, PackageStore},
    resolve::{DefId, PackageSrc, Resolutions},
};
use qsc_passes::globals::GlobalId;
use std::{
    collections::{hash_map::Entry, HashMap},
    ops::{ControlFlow, Neg},
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("array too large")]
    ArrayTooLarge(#[label("this array has too many items")] Span),

    #[error("invalid array length: {0}")]
    Count(i64, #[label("cannot be used as a length")] Span),

    #[error("division by zero")]
    DivZero(#[label("cannot divide by zero")] Span),

    #[error("nothing to evaluate; entry expression is empty")]
    EmptyExpr,

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

    #[error("index out of range: {0}")]
    OutOfRange(i64, #[label("out of range")] Span),

    #[error("negative integers cannot be used here: {0}")]
    Negative(i64, #[label("invalid negative integer")] Span),

    #[error("range with step size of zero")]
    RangeStepZero(#[label("invalid range")] Span),

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

    #[error("not implemented")]
    #[diagnostic(help("this language feature is not yet supported"))]
    Unimplemented(#[label("cannot evaluate this")] Span),

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
            Ok(c) => ControlFlow::Continue(c),
            Err(e) => ControlFlow::Break(Reason::Error(Error::Type(e.expected, e.actual, span))),
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
            Ok(index) => ControlFlow::Continue(index),
            Err(_) => ControlFlow::Break(Reason::Error(Error::IndexVal(*self, span))),
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

#[derive(Default)]
pub struct Env(Vec<HashMap<GlobalId, Variable>>);

pub struct Evaluator<'a> {
    store: &'a PackageStore,
    globals: &'a HashMap<GlobalId, &'a CallableDecl>,
    resolutions: &'a Resolutions,
    package: PackageId,
    env: Env,
}

impl<'a> Evaluator<'a> {
    #[must_use]
    pub fn new(
        store: &'a PackageStore,
        globals: &'a HashMap<GlobalId, &CallableDecl>,
        resolutions: &'a Resolutions,
        package: PackageId,
        env: Env,
    ) -> Self {
        Self {
            store,
            globals,
            resolutions,
            package,
            env,
        }
    }

    #[must_use]
    pub fn from_store(
        store: &'a PackageStore,
        id: PackageId,
        globals: &'a HashMap<GlobalId, &CallableDecl>,
    ) -> Self {
        let unit = store
            .get(id)
            .expect("compile unit should be in package store");
        Evaluator {
            store,
            globals,
            resolutions: unit.context.resolutions(),
            package: id,
            env: Env::default(),
        }
    }

    /// Evaluates the given statement.
    /// # Errors
    /// Returns the first error encountered during execution.
    pub fn eval_stmt(mut self, stmt: &Stmt) -> Result<(Value, Env), Error> {
        match self.eval_stmt_impl(stmt) {
            ControlFlow::Continue(val) | ControlFlow::Break(Reason::Return(val)) => {
                Ok((val, self.env))
            }
            ControlFlow::Break(Reason::Error(error)) => Err(error),
        }
    }

    /// Evaluates the given expression.
    /// # Errors
    /// Returns the first error encountered during execution.
    pub fn eval_expr(mut self, expr: &Expr) -> Result<(Value, Env), Error> {
        match self.eval_expr_impl(expr) {
            ControlFlow::Continue(val) | ControlFlow::Break(Reason::Return(val)) => {
                Ok((val, self.env))
            }
            ControlFlow::Break(Reason::Error(error)) => Err(error),
        }
    }

    fn eval_expr_impl(&mut self, expr: &Expr) -> ControlFlow<Reason, Value> {
        match &expr.kind {
            ExprKind::Array(arr) => {
                let mut val_arr = vec![];
                for expr in arr {
                    val_arr.push(self.eval_expr_impl(expr)?);
                }
                ControlFlow::Continue(Value::Array(val_arr))
            }
            ExprKind::ArrayRepeat(item, size) => {
                let item_val = self.eval_expr_impl(item)?;
                let size_val: i64 = self.eval_expr_impl(size)?.try_into().with_span(size.span)?;
                let s = match size_val.try_into() {
                    Ok(i) => ControlFlow::Continue(i),
                    Err(_) => ControlFlow::Break(Reason::Error(Error::Count(size_val, size.span))),
                }?;
                ControlFlow::Continue(Value::Array(vec![item_val; s]))
            }
            ExprKind::Assign(lhs, rhs) => {
                let val = self.eval_expr_impl(rhs)?;
                self.update_binding(lhs, val)
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                let update = self.eval_binop(*op, lhs, rhs)?;
                self.update_binding(lhs, update)
            }
            ExprKind::BinOp(op, lhs, rhs) => self.eval_binop(*op, lhs, rhs),
            ExprKind::Block(block) => self.eval_block(block),
            ExprKind::Call(call, args) => self.eval_call(call, args),
            ExprKind::Fail(msg) => ControlFlow::Break(Reason::Error(Error::UserFail(
                self.eval_expr_impl(msg)?.try_into().with_span(msg.span)?,
                expr.span,
            ))),
            ExprKind::If(cond, then, els) => {
                if self.eval_expr_impl(cond)?.try_into().with_span(cond.span)? {
                    self.eval_block(then)
                } else if let Some(els) = els {
                    self.eval_expr_impl(els)
                } else {
                    ControlFlow::Continue(Value::UNIT)
                }
            }
            ExprKind::Index(arr, index_expr) => {
                let arr = self
                    .eval_expr_impl(arr)?
                    .try_into_array()
                    .with_span(arr.span)?;
                let index_val = self.eval_expr_impl(index_expr)?;
                match &index_val {
                    Value::Int(index) => index_array(&arr, *index, index_expr.span),
                    Value::Range(start, step, end) => {
                        slice_array(&arr, start, step, end, index_expr.span)
                    }
                    _ => ControlFlow::Break(Reason::Error(Error::Type(
                        "Int or Range",
                        index_val.type_name(),
                        index_expr.span,
                    ))),
                }
            }
            ExprKind::Lit(lit) => ControlFlow::Continue(lit_to_val(lit)),
            ExprKind::Paren(expr) => self.eval_expr_impl(expr),
            ExprKind::Path(path) => ControlFlow::Continue(self.resolve_binding(path.id)),
            ExprKind::Range(start, step, end) => self.eval_range(start, step, end),
            ExprKind::Return(expr) => {
                ControlFlow::Break(Reason::Return(self.eval_expr_impl(expr)?))
            }
            ExprKind::Tuple(tup) => {
                let mut val_tup = vec![];
                for expr in tup {
                    val_tup.push(self.eval_expr_impl(expr)?);
                }
                ControlFlow::Continue(Value::Tuple(val_tup))
            }
            ExprKind::UnOp(op, rhs) => self.eval_unop(expr, *op, rhs),
            ExprKind::AssignUpdate(..)
            | ExprKind::Conjugate(..)
            | ExprKind::Err
            | ExprKind::Field(..)
            | ExprKind::For(..)
            | ExprKind::Hole
            | ExprKind::Lambda(..)
            | ExprKind::Repeat(..)
            | ExprKind::TernOp(..)
            | ExprKind::While(..) => {
                ControlFlow::Break(Reason::Error(Error::Unimplemented(expr.span)))
            }
        }
    }

    fn eval_range(
        &mut self,
        start: &Option<Box<Expr>>,
        step: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
    ) -> ControlFlow<Reason, Value> {
        let mut to_opt_i64 = |e: &Option<Box<Expr>>| match e {
            Some(expr) => ControlFlow::Continue(Some(
                self.eval_expr_impl(expr)?.try_into().with_span(expr.span)?,
            )),
            None => ControlFlow::Continue(None),
        };
        ControlFlow::Continue(Value::Range(
            to_opt_i64(start)?,
            to_opt_i64(step)?,
            to_opt_i64(end)?,
        ))
    }

    fn eval_block(&mut self, block: &Block) -> ControlFlow<Reason, Value> {
        self.enter_scope();
        let result = if let Some((last, most)) = block.stmts.split_last() {
            for stmt in most {
                let _ = self.eval_stmt_impl(stmt)?;
            }
            self.eval_stmt_impl(last)
        } else {
            ControlFlow::Continue(Value::UNIT)
        };
        self.leave_scope(true);
        result
    }

    fn eval_stmt_impl(&mut self, stmt: &Stmt) -> ControlFlow<Reason, Value> {
        match &stmt.kind {
            StmtKind::Empty => ControlFlow::Continue(Value::UNIT),
            StmtKind::Expr(expr) => self.eval_expr_impl(expr),
            StmtKind::Local(mutability, pat, expr) => {
                let val = self.eval_expr_impl(expr)?;
                self.bind_value(pat, val, expr.span, *mutability)?;
                ControlFlow::Continue(Value::UNIT)
            }
            StmtKind::Semi(expr) => {
                let _ = self.eval_expr_impl(expr)?;
                ControlFlow::Continue(Value::UNIT)
            }
            StmtKind::Qubit(_, pat, qubit_init, block) => {
                let qubits = self.eval_qubit_init(qubit_init)?;
                if let Some(block) = block {
                    self.enter_scope();
                    self.bind_value(pat, qubits, stmt.span, Mutability::Immutable)?;
                    let _ = self.eval_block(block)?;
                    self.leave_scope(true);
                } else {
                    self.bind_value(pat, qubits, stmt.span, Mutability::Immutable)?;
                }
                ControlFlow::Continue(Value::UNIT)
            }
        }
    }

    fn eval_qubit_init(&mut self, qubit_init: &QubitInit) -> ControlFlow<Reason, Value> {
        match &qubit_init.kind {
            QubitInitKind::Array(count) => {
                let count_val: i64 = self
                    .eval_expr_impl(count)?
                    .try_into()
                    .with_span(count.span)?;
                let count: usize = match count_val.try_into() {
                    Ok(i) => ControlFlow::Continue(i),
                    Err(_) => {
                        ControlFlow::Break(Reason::Error(Error::Count(count_val, count.span)))
                    }
                }?;
                let mut arr = vec![];
                arr.resize_with(count, || Value::Qubit(__quantum__rt__qubit_allocate()));
                ControlFlow::Continue(Value::Array(arr))
            }
            QubitInitKind::Paren(qubit_init) => self.eval_qubit_init(qubit_init),
            QubitInitKind::Single => {
                ControlFlow::Continue(Value::Qubit(__quantum__rt__qubit_allocate()))
            }
            QubitInitKind::Tuple(tup) => {
                let mut tup_vec = vec![];
                for init in tup {
                    tup_vec.push(self.eval_qubit_init(init)?);
                }
                ControlFlow::Continue(Value::Tuple(tup_vec))
            }
        }
    }

    fn eval_call(&mut self, call: &Expr, args: &Expr) -> ControlFlow<Reason, Value> {
        let call_val = self.eval_expr_impl(call)?;
        let call_span = call.span;
        let (call, functor) = value_to_call_id(call_val, call.span)?;

        let args_val = self.eval_expr_impl(args)?;

        let decl = *self
            .globals
            .get(&call)
            .unwrap_or_else(|| panic!("called unknown global value: {call}"));

        let spec = specialization_from_functor_app(&functor);

        let resolutions = if call.package == self.package {
            self.resolutions
        } else {
            self.store
                .get(call.package)
                .expect("global value should refer only to stored packages")
                .context
                .resolutions()
        };

        let mut new_self = Self {
            env: Env::default(),
            package: call.package,
            resolutions,
            ..*self
        };
        let call_res = new_self.eval_call_spec(decl, spec, args_val, args.span, call_span);

        match call_res {
            ControlFlow::Break(Reason::Return(val)) => ControlFlow::Continue(val),
            ControlFlow::Continue(_) | ControlFlow::Break(_) => call_res,
        }
    }

    fn eval_call_spec(
        &mut self,
        decl: &CallableDecl,
        spec: Spec,
        args_val: Value,
        args_span: Span,
        call_span: Span,
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
                        || ControlFlow::Break(Reason::Error(Error::MissingSpec(spec, call_span))),
                        |spec_decl| ControlFlow::Continue(&spec_decl.body),
                    )?;
                match spec_decl {
                    SpecBody::Impl(_, body_block) => {
                        if spec == Spec::Ctl || spec == Spec::CtlAdj {
                            ControlFlow::Break(Reason::Error(Error::Unimplemented(call_span)))
                        } else {
                            self.bind_value(
                                &decl.input,
                                args_val,
                                args_span,
                                Mutability::Immutable,
                            )?;
                            self.eval_block(body_block)
                        }
                    }
                    SpecBody::Gen(SpecGen::Slf) => {
                        self.eval_call_spec(decl, Spec::Body, args_val, args_span, call_span)
                    }
                    SpecBody::Gen(SpecGen::Intrinsic) => {
                        invoke_intrinsic(&decl.name.name, call_span, args_val, args_span)
                    }
                    SpecBody::Gen(_) => {
                        ControlFlow::Break(Reason::Error(Error::MissingSpec(spec, call_span)))
                    }
                }
            }
            _ => ControlFlow::Break(Reason::Error(Error::MissingSpec(spec, call_span))),
        };
        self.leave_scope(false);
        res
    }

    fn eval_unop(&mut self, expr: &Expr, op: UnOp, rhs: &Expr) -> ControlFlow<Reason, Value> {
        let val = self.eval_expr_impl(rhs)?;
        match op {
            UnOp::Neg => match val {
                Value::BigInt(v) => ControlFlow::Continue(Value::BigInt(v.neg())),
                Value::Double(v) => ControlFlow::Continue(Value::Double(v.neg())),
                Value::Int(v) => ControlFlow::Continue(Value::Int(v.wrapping_neg())),
                _ => ControlFlow::Break(Reason::Error(Error::Type(
                    "Int, BigInt, or Double",
                    val.type_name(),
                    rhs.span,
                ))),
            },
            UnOp::Pos => match val {
                Value::BigInt(_) | Value::Int(_) | Value::Double(_) => ControlFlow::Continue(val),
                _ => ControlFlow::Break(Reason::Error(Error::Type(
                    "Int, BigInt, or Double",
                    val.type_name(),
                    rhs.span,
                ))),
            },
            UnOp::NotL => match val {
                Value::Bool(b) => ControlFlow::Continue(Value::Bool(!b)),
                _ => ControlFlow::Break(Reason::Error(Error::Type(
                    "Bool",
                    val.type_name(),
                    rhs.span,
                ))),
            },
            UnOp::NotB => match val {
                Value::Int(v) => ControlFlow::Continue(Value::Int(!v)),
                Value::BigInt(v) => ControlFlow::Continue(Value::BigInt(!v)),
                _ => ControlFlow::Break(Reason::Error(Error::Type(
                    "Int or BigInt",
                    val.type_name(),
                    rhs.span,
                ))),
            },
            UnOp::Functor(functor) => match val {
                Value::Closure => {
                    ControlFlow::Break(Reason::Error(Error::Unimplemented(expr.span)))
                }
                Value::Global(id, app) => {
                    ControlFlow::Continue(Value::Global(id, update_functor_app(functor, &app)))
                }
                _ => ControlFlow::Break(Reason::Error(Error::Type(
                    "Callable",
                    val.type_name(),
                    rhs.span,
                ))),
            },
            UnOp::Unwrap => ControlFlow::Break(Reason::Error(Error::Unimplemented(expr.span))),
        }
    }

    fn eval_binop(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr) -> ControlFlow<Reason, Value> {
        let lhs_val = self.eval_expr_impl(lhs)?;
        match op {
            BinOp::Add => eval_binop_add(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::AndB => eval_binop_andb(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::AndL => self.eval_binop_andl(lhs_val.try_into().with_span(lhs.span)?, rhs),
            BinOp::Div => eval_binop_div(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::Eq => eval_binop_eq(&lhs_val, lhs.span, &self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::Exp => eval_binop_exp(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::Gt => eval_binop_gt(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::Gte => eval_binop_gte(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::Lt => eval_binop_lt(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::Lte => eval_binop_lte(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::Mod => eval_binop_mod(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::Mul => eval_binop_mul(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::Neq => eval_binop_neq(&lhs_val, lhs.span, &self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::OrB => eval_binop_orb(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::OrL => self.eval_binop_orl(lhs_val.try_into().with_span(lhs.span)?, rhs),
            BinOp::Shl => eval_binop_shl(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::Shr => eval_binop_shr(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::Sub => eval_binop_sub(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
            BinOp::XorB => eval_binop_xorb(lhs_val, lhs.span, self.eval_expr_impl(rhs)?, rhs.span),
        }
    }

    fn eval_binop_andl(&mut self, lhs: bool, rhs: &Expr) -> ControlFlow<Reason, Value> {
        ControlFlow::Continue(Value::Bool(
            lhs && self.eval_expr_impl(rhs)?.try_into().with_span(rhs.span)?,
        ))
    }

    fn eval_binop_orl(&mut self, lhs: bool, rhs: &Expr) -> ControlFlow<Reason, Value> {
        ControlFlow::Continue(Value::Bool(
            lhs || self.eval_expr_impl(rhs)?.try_into().with_span(rhs.span)?,
        ))
    }

    fn enter_scope(&mut self) {
        self.env.0.push(HashMap::default());
    }

    fn leave_scope(&mut self, release: bool) {
        if release {
            for (_, var) in self
                .env
                .0
                .pop()
                .expect("scope should be entered first before leaving")
                .drain()
            {
                var.value.release();
            }
        } else {
            let _ = self.env.0.pop();
        }
    }

    fn bind_value(
        &mut self,
        pat: &Pat,
        value: Value,
        span: Span,
        mutability: Mutability,
    ) -> ControlFlow<Reason, ()> {
        match &pat.kind {
            PatKind::Bind(variable, _) => {
                let id = self.defid_to_globalid(
                    self.resolutions
                        .get(&variable.id)
                        .unwrap_or_else(|| panic!("binding is not resolved: {}", variable.id)),
                );

                let scope = self.env.0.last_mut().expect("binding should have a scope");
                match scope.entry(id) {
                    Entry::Vacant(entry) => entry.insert(Variable { value, mutability }),
                    Entry::Occupied(_) => panic!("duplicate binding: {id}"),
                };
                ControlFlow::Continue(())
            }
            PatKind::Discard(_) => ControlFlow::Continue(()),
            PatKind::Elided => panic!("elision used in binding"),
            PatKind::Paren(pat) => self.bind_value(pat, value, span, mutability),
            PatKind::Tuple(tup) => {
                let val_tup = value.try_into_tuple().with_span(span)?;
                if val_tup.len() == tup.len() {
                    for (pat, val) in tup.iter().zip(val_tup.into_iter()) {
                        self.bind_value(pat, val, span, mutability)?;
                    }
                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(Reason::Error(Error::TupleArity(
                        tup.len(),
                        val_tup.len(),
                        pat.span,
                    )))
                }
            }
        }
    }

    fn resolve_binding(&mut self, id: NodeId) -> Value {
        let id = self
            .resolutions
            .get(&id)
            .unwrap_or_else(|| panic!("binding is not resolved: {id}"));

        let global_id = self.defid_to_globalid(id);
        let local = if id.package == PackageSrc::Local {
            self.env
                .0
                .iter()
                .rev()
                .find_map(|s| s.get(&global_id))
                .map(|v| v.value.clone())
        } else {
            None
        };
        local.unwrap_or_else(|| Value::Global(global_id, FunctorApp::default()))
    }

    fn update_binding(&mut self, lhs: &Expr, rhs: Value) -> ControlFlow<Reason, Value> {
        match (&lhs.kind, rhs) {
            (ExprKind::Path(path), rhs) => {
                let id = self.defid_to_globalid(
                    self.resolutions
                        .get(&path.id)
                        .unwrap_or_else(|| panic!("path is not resolved: {}", path.id)),
                );

                let mut variable = self
                    .env
                    .0
                    .iter_mut()
                    .rev()
                    .find_map(|scope| scope.get_mut(&id))
                    .unwrap_or_else(|| panic!("path is not bound: {id}"));

                if variable.is_mutable() {
                    variable.value = rhs;
                    ControlFlow::Continue(Value::UNIT)
                } else {
                    ControlFlow::Break(Reason::Error(Error::Mutability(path.span)))
                }
            }
            (ExprKind::Hole, _) => ControlFlow::Continue(Value::UNIT),
            (ExprKind::Paren(expr), rhs) => self.update_binding(expr, rhs),
            (ExprKind::Tuple(var_tup), Value::Tuple(mut tup)) => {
                if var_tup.len() == tup.len() {
                    for (expr, val) in var_tup.iter().zip(tup.drain(..)) {
                        self.update_binding(expr, val)?;
                    }
                    ControlFlow::Continue(Value::UNIT)
                } else {
                    ControlFlow::Break(Reason::Error(Error::TupleArity(
                        var_tup.len(),
                        tup.len(),
                        lhs.span,
                    )))
                }
            }
            _ => ControlFlow::Break(Reason::Error(Error::Unassignable(lhs.span))),
        }
    }

    fn defid_to_globalid(&self, id: &DefId) -> GlobalId {
        GlobalId {
            package: match id.package {
                PackageSrc::Local => self.package,
                PackageSrc::Extern(id) => id,
            },
            node: id.node,
        }
    }
}

fn specialization_from_functor_app(functor: &FunctorApp) -> Spec {
    match (functor.adjoint, functor.controlled) {
        (false, 0) => Spec::Body,
        (true, 0) => Spec::Adj,
        (false, _) => Spec::Ctl,
        (true, _) => Spec::CtlAdj,
    }
}

fn value_to_call_id(val: Value, span: Span) -> ControlFlow<Reason, (GlobalId, FunctorApp)> {
    match val {
        Value::Closure => ControlFlow::Break(Reason::Error(Error::Unimplemented(span))),
        Value::Global(global, functor) => ControlFlow::Continue((global, functor)),
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
        Lit::Result(v) => Value::Result(match v {
            ast::Result::Zero => false,
            ast::Result::One => true,
        }),
        Lit::String(v) => Value::String(v.clone()),
    }
}

fn index_array(arr: &[Value], index: i64, span: Span) -> ControlFlow<Reason, Value> {
    match arr.get(index.as_index(span)?) {
        Some(v) => ControlFlow::Continue(v.clone()),
        None => ControlFlow::Break(Reason::Error(Error::OutOfRange(index, span))),
    }
}

fn slice_array(
    arr: &[Value],
    start: &Option<i64>,
    step: &Option<i64>,
    end: &Option<i64>,
    span: Span,
) -> ControlFlow<Reason, Value> {
    if let Some(0) = step {
        ControlFlow::Break(Reason::Error(Error::RangeStepZero(span)))
    } else {
        let len: i64 = match arr.len().try_into() {
            Ok(len) => ControlFlow::Continue(len),
            Err(_) => ControlFlow::Break(Reason::Error(Error::ArrayTooLarge(span))),
        }?;
        let step = step.unwrap_or(1);
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

        ControlFlow::Continue(Value::Array(slice))
    }
}

fn update_functor_app(functor: Functor, app: &FunctorApp) -> FunctorApp {
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
        Value::Array(mut arr) => {
            arr.append(&mut rhs_val.try_into_array().with_span(rhs_span)?);
            ControlFlow::Continue(Value::Array(arr))
        }
        Value::BigInt(val) => {
            let rhs: BigInt = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::BigInt(val + rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Double(val + rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Int(val + rhs))
        }
        Value::String(val) => {
            let rhs: String = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::String(val + &rhs))
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
            ControlFlow::Continue(Value::BigInt(val & rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Int(val & rhs))
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
                ControlFlow::Break(Reason::Error(Error::DivZero(rhs_span)))
            } else {
                ControlFlow::Continue(Value::BigInt(val / rhs))
            }
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs == 0 {
                ControlFlow::Break(Reason::Error(Error::DivZero(rhs_span)))
            } else {
                ControlFlow::Continue(Value::Int(val / rhs))
            }
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs == 0.0 {
                ControlFlow::Break(Reason::Error(Error::DivZero(rhs_span)))
            } else {
                ControlFlow::Continue(Value::Double(val / rhs))
            }
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
            "BigInt, Double, or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}

fn supports_eq(val: &Value, val_span: Span) -> ControlFlow<Reason, ()> {
    match val {
        Value::Closure | Value::Global(..) => {
            ControlFlow::Break(Reason::Error(Error::Equality(val.type_name(), val_span)))
        }
        _ => ControlFlow::Continue(()),
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
        ControlFlow::Continue(Value::Bool(lhs_val == rhs_val))
    } else {
        ControlFlow::Break(Reason::Error(Error::Type(
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
            if rhs_val <= 0 {
                ControlFlow::Break(Reason::Error(Error::Negative(rhs_val, rhs_span)))
            } else {
                let rhs_val: u32 = match rhs_val.try_into() {
                    Ok(v) => ControlFlow::Continue(v),
                    Err(_) => {
                        ControlFlow::Break(Reason::Error(Error::IntTooLarge(rhs_val, rhs_span)))
                    }
                }?;
                ControlFlow::Continue(Value::BigInt(val.pow(rhs_val)))
            }
        }
        Value::Double(val) => ControlFlow::Continue(Value::Double(
            val.powf(rhs_val.try_into().with_span(rhs_span)?),
        )),
        Value::Int(val) => {
            let rhs_val: i64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs_val <= 0 {
                ControlFlow::Break(Reason::Error(Error::Negative(rhs_val, rhs_span)))
            } else {
                let rhs_val: u32 = match rhs_val.try_into() {
                    Ok(v) => ControlFlow::Continue(v),
                    Err(_) => {
                        ControlFlow::Break(Reason::Error(Error::IntTooLarge(rhs_val, rhs_span)))
                    }
                }?;
                ControlFlow::Continue(Value::Int(val.pow(rhs_val)))
            }
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
            ControlFlow::Continue(Value::Bool(val > rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Bool(val > rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Bool(val > rhs))
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
            ControlFlow::Continue(Value::Bool(val >= rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Bool(val >= rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Bool(val >= rhs))
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
            ControlFlow::Continue(Value::Bool(val < rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Bool(val < rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Bool(val < rhs))
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
            ControlFlow::Continue(Value::Bool(val <= rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Bool(val <= rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Bool(val <= rhs))
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
            ControlFlow::Continue(Value::BigInt(val % rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Int(val % rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Double(val % rhs))
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
            ControlFlow::Continue(Value::BigInt(val * rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Int(val * rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Double(val * rhs))
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
        ControlFlow::Continue(Value::Bool(lhs_val != rhs_val))
    } else {
        ControlFlow::Break(Reason::Error(Error::Type(
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
            ControlFlow::Continue(Value::BigInt(val | rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Int(val | rhs))
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
                ControlFlow::Continue(Value::BigInt(val << rhs))
            } else {
                ControlFlow::Continue(Value::BigInt(val >> rhs.abs()))
            }
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs > 0 {
                ControlFlow::Continue(Value::Int(val << rhs))
            } else {
                ControlFlow::Continue(Value::Int(val >> rhs.abs()))
            }
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
                ControlFlow::Continue(Value::BigInt(val >> rhs))
            } else {
                ControlFlow::Continue(Value::BigInt(val << rhs.abs()))
            }
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            if rhs > 0 {
                ControlFlow::Continue(Value::Int(val >> rhs))
            } else {
                ControlFlow::Continue(Value::Int(val << rhs.abs()))
            }
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
            ControlFlow::Continue(Value::BigInt(val - rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Int(val - rhs))
        }
        Value::Double(val) => {
            let rhs: f64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Double(val - rhs))
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
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
            ControlFlow::Continue(Value::BigInt(val ^ rhs))
        }
        Value::Int(val) => {
            let rhs: i64 = rhs_val.try_into().with_span(rhs_span)?;
            ControlFlow::Continue(Value::Int(val ^ rhs))
        }
        _ => ControlFlow::Break(Reason::Error(Error::Type(
            "BigInt or Int",
            lhs_val.type_name(),
            lhs_span,
        ))),
    }
}
