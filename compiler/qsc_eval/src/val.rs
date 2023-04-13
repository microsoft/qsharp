// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    ffi::c_void,
    fmt::{self, Display, Formatter},
    iter,
};

use num_bigint::BigInt;
use qsc_ast::ast::Pauli;
use qsc_passes::globals::GlobalId;

pub(super) type Qubit = *mut c_void;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Array(Vec<Value>),
    BigInt(BigInt),
    Bool(bool),
    Closure,
    Double(f64),
    Global(GlobalId, FunctorApp),
    Int(i64),
    Pauli(Pauli),
    Qubit(Qubit),
    Range(Option<i64>, Option<i64>, Option<i64>),
    Result(bool),
    String(String),
    Tuple(Vec<Value>),
    Udt,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FunctorApp {
    /// An invocation is either adjoint or not, with each successive use of `Adjoint` functor switching
    /// between the two, so a bool is sufficient to track.
    pub adjoint: bool,

    /// An invocation can have multiple `Controlled` functors with each one adding another layer of updates
    /// to the argument tuple, so the functor application must be tracked with a count.
    pub controlled: u8,
}

impl Display for FunctorApp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let controlleds = iter::repeat("Controlled").take(self.controlled.into());
        let adjoint = iter::once("Adjoint").filter(|_| self.adjoint);
        join(f, controlleds.chain(adjoint), " ")
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Array(arr) => {
                write!(f, "[")?;
                join(f, arr.iter(), ", ")?;
                write!(f, "]")
            }
            Value::BigInt(v) => write!(f, "{v}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Closure => todo!("https://github.com/microsoft/qsharp/issues/151"),
            Value::Double(v) => {
                if (v.floor() - v.ceil()).abs() < f64::EPSILON {
                    // The value is a whole number, which by convention is displayed with one decimal point
                    // to differentiate it from an integer value.
                    write!(f, "{v:.1}")
                } else {
                    write!(f, "{v}")
                }
            }
            Value::Global(id, functor) if functor == &FunctorApp::default() => id.fmt(f),
            Value::Global(id, functor) => write!(f, "{functor} {id}"),
            Value::Int(v) => write!(f, "{v}"),
            Value::Pauli(v) => match v {
                Pauli::I => write!(f, "PauliI"),
                Pauli::X => write!(f, "PauliX"),
                Pauli::Z => write!(f, "PauliZ"),
                Pauli::Y => write!(f, "PauliY"),
            },
            Value::Qubit(v) => write!(f, "Qubit{}", (*v as usize)),
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
                if tup.len() == 1 {
                    write!(f, ",")?;
                }
                write!(f, ")")
            }
            Value::Udt => todo!("https://github.com/microsoft/qsharp/issues/148"),
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

impl TryFrom<Value> for BigInt {
    type Error = ConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::BigInt(v) = value {
            Ok(v)
        } else {
            Err(ConversionError {
                expected: "BigInt",
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

impl TryFrom<Value> for *mut c_void {
    type Error = ConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Qubit(q) = value {
            Ok(q)
        } else {
            Err(ConversionError {
                expected: "Qubit",
                actual: value.type_name(),
            })
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = ConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Double(v) = value {
            Ok(v)
        } else {
            Err(ConversionError {
                expected: "Double",
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

fn join(f: &mut Formatter, mut vals: impl Iterator<Item = impl Display>, sep: &str) -> fmt::Result {
    if let Some(v) = vals.next() {
        v.fmt(f)?;
    }
    for v in vals {
        write!(f, "{sep}")?;
        v.fmt(f)?;
    }
    Ok(())
}
