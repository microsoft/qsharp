// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

pub mod val;

use std::{
    collections::{hash_map::Entry, HashMap},
    ops::ControlFlow,
};

use qir_backend::Pauli;
use qsc_ast::ast::{
    self, Block, CallableDecl, Expr, ExprKind, Lit, Mutability, NodeId, Package, Pat, PatKind,
    Span, Stmt, StmtKind,
};
use qsc_frontend::{
    compile::{CompileUnit, Context},
    resolve::DefId,
};
use val::{ConversionError, Value};

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
    EmptyExpr,
    Index(i64),
    Mutability,
    OutOfRange(i64),
    Type(&'static str, &'static str),
    TupleArity(usize, usize),
    Unassignable,
    Unimplemented,
    UserFail(String),
}

trait WithSpan {
    type Output;

    fn with_span(self, span: Span) -> Self::Output;
}

impl<T> WithSpan for Result<T, ConversionError> {
    type Output = ControlFlow<Reason, T>;

    fn with_span(self, span: Span) -> ControlFlow<Reason, T> {
        match self {
            Ok(c) => ControlFlow::Continue(c),
            Err(e) => {
                ControlFlow::Break(Reason::Error(span, ErrorKind::Type(e.expected, e.actual)))
            }
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
            ExprKind::Assign(lhs, rhs) => {
                let val = self.eval_expr(rhs)?;
                self.update_binding(lhs, val)
            }
            ExprKind::Block(block) => self.eval_block(block),
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
            ExprKind::Index(arr, index) => {
                let arr = self.eval_expr(arr)?.try_into_array().with_span(arr.span)?;
                let index_val: i64 = self.eval_expr(index)?.try_into().with_span(index.span)?;
                let i: usize = match index_val.try_into() {
                    Ok(i) => ControlFlow::Continue(i),
                    Err(_) => {
                        ControlFlow::Break(Reason::Error(index.span, ErrorKind::Index(index_val)))
                    }
                }?;
                match arr.get(i) {
                    Some(v) => ControlFlow::Continue(v.clone()),
                    None => ControlFlow::Break(Reason::Error(
                        index.span,
                        ErrorKind::OutOfRange(index_val),
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
            ExprKind::ArrayRepeat(_, _)
            | ExprKind::AssignOp(_, _, _)
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
            | ExprKind::UnOp(_, _)
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

    fn bind_value(
        &mut self,
        pat: &Pat,
        value: Value,
        span: Span,
        mutability: Mutability,
    ) -> ControlFlow<Reason, ()> {
        match &pat.kind {
            PatKind::Bind(variable, _) => {
                let key = self
                    .context
                    .resolutions()
                    .get(&variable.id)
                    .unwrap_or_else(|| panic!("{:?} is not resolved", variable.id));

                let scope = self.scopes.last_mut().expect("Binding requires a scope.");
                match scope.entry(*key) {
                    Entry::Vacant(entry) => entry.insert(Variable { value, mutability }),
                    Entry::Occupied(_) => panic!("{key:?} is already bound"),
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

    fn resolve_binding(&self, id: NodeId) -> Value {
        let key = self
            .context
            .resolutions()
            .get(&id)
            .unwrap_or_else(|| panic!("{id:?} is not resolved"));

        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(key))
            .unwrap_or_else(|| panic!("{key:?} is not bound."))
            .value
            .clone()
    }

    fn update_binding(&mut self, lhs: &Expr, rhs: Value) -> ControlFlow<Reason, Value> {
        match (&lhs.kind, rhs) {
            (ExprKind::Path(path), rhs) => {
                let key = self
                    .context
                    .resolutions()
                    .get(&path.id)
                    .unwrap_or_else(|| panic!("{:?} is not resolved", path.id));

                let mut variable = self
                    .scopes
                    .iter_mut()
                    .rev()
                    .find_map(|scope| scope.get_mut(key))
                    .unwrap_or_else(|| panic!("{key:?} is not bound"));

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
