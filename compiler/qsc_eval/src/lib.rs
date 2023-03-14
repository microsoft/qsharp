// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

mod globals;
mod intrinsic;
pub mod val;

use std::{
    collections::{hash_map::Entry, HashMap},
    ops::{ControlFlow, Neg},
};

use globals::{extract_callables, GlobalId};
use intrinsic::invoke_intrinsic;
use qir_backend::{Pauli, __quantum__rt__qubit_allocate};
use qsc_ast::ast::{
    self, Block, CallableBody, CallableDecl, Expr, ExprKind, Functor, Lit, Mutability, NodeId, Pat,
    PatKind, QubitInit, Span, Spec, SpecBody, SpecGen, Stmt, StmtKind, UnOp,
};
use qsc_frontend::{
    compile::{CompileUnit, PackageId, PackageStore},
    resolve::{DefId, PackageSrc},
};
use val::{ConversionError, FunctorApp, Value};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Error {
    span: Span,
    kind: ErrorKind,
}

#[derive(Debug)]
enum Reason {
    Error(Span, ErrorKind),
    Return(Value),
}

#[derive(Debug)]
enum ErrorKind {
    Count(i64),
    EmptyExpr,
    IndexVal(i64),
    IntegerSize,
    MissingSpec(Spec),
    Mutability,
    OutOfRange(i64),
    RangeStepZero,
    Type(&'static str, &'static str),
    TupleArity(usize, usize),
    Unassignable,
    Unimplemented,
    UnknownIntrinsic,
    UserFail(String),
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
            Err(e) => {
                ControlFlow::Break(Reason::Error(span, ErrorKind::Type(e.expected, e.actual)))
            }
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
            Err(_) => ControlFlow::Break(Reason::Error(span, ErrorKind::IndexVal(*self))),
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

#[allow(dead_code)]
pub struct Evaluator<'a> {
    store: &'a PackageStore,
    current_unit: &'a CompileUnit,
    current_id: PackageId,
    scopes: Vec<HashMap<GlobalId, Variable>>,
    globals: HashMap<GlobalId, &'a CallableDecl>,
}

impl<'a> Evaluator<'a> {
    #[must_use]
    pub fn new(store: &'a PackageStore, entry_id: PackageId) -> Self {
        Self {
            store,
            current_unit: store
                .get(entry_id)
                .expect("Entry id must be present in package store"),
            current_id: entry_id,
            scopes: vec![],
            globals: extract_callables(store),
        }
    }

