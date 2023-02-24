// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

use std::{ffi::c_void, fmt::Display};

use num_bigint::BigInt;
use qir_backend::Pauli;
use qsc_ast::ast::{self, Expr, ExprKind, Lit, Result, Stmt, StmtKind};

pub enum Value {
    Array(Vec<Box<Value>>),
    BigInt(BigInt),
    Bool(bool),
    Callable,
    Double(f64),
    Int(i64),
    Pauli(Pauli),
    Qubit(*mut c_void),
    Range(Option<Box<Value>>, Option<Box<Value>>, Option<Box<Value>>),
    Result(bool),
    String(String),
    Tuple(Vec<Box<Value>>),
    Udt,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Value::Array(arr) => format!(
                    "[{}]",
                    arr.iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Value::BigInt(v) => v.to_string(),
                Value::Bool(v) => v.to_string(),
                Value::Callable => unimplemented!(),
                Value::Double(v) => {
                    if (v.floor() - v.ceil()).abs() < f64::EPSILON {
                        // The value is a whole number, which by convention is displayed with one decimal point
                        // to differentiate it from an integer value.
                        format!("{v:.1}")
                    } else {
                        format!("{v}")
                    }
                }
                Value::Int(v) => v.to_string(),
                Value::Pauli(v) => match v {
                    Pauli::I => "PauliI".to_string(),
                    Pauli::X => "PauliX".to_string(),
                    Pauli::Z => "PauliZ".to_string(),
                    Pauli::Y => "PauliY".to_string(),
                },
                Value::Qubit(v) => (*v as usize).to_string(),
                Value::Range(start, step, end) => match (start, step, end) {
                    (Some(start), Some(step), Some(end)) => format!("{start}..{step}..{end}"),
                    (Some(start), Some(step), None) => format!("{start}..{step}..."),
                    (Some(start), None, Some(end)) => format! {"{start}..{end}"},
                    (Some(start), None, None) => format!("{start}..."),
                    (None, Some(step), Some(end)) => format!("...{step}..{end}"),
                    (None, Some(step), None) => format!("...{step}..."),
                    (None, None, Some(end)) => format!("...{end}"),
                    (None, None, None) => "...".to_string(),
                },
                Value::Result(v) => {
                    if *v {
                        "One".to_string()
                    } else {
                        "Zero".to_string()
                    }
                }
                Value::String(v) => v.clone(),
                Value::Tuple(tup) => format!(
                    "({})",
                    tup.iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Value::Udt => unimplemented!(),
            }
        )
    }
}

pub struct Evaluator {}

impl Evaluator {
    #[must_use]
    pub fn eval_stmt(&mut self, stmt: &Stmt) -> Value {
        match &stmt.kind {
            StmtKind::Borrow(_, _, _) => unimplemented!(),
            StmtKind::Expr(expr) => self.eval_expr(expr),
            StmtKind::Let(_, _) => unimplemented!(),
            StmtKind::Mutable(_, _) => unimplemented!(),
            StmtKind::Semi(expr) => {
                let _ = self.eval_expr(expr);
                Value::Tuple(vec![])
            }
            StmtKind::Use(_, _, _) => unimplemented!(),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Value {
        match &expr.kind {
            ExprKind::Array(arr) => Value::Array(
                arr.iter()
                    .map(|expr| Box::new(self.eval_expr(expr)))
                    .collect::<Vec<_>>(),
            ),
            ExprKind::ArrayRepeat(_, _) => unimplemented!(),
            ExprKind::Assign(_, _) => unimplemented!(),
            ExprKind::AssignOp(_, _, _) => unimplemented!(),
            ExprKind::AssignUpdate(_, _, _) => unimplemented!(),
            ExprKind::BinOp(_, _, _) => unimplemented!(),
            ExprKind::Block(block) => {
                if let Some((last, most)) = block.stmts.split_last() {
                    for stmt in most {
                        let _ = self.eval_stmt(stmt);
                    }
                    self.eval_stmt(last)
                } else {
                    Value::Tuple(vec![])
                }
            }
            ExprKind::Call(_, _) => unimplemented!(),
            ExprKind::Conjugate(_, _) => unimplemented!(),
            ExprKind::Fail(_) => unimplemented!(),
            ExprKind::Field(_, _) => unimplemented!(),
            ExprKind::For(_, _, _) => unimplemented!(),
            ExprKind::Hole => unimplemented!(),
            ExprKind::If(_, _, _) => unimplemented!(),
            ExprKind::Index(_, _) => unimplemented!(),
            ExprKind::Lambda(_, _, _) => unimplemented!(),
            ExprKind::Lit(lit) => match lit {
                Lit::BigInt(v) => Value::BigInt(v.clone()),
                Lit::Bool(v) => Value::Bool(*v),
                Lit::Double(v) => Value::Double(*v),
                Lit::Int(v) => Value::Int(
                    (*v).try_into()
                        .expect("Integer literal does not fit in signed 64 bit value"),
                ),
                Lit::Pauli(v) => Value::Pauli(match v {
                    ast::Pauli::I => Pauli::I,
                    ast::Pauli::X => Pauli::X,
                    ast::Pauli::Y => Pauli::Y,
                    ast::Pauli::Z => Pauli::Z,
                }),
                Lit::Result(v) => Value::Result(match v {
                    Result::Zero => false,
                    Result::One => true,
                }),
                Lit::String(v) => Value::String(v.clone()),
            },
            ExprKind::Paren(expr) => self.eval_expr(expr),
            ExprKind::Path(_) => unimplemented!(),
            ExprKind::Range(start, step, end) => Value::Range(
                start.as_ref().map(|expr| Box::new(self.eval_expr(expr))),
                step.as_ref().map(|expr| Box::new(self.eval_expr(expr))),
                end.as_ref().map(|expr| Box::new(self.eval_expr(expr))),
            ),
            ExprKind::Repeat(_, _, _) => unimplemented!(),
            ExprKind::Return(_) => unimplemented!(),
            ExprKind::TernOp(_, _, _, _) => unimplemented!(),
            ExprKind::Tuple(tup) => Value::Tuple(
                tup.iter()
                    .map(|expr| Box::new(self.eval_expr(expr)))
                    .collect::<Vec<_>>(),
            ),
            ExprKind::UnOp(_, _) => unimplemented!(),
            ExprKind::While(_, _) => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::Expect;

    fn check_statement(stmt: &str, expect: Expect) {}

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
