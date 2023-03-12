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
    ops::ControlFlow,
};

use globals::extract_callables;
use intrinsic::invoke_intrinsic;
use qir_backend::Pauli;
use qsc_ast::ast::{
    self, Block, CallableBody, CallableDecl, Expr, ExprKind, Lit, Mutability, NodeId, Pat, PatKind,
    Span, Spec, SpecBody, SpecGen, Stmt, StmtKind, UnOp,
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
    MissingSpecialization(Spec),
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
    scopes: Vec<HashMap<DefId, Variable>>,
    globals: HashMap<DefId, &'a CallableDecl>,
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
                    ControlFlow::Continue(Value::Tuple(vec![]))
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
            ExprKind::UnOp(op, rhs) => {
                let val = self.eval_expr(rhs)?;
                match op {
                    UnOp::Neg => val.arithmetic_negate().with_span(rhs.span),
                    UnOp::Pos => match val {
                        Value::BigInt(_) | Value::Int(_) | Value::Double(_) => {
                            ControlFlow::Continue(val)
                        }
                        _ => ControlFlow::Break(Reason::Error(
                            rhs.span,
                            ErrorKind::Type("Int, BigInt, or Double", val.type_name()),
                        )),
                    },
                    UnOp::Functor(_) | UnOp::NotB | UnOp::NotL | UnOp::Unwrap => {
                        ControlFlow::Break(Reason::Error(expr.span, ErrorKind::Unimplemented))
                    }
                }
            }
            ExprKind::AssignOp(_, _, _)
            | ExprKind::AssignUpdate(_, _, _)
            | ExprKind::BinOp(_, _, _)
            | ExprKind::Conjugate(_, _)
            | ExprKind::Err
            | ExprKind::Field(_, _)
            | ExprKind::For(_, _, _)
            | ExprKind::Hole
            | ExprKind::Lambda(_, _, _)
            | ExprKind::Repeat(_, _, _)
            | ExprKind::TernOp(_, _, _, _)
            | ExprKind::While(_, _) => {
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
        self.scopes.push(HashMap::default());
        let result = if let Some((last, most)) = block.stmts.split_last() {
            for stmt in most {
                let _ = self.eval_stmt(stmt)?;
            }
            self.eval_stmt(last)
        } else {
            ControlFlow::Continue(Value::Tuple(vec![]))
        };
        let _ = self.scopes.pop();
        result
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> ControlFlow<Reason, Value> {
        match &stmt.kind {
            StmtKind::Expr(expr) => self.eval_expr(expr),
            StmtKind::Local(mutability, pat, expr) => {
                let val = self.eval_expr(expr)?;
                self.bind_value(pat, val, expr.span, *mutability)?;
                ControlFlow::Continue(Value::Tuple(vec![]))
            }
            StmtKind::Semi(expr) => {
                let _ = self.eval_expr(expr)?;
                ControlFlow::Continue(Value::Tuple(vec![]))
            }
            StmtKind::Qubit(..) => {
                ControlFlow::Break(Reason::Error(stmt.span, ErrorKind::Unimplemented))
            }
        }
    }

    fn eval_call(&mut self, call: &Expr, args: &Expr) -> ControlFlow<Reason, Value> {
        let call_val = self.eval_expr(call)?;
        let call_span = call.span;
        let (call, functor) = match call_val {
            Value::Closure(_, _, _) => {
                ControlFlow::Break(Reason::Error(call.span, ErrorKind::Unimplemented))
            }
            Value::Global(global, functor) => ControlFlow::Continue(match global.package {
                PackageSrc::Local => (
                    DefId {
                        package: PackageSrc::Extern(self.current_id),
                        node: global.node,
                    },
                    functor,
                ),
                PackageSrc::Extern(_) => (global, functor),
            }),
            _ => ControlFlow::Break(Reason::Error(
                call.span,
                ErrorKind::Type("Callable", call_val.type_name()),
            )),
        }?;

        let args_val = self.eval_expr(args)?;

        let decl = *self
            .globals
            .get(&call)
            .unwrap_or_else(|| panic!("{call:?} is not in globals map"));

        let spec = match (functor.adjoint, functor.controlled) {
            (false, 0) => Spec::Body,
            (true, 0) => Spec::Adj,
            (false, _) => Spec::Ctl,
            (true, _) => Spec::CtlAdj,
        };

        self.scopes.push(HashMap::default());
        let call_res = match (&decl.body, spec) {
            (CallableBody::Block(body_block), Spec::Body) => {
                self.bind_value(&decl.input, args_val, args.span, Mutability::Immutable)?;
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
                                ErrorKind::MissingSpecialization(spec),
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
                                args.span,
                                Mutability::Immutable,
                            )?;
                            self.eval_block(body_block)
                        }
                    }
                    SpecBody::Gen(SpecGen::Intrinsic) => {
                        invoke_intrinsic(&decl.name.name, decl.name.span, args_val, args.span)
                    }
                    SpecBody::Gen(_) => ControlFlow::Break(Reason::Error(
                        decl.span,
                        ErrorKind::MissingSpecialization(spec),
                    )),
                }
            }
            _ => ControlFlow::Break(Reason::Error(
                decl.span,
                ErrorKind::MissingSpecialization(spec),
            )),
        };
        let _ = self.scopes.pop();

        match call_res {
            ControlFlow::Break(Reason::Return(val)) => ControlFlow::Continue(val),
            ControlFlow::Continue(_) | ControlFlow::Break(_) => call_res,
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
                let id = self
                    .current_unit
                    .context
                    .resolutions()
                    .get(&variable.id)
                    .unwrap_or_else(|| panic!("{:?} is not resolved", variable.id));

                let scope = self.scopes.last_mut().expect("Binding requires a scope.");
                match scope.entry(*id) {
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

        if id.package == PackageSrc::Local {
            if let Some(var) = self.scopes.iter().rev().find_map(|scope| scope.get(id)) {
                var.value.clone()
            } else {
                let id = &DefId {
                    package: PackageSrc::Extern(self.current_id),
                    node: id.node,
                };
                self.resolve_global(id)
            }
        } else {
            self.resolve_global(id)
        }
    }

    fn update_binding(&mut self, lhs: &Expr, rhs: Value) -> ControlFlow<Reason, Value> {
        match (&lhs.kind, rhs) {
            (ExprKind::Path(path), rhs) => {
                let id = self
                    .current_unit
                    .context
                    .resolutions()
                    .get(&path.id)
                    .unwrap_or_else(|| panic!("{:?} is not resolved", path.id));

                let mut variable = self
                    .scopes
                    .iter_mut()
                    .rev()
                    .find_map(|scope| scope.get_mut(id))
                    .unwrap_or_else(|| panic!("{id:?} is not bound"));

                if variable.is_mutable() {
                    variable.value = rhs;
                    ControlFlow::Continue(Value::Tuple(vec![]))
                } else {
                    ControlFlow::Break(Reason::Error(path.span, ErrorKind::Mutability))
                }
            }
            (ExprKind::Hole, _) => ControlFlow::Continue(Value::Tuple(vec![])),
            (ExprKind::Paren(expr), rhs) => self.update_binding(expr, rhs),
            (ExprKind::Tuple(var_tup), Value::Tuple(mut tup)) => {
                if var_tup.len() == tup.len() {
                    for (expr, val) in var_tup.iter().zip(tup.drain(..)) {
                        self.update_binding(expr, val)?;
                    }
                    ControlFlow::Continue(Value::Tuple(vec![]))
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

    fn resolve_global(&mut self, id: &DefId) -> Value {
        if self.globals.contains_key(id) {
            Value::Global(*id, FunctorApp::default())
        } else {
            panic!("{id:?} is not bound")
        }
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
