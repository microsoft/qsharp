// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{ffi::c_void, fmt::Display};

use crate::globals::GlobalId;
use num_bigint::BigInt;
use qir_backend::{Pauli, __quantum__rt__qubit_release};

#[derive(Clone, Debug)]
pub enum Value {
    Array(Vec<Value>),
    BigInt(BigInt),
    Bool(bool),
    Closure,
    Double(f64),
    Global(GlobalId, FunctorApp),
    Int(i64),
    Pauli(Pauli),
    Qubit(*mut c_void),
    Range(Option<i64>, Option<i64>, Option<i64>),
    Result(bool),
    String(String),
    Tuple(Vec<Value>),
    Udt,
}

#[derive(Clone, Debug, Default)]
pub struct FunctorApp {
    /// An invocation is either adjoint or not, with each successive use of `Adjoint` functor switching
    /// between the two, so a bool is sufficient to track.
    pub adjoint: bool,

    /// An invocation can have multiple `Controlled` functors with each one adding another layer of updates
    /// to the argument tuple, so the functor application must be tracked with a count.
    pub controlled: u8,
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
            Value::Closure => todo!(),
            Value::Double(v) => {
                if (v.floor() - v.ceil()).abs() < f64::EPSILON {
                    // The value is a whole number, which by convention is displayed with one decimal point
                    // to differentiate it from an integer value.
                    write!(f, "{v:.1}")
                } else {
                    write!(f, "{v}")
                }
            }
            Value::Global(g, functor) => write!(f, "{g:?}({functor:?})"),
            Value::Int(v) => write!(f, "{v}"),
            Value::Pauli(v) => match v {
                Pauli::I => write!(f, "PauliI"),
                Pauli::X => write!(f, "PauliX"),
                Pauli::Z => write!(f, "PauliZ"),
                Pauli::Y => write!(f, "PauliY"),
            },
            Value::Qubit(v) => write!(f, "Qubit_{}", (*v as usize)),
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

pub struct ConversionError {
    pub expected: &'static str,
    pub actual: &'static str,
}

impl TryFrom<Value> for i64 {
    type Error = ConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Int(v) = value {
            Ok(v)
        } else {
            Err(ConversionError {
                expected: "Int",
                actual: value.type_name(),
            })
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = ConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Bool(v) = value {
            Ok(v)
        } else {
            Err(ConversionError {
                expected: "Bool",
                actual: value.type_name(),
            })
        }
    }
}

impl TryFrom<Value> for String {
    type Error = ConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::String(v) = value {
            Ok(v)
        } else {
            Err(ConversionError {
                expected: "String",
                actual: value.type_name(),
            })
        }
    }
}

impl Value {
    pub const UNIT: Self = Self::Tuple(Vec::new());

    /// Convert the [Value] into an array of [Value]
    /// # Errors
    /// This will return an error if the [Value] is not a [`Value::Array`].
    pub fn try_into_array(self) -> Result<Vec<Self>, ConversionError> {
        if let Value::Array(v) = self {
            Ok(v)
        } else {
            Err(ConversionError {
                expected: "Array",
                actual: self.type_name(),
            })
        }
    }

    /// Convert the [Value] into an tuple of [Value]
    /// # Errors
    /// This will return an error if the [Value] is not a [`Value::Tuple`].
    pub fn try_into_tuple(self) -> Result<Vec<Self>, ConversionError> {
        if let Value::Tuple(v) = self {
            Ok(v)
        } else {
            Err(ConversionError {
                expected: "Tuple",
                actual: self.type_name(),
            })
        }
    }

    pub fn release(&mut self) {
        if let Value::Qubit(q) = self {
            __quantum__rt__qubit_release(*q);
        }
    }

    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Array(_) => "Array",
            Value::BigInt(_) => "BigInt",
            Value::Bool(_) => "Bool",
            Value::Closure => "Closure",
            Value::Double(_) => "Double",
            Value::Global(_, _) => "Global",
            Value::Int(_) => "Int",
            Value::Pauli(_) => "Pauli",
            Value::Qubit(_) => "Qubit",
            Value::Range(_, _, _) => "Range",
            Value::Result(_) => "Result",
            Value::String(_) => "String",
            Value::Tuple(_) => "Tuple",
            Value::Udt => "Udt",
        }
    }
}

fn join<'a>(
    f: &mut std::fmt::Formatter<'_>,
    mut vals: impl Iterator<Item = &'a Value>,
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
