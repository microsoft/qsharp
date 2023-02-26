// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

pub mod val;

use qir_backend::Pauli;
use qsc_ast::ast::{self, Expr, ExprKind, Lit, Span, Stmt, StmtKind};
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
    IndexErr(i64),
    IntegerSize,
    TypeError(String),
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

pub struct Evaluator {
    context: Context,
}

impl Evaluator {
    #[must_use]
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    /// Evaluates the expression from the current context.
    /// # Errors
    /// Returns the first error encountered during execution.
    pub fn run(&mut self) -> Result<Value, Error> {
        if let Some(expr) = &self.context.expr.clone() {
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
                    val_arr.push(Box::new(self.eval_expr(expr)?));
                }
                Ok(Value::Array(val_arr))
            }
            ExprKind::Block(block) => {
                if let Some((last, most)) = block.stmts.split_last() {
                    for stmt in most {
                        let _ = self.eval_stmt(stmt);
                    }
                    self.eval_stmt(last)
                } else {
                    Ok(Value::Tuple(vec![]))
                }
            }
            ExprKind::Fail(msg) => Err(Error {
                span: expr.span,
                kind: ErrorKind::UserFail(self.eval_expr(msg)?.as_string(expr.span)?),
            }),
            ExprKind::Index(arr, index) => {
                let arr = self.eval_expr(arr)?.as_array(arr.span)?;
                let index_val = self.eval_expr(index)?.as_int(index.span)?;
                let index: usize = index_val.try_into().map_err(|_| Error {
                    span: index.span,
                    kind: ErrorKind::IndexErr(index_val),
                })?;
                Ok((*arr[index]).clone())
            }
            ExprKind::Lit(lit) => Ok(match lit {
                Lit::BigInt(v) => Value::BigInt(v.clone()),
                Lit::Bool(v) => Value::Bool(*v),
                Lit::Double(v) => Value::Double(*v),
                Lit::Int(v) => Value::Int((*v).try_into().map_err(|_| Error {
                    span: expr.span,
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
            }),
            ExprKind::Paren(expr) => self.eval_expr(expr),
            ExprKind::Range(start, step, end) => Ok(Value::Range(
                start
                    .as_ref()
                    .map(|expr| self.eval_expr(expr))
                    .transpose()?
                    .map(Box::new),
                step.as_ref()
                    .map(|expr| self.eval_expr(expr))
                    .transpose()?
                    .map(Box::new),
                end.as_ref()
                    .map(|expr| self.eval_expr(expr))
                    .transpose()?
                    .map(Box::new),
            )),
            ExprKind::Tuple(tup) => {
                let mut val_tup = vec![];
                for expr in tup {
                    val_tup.push(Box::new(self.eval_expr(expr)?));
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
            | ExprKind::Field(_, _)
            | ExprKind::For(_, _, _)
            | ExprKind::Hole
            | ExprKind::If(_, _, _)
            | ExprKind::Lambda(_, _, _)
            | ExprKind::Path(_)
            | ExprKind::Repeat(_, _, _)
            | ExprKind::Return(_)
            | ExprKind::TernOp(_, _, _, _)
            | ExprKind::UnOp(_, _)
            | ExprKind::While(_, _) => Error::unimpl(expr.span),
        }
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Value, Error> {
        match &stmt.kind {
            StmtKind::Expr(expr) => self.eval_expr(expr),
            StmtKind::Semi(expr) => {
                let _ = self.eval_expr(expr);
                Ok(Value::Tuple(vec![]))
            }
            StmtKind::Borrow(_, _, _)
            | StmtKind::Let(_, _)
            | StmtKind::Mutable(_, _)
            | StmtKind::Use(_, _, _) => Error::unimpl(stmt.span),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};

    use crate::Evaluator;

    fn check_expression(expr: &str, expect: &Expect) {
        let context = qsc_frontend::compile(&[], Some(expr));
        let mut eval = Evaluator::new(context);
        expect.assert_debug_eq(&eval.run());
    }

    #[test]
    fn literal_int() {
        check_expression(
            "42",
            &expect![[r#"
            Ok(
                Int(
                    42,
                ),
            )
        "#]],
        );
    }

    #[test]
    fn literal_int_too_big() {
        check_expression(
            "9_223_372_036_854_775_808",
            &expect![[r#"
                Err(
                    Error {
                        span: Span {
                            lo: 0,
                            hi: 25,
                        },
                        kind: IntegerSize,
                    },
                )
            "#]],
        );
    }

    #[test]
    fn literal_big_int() {
        check_expression(
            "9_223_372_036_854_775_808L",
            &expect![[r#"
                Ok(
                    BigInt(
                        9223372036854775808,
                    ),
                )
            "#]],
        );
    }
}
