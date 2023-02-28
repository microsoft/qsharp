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
    Range(Option<i64>, Option<i64>, Option<i64>),
    Result(bool),
    String(String),
    Tuple(Vec<Box<Value>>),
    Udt,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Array(arr) => {
                write!(f, "[")?;
                join(f, arr.iter(), ", ")?;
                write!(f, "]")
            }
            Value::BigInt(v) => write!(f, "{v}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Callable => todo!(),
            Value::Double(v) => {
                if (v.floor() - v.ceil()).abs() < f64::EPSILON {
                    // The value is a whole number, which by convention is displayed with one decimal point
                    // to differentiate it from an integer value.
                    write!(f, "{v:.1}")
                } else {
                    write!(f, "{v}")
                }
            }
            Value::Int(v) => write!(f, "{v}"),
            Value::Pauli(v) => match v {
                Pauli::I => write!(f, "PauliI"),
                Pauli::X => write!(f, "PauliX"),
                Pauli::Z => write!(f, "PauliZ"),
                Pauli::Y => write!(f, "PauliY"),
            },
            Value::Qubit(v) => write!(f, "{}", (*v as usize)),
            Value::Range(start, step, end) => match (start, step, end) {
                (Some(start), Some(step), Some(end)) => write!(f, "{start}..{step}..{end}"),
                (Some(start), Some(step), None) => write!(f, "{start}..{step}..."),
                (Some(start), None, Some(end)) => write!(f, "{start}..{end}"),
                (Some(start), None, None) => write!(f, "{start}..."),
                (None, Some(step), Some(end)) => write!(f, "...{step}..{end}"),
                (None, Some(step), None) => write!(f, "...{step}..."),
                (None, None, Some(end)) => write!(f, "...{end}"),
                (None, None, None) => write!(f, "..."),
            },
            Value::Result(v) => {
                if *v {
                    write!(f, "One")
                } else {
                    write!(f, "Zero")
                }
            }
            Value::String(v) => write!(f, "{v}"),
            Value::Tuple(tup) => {
                write!(f, "(")?;
                join(f, tup.iter(), ", ")?;
                write!(f, ")")
            }
            Value::Udt => todo!(),
        }
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
            Err(Error {
                span,
                kind: ErrorKind::Type("String"),
            })
        }
    }

    /// Unwraps the [Value] to an integer.
    /// # Errors
    /// Will return a type error if the [Value] is not an integer.
    pub fn as_int(&self, span: Span) -> Result<i64, Error> {
        if let Value::Int(v) = self {
            Ok(*v)
        } else {
            Err(Error {
                span,
                kind: ErrorKind::Type("Int"),
            })
        }
    }

    /// Unwraps the [Value] to a Boolean.
    /// # Errors
    /// Will return a type error if the [Value] is not a Boolean.
    pub fn as_bool(&self, span: Span) -> Result<bool, Error> {
        if let Value::Bool(b) = self {
            Ok(*b)
        } else {
            Err(Error {
                span,
                kind: ErrorKind::Type("Bool"),
            })
        }
    }

    /// Unwraps the [Value] to an Array.
    /// # Errors
    /// Will return a type error if the [Value] is not an integer.
    pub fn as_array(&self, span: Span) -> Result<Vec<Box<Value>>, Error> {
        if let Value::Array(v) = self {
            Ok((*v).clone())
        } else {
            Err(Error {
                span,
                kind: ErrorKind::Type("Array"),
            })
        }
    }
}

fn join<'a>(
    f: &mut std::fmt::Formatter<'_>,
    mut vals: impl Iterator<Item = &'a Box<Value>>,
    sep: &str,
) -> std::fmt::Result {
    if let Some(v) = vals.next() {
        v.fmt(f)?;
    }
    for v in vals {
        write!(f, "{sep}")?;
        v.fmt(f)?;
    }
    Ok(())
}
