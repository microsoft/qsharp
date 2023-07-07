// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use num_bigint::BigInt;
use qsc_hir::hir::{LocalItemId, PackageId, Pauli};
use std::{
    fmt::{self, Display, Formatter},
    iter,
    rc::Rc,
};

pub(super) const DEFAULT_RANGE_STEP: i64 = 1;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Array(Rc<[Value]>),
    BigInt(BigInt),
    Bool(bool),
    Closure(Rc<[Value]>, GlobalId, FunctorApp),
    Double(f64),
    Global(GlobalId, FunctorApp),
    Int(i64),
    Pauli(Pauli),
    Qubit(Qubit),
    Range(Option<i64>, i64, Option<i64>),
    Result(bool),
    String(Rc<str>),
    Tuple(Rc<[Value]>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlobalId {
    pub package: PackageId,
    pub item: LocalItemId,
}

impl Display for GlobalId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<item {} in package {}>", self.item, self.package)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Qubit(pub usize);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
            Value::Closure(..) => f.write_str("<closure>"),
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
            Value::Qubit(v) => write!(f, "Qubit{}", (v.0)),
            &Value::Range(start, step, end) => match (start, step, end) {
                (Some(start), DEFAULT_RANGE_STEP, Some(end)) => write!(f, "{start}..{end}"),
                (Some(start), DEFAULT_RANGE_STEP, None) => write!(f, "{start}..."),
                (Some(start), step, Some(end)) => write!(f, "{start}..{step}..{end}"),
                (Some(start), step, None) => write!(f, "{start}..{step}..."),
                (None, DEFAULT_RANGE_STEP, Some(end)) => write!(f, "...{end}"),
                (None, DEFAULT_RANGE_STEP, None) => write!(f, "..."),
                (None, step, Some(end)) => write!(f, "...{step}..{end}"),
                (None, step, None) => write!(f, "...{step}..."),
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
        }
    }
}

impl Value {
    #[must_use]
    pub fn unit() -> Self {
        Self::Tuple([].as_slice().into())
    }

    /// Convert the [Value] into an array of [Value]
    /// # Panics
    /// This will panic if the [Value] is not a [`Value::Array`].
    #[must_use]
    pub fn unwrap_array(self) -> Rc<[Self]> {
        let Value::Array(v) = self else {
            panic!("value should be Array, got {}", self.type_name());
        };
        v
    }

    /// Convert the [Value] into a `BigInt`
    /// # Panics
    /// This will panic if the [Value] is not a [`Value::BigInt`].
    #[must_use]
    pub fn unwrap_big_int(self) -> BigInt {
        let Value::BigInt(v) = self else {
            panic!("value should be BigInt, got {}", self.type_name());
        };
        v
    }

    /// Convert the [Value] into a bool
    /// # Panics
    /// This will panic if the [Value] is not a [`Value::Bool`].
    #[must_use]
    pub fn unwrap_bool(self) -> bool {
        let Value::Bool(v) = self else {
            panic!("value should be Bool, got {}", self.type_name());
        };
        v
    }

    /// Convert the [Value] into a double
    /// # Panics
    /// This will panic if the [Value] is not a [`Value::Double`].
    #[must_use]
    pub fn unwrap_double(self) -> f64 {
        let Value::Double(v) = self else {
            panic!("value should be Double, got {}", self.type_name());
        };
        v
    }

    /// Convert the [Value] into a global tuple
    /// # Panics
    /// This will panic if the [Value] is not a [`Value::Global`].
    #[must_use]
    pub fn unwrap_global(self) -> (GlobalId, FunctorApp) {
        let Value::Global(id, functor) = self else {
            panic!("value should be Global, got {}", self.type_name());
        };
        (id, functor)
    }

    /// Convert the [Value] into an integer
    /// # Panics
    /// This will panic if the [Value] is not a [`Value::Int`].
    #[must_use]
    pub fn unwrap_int(self) -> i64 {
        let Value::Int(v) = self else {
            panic!("value should be Int, got {}", self.type_name());
        };
        v
    }

    /// Convert the [Value] into a Pauli
    /// # Panics
    /// This will panic if the [Value] is not a [`Value::Pauli`].
    #[must_use]
    pub fn unwrap_pauli(self) -> Pauli {
        let Value::Pauli(v) = self else {
            panic!("value should be Pauli, got {}", self.type_name());
        };
        v
    }

    /// Convert the [Value] into a qubit
    /// # Panics
    /// This will panic if the [Value] is not a [`Value::Qubit`].
    #[must_use]
    pub fn unwrap_qubit(self) -> Qubit {
        let Value::Qubit(v) = self else {
            panic!("value should be Qubit, got {}", self.type_name());
        };
        v
    }

    /// Convert the [Value] into a range tuple
    /// # Panics
    /// This will panic if the [Value] is not a [`Value::Range`].
    #[must_use]
    pub fn unwrap_range(self) -> (Option<i64>, i64, Option<i64>) {
        let Value::Range(start, step, end) = self else {
            panic!("value should be Range, got {}", self.type_name());
        };
        (start, step, end)
    }

    /// Convert the [Value] into a measurement result
    /// # Panics
    /// This will panic if the [Value] is not a [`Value::Result`].
    #[must_use]
    pub fn unwrap_result(self) -> bool {
        let Value::Result(v) = self else {
            panic!("value should be Result, got {}", self.type_name());
        };
        v
    }

    /// Convert the [Value] into a string
    /// # Panics
    /// This will panic if the [Value] is not a [`Value::String`].
    #[must_use]
    pub fn unwrap_string(self) -> Rc<str> {
        let Value::String(v) = self else {
            panic!("value should be String, got {}", self.type_name());
        };
        v
    }

    /// Convert the [Value] into an array of [Value]
    /// # Panics
    /// This will panic if the [Value] is not a [`Value::Tuple`].
    #[must_use]
    pub fn unwrap_tuple(self) -> Rc<[Self]> {
        let Value::Tuple(v) = self else {
            panic!("value should be Tuple, got {}", self.type_name());
        };
        v
    }

    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Array(_) => "Array",
            Value::BigInt(_) => "BigInt",
            Value::Bool(_) => "Bool",
            Value::Closure(..) => "Closure",
            Value::Double(_) => "Double",
            Value::Global(..) => "Global",
            Value::Int(_) => "Int",
            Value::Pauli(_) => "Pauli",
            Value::Qubit(_) => "Qubit",
            Value::Range(..) => "Range",
            Value::Result(_) => "Result",
            Value::String(_) => "String",
            Value::Tuple(_) => "Tuple",
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
