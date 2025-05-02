// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt::{self, Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Complex {
    pub real: f64,
    pub imaginary: f64,
}

impl Complex {
    pub fn new(real: f64, imaginary: f64) -> Self {
        Self { real, imaginary }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Type {
    Angle(bool),
    Bool(bool),
    BigInt(bool),
    Complex(bool),
    Int(bool),
    Double(bool),
    Qubit,
    Result(bool),
    Tuple(Vec<Type>),
    Range,
    BoolArray(ArrayDimensions, bool),
    BigIntArray(ArrayDimensions, bool),
    IntArray(ArrayDimensions, bool),
    DoubleArray(ArrayDimensions),
    ComplexArray(ArrayDimensions, bool),
    AngleArray(ArrayDimensions, bool),
    QubitArray(ArrayDimensions),
    ResultArray(ArrayDimensions, bool),
    TupleArray(ArrayDimensions, Vec<Type>),
    /// Function or operation, with the number of classical parameters and qubits.
    Callable(CallableKind, u32, u32),
    #[default]
    Err,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallableKind {
    /// A function.
    Function,
    /// An operation.
    Operation,
}

impl Display for CallableKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CallableKind::Function => write!(f, "Function"),
            CallableKind::Operation => write!(f, "Operation"),
        }
    }
}

/// QASM supports up to seven dimensions, but we are going to limit it to three.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayDimensions {
    One(usize),
    Two(usize, usize),
    Three(usize, usize, usize),
    Four(usize, usize, usize, usize),
    Five(usize, usize, usize, usize, usize),
    Six(usize, usize, usize, usize, usize, usize),
    Seven(usize, usize, usize, usize, usize, usize, usize),
}

impl From<&crate::semantic::types::ArrayDimensions> for ArrayDimensions {
    fn from(value: &crate::semantic::types::ArrayDimensions) -> Self {
        match value {
            crate::semantic::types::ArrayDimensions::One(dim) => {
                ArrayDimensions::One(*dim as usize)
            }
            crate::semantic::types::ArrayDimensions::Two(dim1, dim2) => {
                ArrayDimensions::Two(*dim1 as usize, *dim2 as usize)
            }
            crate::semantic::types::ArrayDimensions::Three(dim1, dim2, dim3) => {
                ArrayDimensions::Three(*dim1 as usize, *dim2 as usize, *dim3 as usize)
            }
            crate::semantic::types::ArrayDimensions::Four(dim1, dim2, dim3, dim4) => {
                ArrayDimensions::Four(
                    *dim1 as usize,
                    *dim2 as usize,
                    *dim3 as usize,
                    *dim4 as usize,
                )
            }
            crate::semantic::types::ArrayDimensions::Five(dim1, dim2, dim3, dim4, dim5) => {
                ArrayDimensions::Five(
                    *dim1 as usize,
                    *dim2 as usize,
                    *dim3 as usize,
                    *dim4 as usize,
                    *dim5 as usize,
                )
            }
            crate::semantic::types::ArrayDimensions::Six(dim1, dim2, dim3, dim4, dim5, dim6) => {
                ArrayDimensions::Six(
                    *dim1 as usize,
                    *dim2 as usize,
                    *dim3 as usize,
                    *dim4 as usize,
                    *dim5 as usize,
                    *dim6 as usize,
                )
            }
            crate::semantic::types::ArrayDimensions::Seven(
                dim1,
                dim2,
                dim3,
                dim4,
                dim5,
                dim6,
                dim7,
            ) => ArrayDimensions::Seven(
                *dim1 as usize,
                *dim2 as usize,
                *dim3 as usize,
                *dim4 as usize,
                *dim5 as usize,
                *dim6 as usize,
                *dim7 as usize,
            ),
            crate::semantic::types::ArrayDimensions::Err => {
                unimplemented!("Array dimensions greater than seven are not supported.")
            }
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::Angle(_) => write!(f, "Angle"),
            Type::Bool(_) => write!(f, "bool"),
            Type::BigInt(_) => write!(f, "BigInt"),
            Type::Complex(_) => write!(f, "Complex"),
            Type::Int(_) => write!(f, "Int"),
            Type::Double(_) => write!(f, "Double"),
            Type::Qubit => write!(f, "Qubit"),
            Type::Range => write!(f, "Range"),
            Type::Result(_) => write!(f, "Result"),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{ty}")?;
                }
                write!(f, ")")
            }
            Type::BoolArray(dim, _) => write!(f, "bool{dim}"),
            Type::BigIntArray(dim, _) => write!(f, "BigInt{dim}"),
            Type::IntArray(dim, _) => write!(f, "Int{dim}"),
            Type::DoubleArray(dim) => write!(f, "Double{dim}"),
            Type::ComplexArray(dim, _) => write!(f, "Complex{dim}"),
            Type::AngleArray(dim, _) => write!(f, "Angle{dim}"),
            Type::QubitArray(dim) => write!(f, "Qubit{dim}"),
            Type::ResultArray(dim, _) => write!(f, "Result{dim}"),
            Type::TupleArray(dim, types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{ty}")?;
                }
                write!(f, "){dim}")
            }
            Type::Callable(kind, num_classical, num_qubits) => {
                write!(f, "Callable({kind}, {num_classical}, {num_qubits})")
            }
            Type::Err => write!(f, "Err"),
        }
    }
}

impl Display for ArrayDimensions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ArrayDimensions::One(..) => write!(f, "[]"),
            ArrayDimensions::Two(..) => write!(f, "[][]"),
            ArrayDimensions::Three(..) => write!(f, "[][][]"),
            ArrayDimensions::Four(..) => write!(f, "[][][][]"),
            ArrayDimensions::Five(..) => write!(f, "[][][][][]"),
            ArrayDimensions::Six(..) => write!(f, "[][][][][][]"),
            ArrayDimensions::Seven(..) => write!(f, "[][][][][][][]"),
        }
    }
}
