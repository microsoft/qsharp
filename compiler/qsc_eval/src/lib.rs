// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

pub mod val;

use qir_backend::Pauli;
use qsc_ast::ast::{self, Block, Expr, ExprKind, Lit, Span, Stmt, StmtKind};
use qsc_frontend::Context;
use val::Value;

#[derive(Debug)]
pub struct Error {
    pub span: Span,
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    EmptyExpr,
    Index(i64),
    IntegerSize,
    OutOfRange(i64),
    Type(&'static str),
    Unimplemented,
    UserFail(String),
}

impl Error {
    fn unimpl(span: Span) -> Result<Value, Error> {
        Err(Self {
            span,
            kind: ErrorKind::Unimplemented,
        })
    }
}

trait WithSpan {
    type Output;

    fn with_span(self, span: Span) -> Self::Output;
}

impl<T> WithSpan for Result<T, ErrorKind> {
    type Output = Result<T, Error>;

    fn with_span(self, span: Span) -> Result<T, Error> {
        self.map_err(|kind| Error { span, kind })
    }
}

pub struct Evaluator<'a> {
    context: &'a Context,
}

impl<'a> Evaluator<'a> {
    #[must_use]
    pub fn new(context: &'a Context) -> Self {
        Self { context }
    }

    /// Evaluates the entry expression from the current context.
    /// # Errors
    /// Returns the first error encountered during execution.
    pub fn run(&mut self) -> Result<Value, Error> {
        if let Some(expr) = self.context.entry() {
            self.eval_expr(expr)
        } else {
            Err(Error {
                span: Span { lo: 0, hi: 0 },
                kind: ErrorKind::EmptyExpr,
            })
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, Error> {
        match &expr.kind {
            ExprKind::Array(arr) => {
                let mut val_arr = vec![];
                for expr in arr {
                    val_arr.push(self.eval_expr(expr)?);
                }
                Ok(Value::Array(val_arr))
            }
            ExprKind::Block(block) => self.eval_block(block),
            ExprKind::Fail(msg) => Err(Error {
                span: expr.span,
                kind: ErrorKind::UserFail(self.eval_expr(msg)?.try_into().with_span(msg.span)?),
            }),
            ExprKind::If(cond, then, els) => {
                if self.eval_expr(cond)?.try_into().with_span(cond.span)? {
                    self.eval_block(then)
                } else if let Some(els) = els {
                    self.eval_expr(els)
                } else {
                    Ok(Value::Tuple(vec![]))
                }
            }
            ExprKind::Index(arr, index) => {
                let arr: Vec<_> = self.eval_expr(arr)?.try_into().with_span(arr.span)?;
                let index_val: i64 = self.eval_expr(index)?.try_into().with_span(index.span)?;
                let i: usize = index_val.try_into().map_err(|_| Error {
                    span: index.span,
                    kind: ErrorKind::Index(index_val),
                })?;
                match arr.get(i) {
                    Some(v) => Ok(v.clone()),
                    None => Err(Error {
                        span: index.span,
                        kind: ErrorKind::OutOfRange(index_val),
                    }),
                }
            }
            ExprKind::Lit(lit) => lit_to_val(lit, expr.span),
            ExprKind::Paren(expr) => self.eval_expr(expr),
            ExprKind::Range(start, step, end) => self.eval_range(start, step, end, expr.span),
            ExprKind::Tuple(tup) => {
                let mut val_tup = vec![];
                for expr in tup {
                    val_tup.push(self.eval_expr(expr)?);
                }
                Ok(Value::Tuple(val_tup))
            }
            ExprKind::ArrayRepeat(_, _)
            | ExprKind::Assign(_, _)
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
            | ExprKind::Path(_)
            | ExprKind::Repeat(_, _, _)
            | ExprKind::Return(_)
            | ExprKind::TernOp(_, _, _, _)
            | ExprKind::UnOp(_, _)
            | ExprKind::While(_, _) => Error::unimpl(expr.span),
        }
    }

    fn eval_range(
        &mut self,
        start: &Option<Box<Expr>>,
        step: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
        span: Span,
    ) -> Result<Value, Error> {
        Ok(Value::Range(
            start
                .as_ref()
                .map(|expr| self.eval_expr(expr)?.try_into().with_span(span))
                .transpose()?,
            step.as_ref()
                .map(|expr| self.eval_expr(expr)?.try_into().with_span(span))
                .transpose()?,
            end.as_ref()
                .map(|expr| self.eval_expr(expr)?.try_into().with_span(span))
                .transpose()?,
        ))
    }

    fn eval_block(&mut self, block: &Block) -> Result<Value, Error> {
        if let Some((last, most)) = block.stmts.split_last() {
            for stmt in most {
                let _ = self.eval_stmt(stmt)?;
            }
            self.eval_stmt(last)
        } else {
            Ok(Value::Tuple(vec![]))
        }
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Value, Error> {
        match &stmt.kind {
            StmtKind::Expr(expr) => self.eval_expr(expr),
            StmtKind::Semi(expr) => {
                let _ = self.eval_expr(expr)?;
                Ok(Value::Tuple(vec![]))
            }
            StmtKind::Borrow(_, _, _)
            | StmtKind::Let(_, _)
            | StmtKind::Mutable(_, _)
            | StmtKind::Use(_, _, _) => Error::unimpl(stmt.span),
        }
    }
}

fn lit_to_val(lit: &Lit, span: Span) -> Result<Value, Error> {
    Ok(match lit {
        Lit::BigInt(v) => Value::BigInt(v.clone()),
        Lit::Bool(v) => Value::Bool(*v),
        Lit::Double(v) => Value::Double(*v),
        Lit::Int(v) => Value::Int((*v).try_into().map_err(|_| Error {
            span,
            kind: ErrorKind::IntegerSize,
        })?),
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
    })
}
