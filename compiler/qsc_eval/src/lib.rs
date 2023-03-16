// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

pub mod val;

use crate::val::{ConversionError, Value};
use miette::Diagnostic;
use qir_backend::Pauli;
use qsc_ast::ast::{
    self, Block, CallableDecl, Expr, ExprKind, Lit, Mutability, NodeId, Package, Pat, PatKind,
    Span, Stmt, StmtKind, UnOp,
};
use qsc_frontend::{
    compile::{CompileUnit, Context},
    resolve::DefId,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    ops::{ControlFlow, Neg},
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("invalid array length: {0}")]
    Count(i64, #[label("cannot be used as a length")] Span),

    #[error("nothing to evaluate; entry expression is empty")]
    EmptyExpr,

    #[error("integer-to-index conversion failed")]
    IndexVal(i64, #[label("that ain't right")] Span),

    #[error("integer-to-integer conversion failed")]
    IntegerSize(#[label] Span),

    #[error("reassigning immutable variable")]
    Mutability(#[label("variable declared as immutable")] Span),

    #[error("index out of range: {0}")]
    OutOfRange(i64, #[label("out of range")] Span),

    #[error("range with step size of zero")]
    RangeStepZero(#[label("invalid range")] Span),

    #[error("mismatched types")]
    Type(
        &'static str,
        &'static str,
        #[label("expected `{0}`, found `{1}`")] Span,
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
    #[diagnostic(help("this is an internal compiler error, not a bug in your program"))]
    Unimplemented(#[label("cannot evaluate this expression")] Span),

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

#[allow(dead_code)]
pub struct Evaluator<'a> {
    package: &'a Package,
    context: &'a Context,
    scopes: Vec<HashMap<DefId, Variable>>,
    globals: HashMap<DefId, &'a CallableDecl>,
}

impl<'a> Evaluator<'a> {
    #[must_use]
    pub fn new(unit: &'a CompileUnit) -> Self {
        Self {
            package: &unit.package,
            context: &unit.context,
            scopes: vec![],
            globals: HashMap::default(),
        }
    }

    /// Evaluates the entry expression from the current context.
    /// # Errors
    /// Returns the first error encountered during execution.
    pub fn run(&mut self) -> Result<Value, Error> {
        if let Some(expr) = &self.package.entry {
            match self.eval_expr(expr) {
                ControlFlow::Continue(val) | ControlFlow::Break(Reason::Return(val)) => Ok(val),
                ControlFlow::Break(Reason::Error(error)) => Err(error),
            }
        } else {
            Err(Error::EmptyExpr)
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
                    Err(_) => ControlFlow::Break(Reason::Error(Error::Count(size_val, size.span))),
                }?;
                ControlFlow::Continue(Value::Array(vec![item_val; s]))
            }
            ExprKind::Assign(lhs, rhs) => {
                let val = self.eval_expr(rhs)?;
                self.update_binding(lhs, val)
            }
            ExprKind::Block(block) => self.eval_block(block),
            ExprKind::Fail(msg) => ControlFlow::Break(Reason::Error(Error::UserFail(
                self.eval_expr(msg)?.try_into().with_span(msg.span)?,
                expr.span,
            ))),
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
                    _ => ControlFlow::Break(Reason::Error(Error::Type(
                        "Int or Range",
                        index_val.type_name(),
                        index_expr.span,
                    ))),
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
            ExprKind::AssignOp(_, _, _)
            | ExprKind::AssignUpdate(_, _, _)
            | ExprKind::BinOp(_, _, _)
            | ExprKind::Call(_, _)
            | ExprKind::Conjugate(_, _)
            | ExprKind::Err
            | ExprKind::Field(_, _)
            | ExprKind::For(_, _, _)
            | ExprKind::Hole
            | ExprKind::Lambda(_, _, _)
            | ExprKind::Repeat(_, _, _)
            | ExprKind::TernOp(_, _, _, _)
            | ExprKind::While(_, _) => {
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
                ControlFlow::Break(Reason::Error(Error::Unimplemented(stmt.span)))
            }
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
            UnOp::Functor(_) | UnOp::Unwrap => {
                ControlFlow::Break(Reason::Error(Error::Unimplemented(expr.span)))
            }
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
                    ControlFlow::Break(Reason::Error(Error::TupleArity(
                        tup.len(),
                        val_tup.len(),
                        pat.span,
                    )))
                }
            }
        }
    }

    fn resolve_binding(&self, id: NodeId) -> Value {
        let id = self
            .context
            .resolutions()
            .get(&id)
            .unwrap_or_else(|| panic!("{id:?} is not resolved"));

        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(id))
            .unwrap_or_else(|| panic!("{id:?} is not bound."))
            .value
            .clone()
    }

    fn update_binding(&mut self, lhs: &Expr, rhs: Value) -> ControlFlow<Reason, Value> {
        match (&lhs.kind, rhs) {
            (ExprKind::Path(path), rhs) => {
                let id = self
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
                    ControlFlow::Break(Reason::Error(Error::Mutability(path.span)))
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
            Err(_) => ControlFlow::Break(Reason::Error(Error::IntegerSize(span))),
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
