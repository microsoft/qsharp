// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{ffi::c_void, fmt::Display};

use num_bigint::BigInt;
use qir_backend::Pauli;
use qsc_ast::ast::Span;

use crate::{Error, ErrorKind};

#[derive(Clone, Debug)]
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

impl Value {
    /// Unwraps the [Value] to a string.
    /// # Errors
    /// Will return a type error if the [Value] is not a string.
    pub fn as_string(&self, span: Span) -> Result<String, Error> {
        if let Value::String(v) = self {
            Ok(v.clone())
        } else {
            Err(Error{span, kind: ErrorKind::TypeError("String".to_string())})
        }
    }

    /// Unwraps the [Value] to an integer.
    /// # Errors
    /// Will return a type error if the [Value] is not an integer.
    pub fn as_int(&self, span: Span) -> Result<i64, Error> {
        if let Value::Int(v) = self {
            Ok(*v)
        } else {
            Err(Error{span, kind: ErrorKind::TypeError("Int".to_string())})
        }
    }

    /// Unwraps the [Value] to an Array.
    /// # Errors
    /// Will return a type error if the [Value] is not an integer.
    pub fn as_array(&self, span: Span) -> Result<Vec<Box<Value>>, Error> {
        if let Value::Array(v) = self {
            Ok((*v).clone())
        } else {
            Err(Error{span, kind: ErrorKind::TypeError("Array".to_string())})
        }
    }
}