    /// Evaluates the entry expression from the current context.
    /// # Errors
    /// Returns the first error encountered during execution.
    pub fn run(&mut self) -> Result<Value, Error> {
        if let Some(expr) = &self.current_unit.package.entry {
            match self.eval_expr(expr) {
                ControlFlow::Continue(val) | ControlFlow::Break(Reason::Return(val)) => Ok(val),
                ControlFlow::Break(Reason::Error(span, kind)) => Err(Error { span, kind }),
            }
        } else {
            Err(Error {
                span: Span { lo: 0, hi: 0 },
                kind: ErrorKind::EmptyExpr,
            })
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> ControlFlow<Reason, Value> {
        match &expr.kind {
            ExprKind::Array(arr) => {
                let mut val_arr = vec![];
                for expr in arr {
                    val_arr.push(self.eval_expr(expr)?);
                }
                ControlFlow::Continue(Value::Array(val_arr))
            }
            ExprKind::ArrayRepeat(item, size) => {
                let item_val = self.eval_expr(item)?;
                let size_val: i64 = self.eval_expr(size)?.try_into().with_span(size.span)?;
                let s = match size_val.try_into() {
                    Ok(i) => ControlFlow::Continue(i),
                    Err(_) => {
                        ControlFlow::Break(Reason::Error(size.span, ErrorKind::Count(size_val)))
                    }
                }?;
                ControlFlow::Continue(Value::Array(vec![item_val; s]))
            }
            ExprKind::Assign(lhs, rhs) => {
                let val = self.eval_expr(rhs)?;
                self.update_binding(lhs, val)
            }
            ExprKind::Block(block) => self.eval_block(block),
            ExprKind::Call(call, args) => self.eval_call(call, args),
            ExprKind::Fail(msg) => ControlFlow::Break(Reason::Error(
                expr.span,
                ErrorKind::UserFail(self.eval_expr(msg)?.try_into().with_span(msg.span)?),
            )),
            ExprKind::If(cond, then, els) => {
                if self.eval_expr(cond)?.try_into().with_span(cond.span)? {
                    self.eval_block(then)
                } else if let Some(els) = els {
                    self.eval_expr(els)
                } else {
                    ControlFlow::Continue(Value::UNIT)
                }
            }
            ExprKind::Index(arr, index_expr) => {
                let arr = self.eval_expr(arr)?.try_into_array().with_span(arr.span)?;
                let index_val = self.eval_expr(index_expr)?;
                match &index_val {
                    Value::Int(index) => index_array(&arr, *index, index_expr.span),
                    Value::Range(start, step, end) => {
                        slice_array(&arr, start, step, end, index_expr.span)
                    }
                    _ => ControlFlow::Break(Reason::Error(
                        index_expr.span,
                        ErrorKind::Type("Int or Range", index_val.type_name()),
                    )),
                }
            }
            ExprKind::Lit(lit) => ControlFlow::Continue(lit_to_val(lit)),
            ExprKind::Paren(expr) => self.eval_expr(expr),
            ExprKind::Path(path) => ControlFlow::Continue(self.resolve_binding(path.id)),
            ExprKind::Range(start, step, end) => self.eval_range(start, step, end),
            ExprKind::Return(expr) => ControlFlow::Break(Reason::Return(self.eval_expr(expr)?)),
            ExprKind::Tuple(tup) => {
                let mut val_tup = vec![];
                for expr in tup {
                    val_tup.push(self.eval_expr(expr)?);
                }
                ControlFlow::Continue(Value::Tuple(val_tup))
            }
            ExprKind::UnOp(op, rhs) => self.eval_unary_op_expr(expr, *op, rhs),
            ExprKind::AssignOp(..)
            | ExprKind::AssignUpdate(..)
            | ExprKind::BinOp(..)
            | ExprKind::Conjugate(..)
            | ExprKind::Err
            | ExprKind::Field(..)
            | ExprKind::For(..)
            | ExprKind::Hole
            | ExprKind::Lambda(..)
            | ExprKind::Repeat(..)
            | ExprKind::TernOp(..)
            | ExprKind::While(..) => {
                ControlFlow::Break(Reason::Error(expr.span, ErrorKind::Unimplemented))
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
            Some(expr) => {
                ControlFlow::Continue(Some(self.eval_expr(expr)?.try_into().with_span(expr.span)?))
            }
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
                let _ = self.eval_stmt(stmt)?;
            }
            self.eval_stmt(last)
        } else {
            ControlFlow::Continue(Value::UNIT)
        };
        self.leave_scope();
        result
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> ControlFlow<Reason, Value> {
        match &stmt.kind {
            StmtKind::Expr(expr) => self.eval_expr(expr),
            StmtKind::Local(mutability, pat, expr) => {
                let val = self.eval_expr(expr)?;
                self.bind_value(pat, val, expr.span, *mutability)?;
                ControlFlow::Continue(Value::UNIT)
            }
            StmtKind::Semi(expr) => {
                let _ = self.eval_expr(expr)?;
                ControlFlow::Continue(Value::UNIT)
            }
            StmtKind::Qubit(_, pat, qubit_init, block) => {
                let qubits = self.eval_qubit_init(qubit_init)?;
                if let Some(block) = block {
                    self.enter_scope();
                    self.bind_value(pat, qubits, stmt.span, Mutability::Immutable)?;
                    let _ = self.eval_block(block)?;
                    self.leave_scope();
                } else {
                    self.bind_value(pat, qubits, stmt.span, Mutability::Immutable)?;
                }
                ControlFlow::Continue(Value::UNIT)
            }
        }
    }

    fn eval_qubit_init(&mut self, qubit_init: &QubitInit) -> ControlFlow<Reason, Value> {
        match &qubit_init.kind {
            ast::QubitInitKind::Array(count) => {
                let count_val: i64 = self.eval_expr(count)?.try_into().with_span(count.span)?;
                let count: usize = match count_val.try_into() {
                    Ok(i) => ControlFlow::Continue(i),
                    Err(_) => {
                        ControlFlow::Break(Reason::Error(count.span, ErrorKind::Count(count_val)))
                    }
                }?;
                let mut arr = vec![];
                for _ in 0..count {
                    arr.push(Value::Qubit(__quantum__rt__qubit_allocate()));
                }
                ControlFlow::Continue(Value::Array(arr))
            }
            ast::QubitInitKind::Paren(qubit_init) => self.eval_qubit_init(qubit_init),
            ast::QubitInitKind::Single => {
                ControlFlow::Continue(Value::Qubit(__quantum__rt__qubit_allocate()))
            }
            ast::QubitInitKind::Tuple(tup) => {
                let mut tup_vec = vec![];
                for init in tup {
                    tup_vec.push(self.eval_qubit_init(init)?);
                }
                ControlFlow::Continue(Value::Tuple(tup_vec))
            }
        }
    }

    fn eval_call(&mut self, call: &Expr, args: &Expr) -> ControlFlow<Reason, Value> {
        let call_val = self.eval_expr(call)?;
        let call_span = call.span;
        let (call, functor) = value_to_call_id(call_val, call.span)?;

        let args_val = self.eval_expr(args)?;

        let decl = *self
            .globals
            .get(&call)
            .unwrap_or_else(|| panic!("{call:?} is not in globals map"));

        let spec = specialization_from_functor_app(&functor);

        let (cached_id, cached_unit) = (self.current_id, self.current_unit);
        (self.current_id, self.current_unit) = (
            call.package,
            self.store
                .get(call.package)
                .expect("Store must contain compile unit for package id"),
        );

        self.enter_scope();
        let call_res = self.eval_call_specialization(decl, spec, args_val, args.span, call_span);
        self.leave_scope();

        (self.current_id, self.current_unit) = (cached_id, cached_unit);

        match call_res {
            ControlFlow::Break(Reason::Return(val)) => ControlFlow::Continue(val),
            ControlFlow::Continue(_) | ControlFlow::Break(_) => call_res,
        }
    }

    fn eval_call_specialization(
        &mut self,
        decl: &CallableDecl,
        spec: Spec,
        args_val: Value,
        args_span: Span,
        call_span: Span,
    ) -> ControlFlow<Reason, Value> {
        match (&decl.body, spec) {
            (CallableBody::Block(body_block), Spec::Body) => {
                self.bind_value(&decl.input, args_val, args_span, Mutability::Immutable)?;
                self.eval_block(body_block)
            }
            (CallableBody::Specs(spec_decls), spec) => {
                let spec_decl = spec_decls
                    .iter()
                    .find(|spec_decl| spec_decl.spec == spec)
                    .map_or_else(
                        || {
                            ControlFlow::Break(Reason::Error(
                                decl.span,
                                ErrorKind::MissingSpec(spec),
                            ))
                        },
                        |spec_decl| ControlFlow::Continue(&spec_decl.body),
                    )?;
                match spec_decl {
                    SpecBody::Impl(_, body_block) => {
                        if spec == Spec::Ctl || spec == Spec::CtlAdj {
                            ControlFlow::Break(Reason::Error(call_span, ErrorKind::Unimplemented))
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
                    SpecBody::Gen(SpecGen::Intrinsic) => {
                        invoke_intrinsic(&decl.name.name, decl.name.span, args_val, args_span)
                    }
                    SpecBody::Gen(_) => {
                        ControlFlow::Break(Reason::Error(decl.span, ErrorKind::MissingSpec(spec)))
                    }
                }
            }
            _ => ControlFlow::Break(Reason::Error(decl.span, ErrorKind::MissingSpec(spec))),
        }
    }

    fn eval_unary_op_expr(
        &mut self,
        expr: &Expr,
        op: UnOp,
        rhs: &Expr,
    ) -> ControlFlow<Reason, Value> {
        let val = self.eval_expr(rhs)?;
        match op {
            UnOp::Neg => match val {
                Value::BigInt(v) => ControlFlow::Continue(Value::BigInt(v.neg())),
                Value::Double(v) => ControlFlow::Continue(Value::Double(v.neg())),
                Value::Int(v) => ControlFlow::Continue(Value::Int(v.wrapping_neg())),
                _ => ControlFlow::Break(Reason::Error(
                    rhs.span,
                    ErrorKind::Type("Int, BigInt, or Double", val.type_name()),
                )),
            },
            UnOp::Pos => match val {
                Value::BigInt(_) | Value::Int(_) | Value::Double(_) => ControlFlow::Continue(val),
                _ => ControlFlow::Break(Reason::Error(
                    rhs.span,
                    ErrorKind::Type("Int, BigInt, or Double", val.type_name()),
                )),
            },
            UnOp::NotL => match val {
                Value::Bool(b) => ControlFlow::Continue(Value::Bool(!b)),
                _ => ControlFlow::Break(Reason::Error(
                    rhs.span,
                    ErrorKind::Type("Bool", val.type_name()),
                )),
            },
            UnOp::NotB => match val {
                Value::Int(v) => ControlFlow::Continue(Value::Int(!v)),
                Value::BigInt(v) => ControlFlow::Continue(Value::BigInt(!v)),
                _ => ControlFlow::Break(Reason::Error(
                    rhs.span,
                    ErrorKind::Type("Int or BigInt", val.type_name()),
                )),
            },
            UnOp::Functor(functor) => match val {
                Value::Closure => {
                    ControlFlow::Break(Reason::Error(expr.span, ErrorKind::Unimplemented))
                }
                Value::Global(id, app) => {
                    ControlFlow::Continue(Value::Global(id, update_functor_app(functor, &app)))
                }
                _ => ControlFlow::Break(Reason::Error(
                    rhs.span,
                    ErrorKind::Type("Callable", val.type_name()),
                )),
            },
            UnOp::Unwrap => ControlFlow::Break(Reason::Error(expr.span, ErrorKind::Unimplemented)),
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::default());
    }

    fn leave_scope(&mut self) {
        for (_, mut var) in self
            .scopes
            .pop()
            .expect("Cannot leave scope without entering")
            .drain()
        {
            var.value.release();
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
                    self.current_unit
                        .context
                        .resolutions()
                        .get(&variable.id)
                        .unwrap_or_else(|| panic!("{:?} is not resolved", variable.id)),
                );

                let scope = self.scopes.last_mut().expect("Binding requires a scope.");
                match scope.entry(id) {
                    Entry::Vacant(entry) => entry.insert(Variable { value, mutability }),
                    Entry::Occupied(_) => panic!("{id:?} is already bound"),
                };
                ControlFlow::Continue(())
            }
            PatKind::Discard(_) => ControlFlow::Continue(()),
            PatKind::Elided => panic!("Elided pattern not valid syntax in binding"),
            PatKind::Paren(pat) => self.bind_value(pat, value, span, mutability),
            PatKind::Tuple(tup) => {
                let val_tup = value.try_into_tuple().with_span(span)?;
                if val_tup.len() == tup.len() {
                    for (pat, val) in tup.iter().zip(val_tup.into_iter()) {
                        self.bind_value(pat, val, span, mutability)?;
                    }
                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(Reason::Error(
                        pat.span,
                        ErrorKind::TupleArity(tup.len(), val_tup.len()),
                    ))
                }
            }
        }
    }

    fn resolve_binding(&mut self, id: NodeId) -> Value {
        let id = self
            .current_unit
            .context
            .resolutions()
            .get(&id)
            .unwrap_or_else(|| panic!("{id:?} is not resolved"));

        let global_id = self.defid_to_globalid(id);
        let local = if id.package == PackageSrc::Local {
            self.scopes
                .iter()
                .rev()
                .find_map(|s| s.get(&global_id))
                .map(|v| v.value.clone())
        } else {
            None
        };
        local.unwrap_or_else(|| self.resolve_global(global_id))
    }

    fn update_binding(&mut self, lhs: &Expr, rhs: Value) -> ControlFlow<Reason, Value> {
        match (&lhs.kind, rhs) {
            (ExprKind::Path(path), rhs) => {
                let id = self.defid_to_globalid(
                    self.current_unit
                        .context
                        .resolutions()
                        .get(&path.id)
                        .unwrap_or_else(|| panic!("{:?} is not resolved", path.id)),
                );

                let mut variable = self
                    .scopes
                    .iter_mut()
                    .rev()
                    .find_map(|scope| scope.get_mut(&id))
                    .unwrap_or_else(|| panic!("{id:?} is not bound"));

                if variable.is_mutable() {
                    variable.value = rhs;
                    ControlFlow::Continue(Value::UNIT)
                } else {
                    ControlFlow::Break(Reason::Error(path.span, ErrorKind::Mutability))
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
                    ControlFlow::Break(Reason::Error(
                        lhs.span,
                        ErrorKind::TupleArity(var_tup.len(), tup.len()),
                    ))
                }
            }
            _ => ControlFlow::Break(Reason::Error(lhs.span, ErrorKind::Unassignable)),
        }
    }

    fn resolve_global(&mut self, id: GlobalId) -> Value {
        if self.globals.contains_key(&id) {
            Value::Global(id, FunctorApp::default())
        } else {
            panic!("{id:?} is not bound")
        }
    }

    fn defid_to_globalid(&self, id: &DefId) -> GlobalId {
        GlobalId {
            package: match id.package {
                PackageSrc::Local => self.current_id,
                PackageSrc::Extern(p) => p,
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
        Value::Closure => ControlFlow::Break(Reason::Error(span, ErrorKind::Unimplemented)),
        Value::Global(global, functor) => ControlFlow::Continue((global, functor)),
        _ => ControlFlow::Break(Reason::Error(
            span,
            ErrorKind::Type("Callable", val.type_name()),
        )),
    }
}

fn lit_to_val(lit: &Lit) -> Value {
    match lit {
        Lit::BigInt(v) => Value::BigInt(v.clone()),
        Lit::Bool(v) => Value::Bool(*v),
        Lit::Double(v) => Value::Double(*v),
        Lit::Int(v) => Value::Int(*v),
        Lit::Pauli(v) => Value::Pauli(match v {
            ast::Pauli::I => Pauli::I,
            ast::Pauli::X => Pauli::X,
            ast::Pauli::Y => Pauli::Y,
            ast::Pauli::Z => Pauli::Z,
        }),
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
        None => ControlFlow::Break(Reason::Error(span, ErrorKind::OutOfRange(index))),
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
        ControlFlow::Break(Reason::Error(span, ErrorKind::RangeStepZero))
    } else {
        let len: i64 = match arr.len().try_into() {
            Ok(len) => ControlFlow::Continue(len),
            Err(_) => ControlFlow::Break(Reason::Error(span, ErrorKind::IntegerSize)),
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
