// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Type {
    // scalar types
    Bit(bool),
    Bool(bool),
    Duration(bool),
    Stretch(bool),

    Angle(Option<usize>, bool),
    Complex(Option<usize>, bool),
    Float(Option<usize>, bool),
    Int(Option<usize>, bool),
    UInt(Option<usize>, bool),

    // quantum
    Qubit,
    HardwareQubit,

    // magic arrays
    BitArray(ArrayDimensions, bool),
    QubitArray(ArrayDimensions),

    // proper arrays
    BoolArray(ArrayDimensions),
    DurationArray(ArrayDimensions),
    AngleArray(Option<usize>, ArrayDimensions),
    ComplexArray(Option<usize>, ArrayDimensions),
    FloatArray(Option<usize>, ArrayDimensions),
    IntArray(Option<usize>, ArrayDimensions),
    UIntArray(Option<usize>, ArrayDimensions),

    Gate(usize, usize),
    Range,
    Set,
    Void,
    #[default]
    Err,
}

impl Type {
    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(
            self,
            Type::BitArray(..)
                | Type::QubitArray(..)
                | Type::AngleArray(..)
                | Type::BoolArray(..)
                | Type::ComplexArray(..)
                | Type::DurationArray(..)
                | Type::FloatArray(..)
                | Type::IntArray(..)
                | Type::UIntArray(..)
        )
    }

    #[must_use]
    pub fn is_const(&self) -> bool {
        match self {
            Type::BitArray(_, is_const)
            | Type::Bit(is_const)
            | Type::Bool(is_const)
            | Type::Duration(is_const)
            | Type::Stretch(is_const)
            | Type::Angle(_, is_const)
            | Type::Complex(_, is_const)
            | Type::Float(_, is_const)
            | Type::Int(_, is_const)
            | Type::UInt(_, is_const) => *is_const,
            _ => false,
        }
    }

    #[must_use]
    pub fn is_inferred_output_type(&self) -> bool {
        matches!(
            self,
            Type::Bit(_)
                | Type::Int(_, _)
                | Type::UInt(_, _)
                | Type::Float(_, _)
                | Type::Angle(_, _)
                | Type::Complex(_, _)
                | Type::Bool(_)
                | Type::BitArray(_, _)
                | Type::IntArray(_, _)
                | Type::UIntArray(_, _)
                | Type::FloatArray(_, _)
                | Type::AngleArray(_, _)
                | Type::ComplexArray(_, _)
                | Type::BoolArray(_)
                | Type::Range
                | Type::Set
        )
    }

    /// Get the indexed type of a given type.
    /// For example, if the type is `Int[2][3]`, the indexed type is `Int[2]`.
    /// If the type is `Int[2]`, the indexed type is `Int`.
    /// If the type is `Int`, the indexed type is `None`.
    ///
    /// This is useful for determining the type of an array element.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn get_indexed_type(&self) -> Option<Self> {
        let ty = match self {
            Type::BitArray(dims, is_const) => match dims {
                ArrayDimensions::One(_) => Type::Bit(*is_const),
                ArrayDimensions::Two(d1, _) => Type::BitArray(ArrayDimensions::One(*d1), *is_const),
                ArrayDimensions::Three(d1, d2, _) => {
                    Type::BitArray(ArrayDimensions::Two(*d1, *d2), *is_const)
                }
                ArrayDimensions::Four(d1, d2, d3, _) => {
                    Type::BitArray(ArrayDimensions::Three(*d1, *d2, *d3), *is_const)
                }
                ArrayDimensions::Five(d1, d2, d3, d4, _) => {
                    Type::BitArray(ArrayDimensions::Four(*d1, *d2, *d3, *d4), *is_const)
                }
                ArrayDimensions::Six(d1, d2, d3, d4, d5, _) => {
                    Type::BitArray(ArrayDimensions::Five(*d1, *d2, *d3, *d4, *d5), *is_const)
                }
                ArrayDimensions::Seven(d1, d2, d3, d4, d5, d6, _) => Type::BitArray(
                    ArrayDimensions::Six(*d1, *d2, *d3, *d4, *d5, *d6),
                    *is_const,
                ),
                ArrayDimensions::Err => Type::Err,
            },
            Type::QubitArray(dims) => match dims {
                ArrayDimensions::One(_) => Type::Qubit,
                ArrayDimensions::Two(d1, _) => Type::QubitArray(ArrayDimensions::One(*d1)),
                ArrayDimensions::Three(d1, d2, _) => {
                    Type::QubitArray(ArrayDimensions::Two(*d1, *d2))
                }
                ArrayDimensions::Four(d1, d2, d3, _) => {
                    Type::QubitArray(ArrayDimensions::Three(*d1, *d2, *d3))
                }
                ArrayDimensions::Five(d1, d2, d3, d4, _) => {
                    Type::QubitArray(ArrayDimensions::Four(*d1, *d2, *d3, *d4))
                }
                ArrayDimensions::Six(d1, d2, d3, d4, d5, _) => {
                    Type::QubitArray(ArrayDimensions::Five(*d1, *d2, *d3, *d4, *d5))
                }
                ArrayDimensions::Seven(d1, d2, d3, d4, d5, d6, _) => {
                    Type::QubitArray(ArrayDimensions::Six(*d1, *d2, *d3, *d4, *d5, *d6))
                }
                ArrayDimensions::Err => Type::Err,
            },
            Type::BoolArray(dims) => match dims {
                ArrayDimensions::One(_) => Type::Bool(false),
                ArrayDimensions::Two(d1, _) => Type::BoolArray(ArrayDimensions::One(*d1)),
                ArrayDimensions::Three(d1, d2, _) => {
                    Type::BoolArray(ArrayDimensions::Two(*d1, *d2))
                }
                ArrayDimensions::Four(d1, d2, d3, _) => {
                    Type::BoolArray(ArrayDimensions::Three(*d1, *d2, *d3))
                }
                ArrayDimensions::Five(d1, d2, d3, d4, _) => {
                    Type::BoolArray(ArrayDimensions::Four(*d1, *d2, *d3, *d4))
                }
                ArrayDimensions::Six(d1, d2, d3, d4, d5, _) => {
                    Type::BoolArray(ArrayDimensions::Five(*d1, *d2, *d3, *d4, *d5))
                }
                ArrayDimensions::Seven(d1, d2, d3, d4, d5, d6, _) => {
                    Type::BoolArray(ArrayDimensions::Six(*d1, *d2, *d3, *d4, *d5, *d6))
                }
                ArrayDimensions::Err => Type::Err,
            },
            Type::AngleArray(size, dims) => match dims {
                ArrayDimensions::One(_) => Type::Angle(*size, false),
                ArrayDimensions::Two(d1, _) => Type::AngleArray(*size, ArrayDimensions::One(*d1)),
                ArrayDimensions::Three(d1, d2, _) => {
                    Type::AngleArray(*size, ArrayDimensions::Two(*d1, *d2))
                }
                ArrayDimensions::Four(d1, d2, d3, _) => {
                    Type::AngleArray(*size, ArrayDimensions::Three(*d1, *d2, *d3))
                }
                ArrayDimensions::Five(d1, d2, d3, d4, _) => {
                    Type::AngleArray(*size, ArrayDimensions::Four(*d1, *d2, *d3, *d4))
                }
                ArrayDimensions::Six(d1, d2, d3, d4, d5, _) => {
                    Type::AngleArray(*size, ArrayDimensions::Five(*d1, *d2, *d3, *d4, *d5))
                }
                ArrayDimensions::Seven(d1, d2, d3, d4, d5, d6, _) => {
                    Type::AngleArray(*size, ArrayDimensions::Six(*d1, *d2, *d3, *d4, *d5, *d6))
                }
                ArrayDimensions::Err => Type::Err,
            },
            Type::ComplexArray(size, dims) => match dims {
                ArrayDimensions::One(_) => Type::Complex(*size, false),
                ArrayDimensions::Two(d1, _) => Type::ComplexArray(*size, ArrayDimensions::One(*d1)),
                ArrayDimensions::Three(d1, d2, _) => {
                    Type::ComplexArray(*size, ArrayDimensions::Two(*d1, *d2))
                }
                ArrayDimensions::Four(d1, d2, d3, _) => {
                    Type::ComplexArray(*size, ArrayDimensions::Three(*d1, *d2, *d3))
                }
                ArrayDimensions::Five(d1, d2, d3, d4, _) => {
                    Type::ComplexArray(*size, ArrayDimensions::Four(*d1, *d2, *d3, *d4))
                }
                ArrayDimensions::Six(d1, d2, d3, d4, d5, _) => {
                    Type::ComplexArray(*size, ArrayDimensions::Five(*d1, *d2, *d3, *d4, *d5))
                }
                ArrayDimensions::Seven(d1, d2, d3, d4, d5, d6, _) => {
                    Type::ComplexArray(*size, ArrayDimensions::Six(*d1, *d2, *d3, *d4, *d5, *d6))
                }
                ArrayDimensions::Err => Type::Err,
            },
            Type::DurationArray(dims) => match dims {
                ArrayDimensions::One(_) => Type::Duration(false),
                ArrayDimensions::Two(d1, _) => Type::DurationArray(ArrayDimensions::One(*d1)),
                ArrayDimensions::Three(d1, d2, _) => {
                    Type::DurationArray(ArrayDimensions::Two(*d1, *d2))
                }
                ArrayDimensions::Four(d1, d2, d3, _) => {
                    Type::DurationArray(ArrayDimensions::Three(*d1, *d2, *d3))
                }
                ArrayDimensions::Five(d1, d2, d3, d4, _) => {
                    Type::DurationArray(ArrayDimensions::Four(*d1, *d2, *d3, *d4))
                }
                ArrayDimensions::Six(d1, d2, d3, d4, d5, _) => {
                    Type::DurationArray(ArrayDimensions::Five(*d1, *d2, *d3, *d4, *d5))
                }
                ArrayDimensions::Seven(d1, d2, d3, d4, d5, d6, _) => {
                    Type::DurationArray(ArrayDimensions::Six(*d1, *d2, *d3, *d4, *d5, *d6))
                }
                ArrayDimensions::Err => Type::Err,
            },
            Type::FloatArray(size, dims) => match dims {
                ArrayDimensions::One(_) => Type::Float(*size, false),
                ArrayDimensions::Two(d1, _) => Type::FloatArray(*size, ArrayDimensions::One(*d1)),
                ArrayDimensions::Three(d1, d2, _) => {
                    Type::FloatArray(*size, ArrayDimensions::Two(*d1, *d2))
                }
                ArrayDimensions::Four(d1, d2, d3, _) => {
                    Type::FloatArray(*size, ArrayDimensions::Three(*d1, *d2, *d3))
                }
                ArrayDimensions::Five(d1, d2, d3, d4, _) => {
                    Type::FloatArray(*size, ArrayDimensions::Four(*d1, *d2, *d3, *d4))
                }
                ArrayDimensions::Six(d1, d2, d3, d4, d5, _) => {
                    Type::FloatArray(*size, ArrayDimensions::Five(*d1, *d2, *d3, *d4, *d5))
                }
                ArrayDimensions::Seven(d1, d2, d3, d4, d5, d6, _) => {
                    Type::FloatArray(*size, ArrayDimensions::Six(*d1, *d2, *d3, *d4, *d5, *d6))
                }
                ArrayDimensions::Err => Type::Err,
            },
            Type::IntArray(size, dims) => match dims {
                ArrayDimensions::One(_) => Type::Int(*size, false),
                ArrayDimensions::Two(d1, _) => Type::IntArray(*size, ArrayDimensions::One(*d1)),
                ArrayDimensions::Three(d1, d2, _) => {
                    Type::IntArray(*size, ArrayDimensions::Two(*d1, *d2))
                }
                ArrayDimensions::Four(d1, d2, d3, _) => {
                    Type::IntArray(*size, ArrayDimensions::Three(*d1, *d2, *d3))
                }
                ArrayDimensions::Five(d1, d2, d3, d4, _) => {
                    Type::IntArray(*size, ArrayDimensions::Four(*d1, *d2, *d3, *d4))
                }
                ArrayDimensions::Six(d1, d2, d3, d4, d5, _) => {
                    Type::IntArray(*size, ArrayDimensions::Five(*d1, *d2, *d3, *d4, *d5))
                }
                ArrayDimensions::Seven(d1, d2, d3, d4, d5, d6, _) => {
                    Type::IntArray(*size, ArrayDimensions::Six(*d1, *d2, *d3, *d4, *d5, *d6))
                }
                ArrayDimensions::Err => Type::Err,
            },
            Type::UIntArray(size, dims) => match dims {
                ArrayDimensions::One(_) => Type::UInt(*size, false),
                ArrayDimensions::Two(d1, _) => Type::UIntArray(*size, ArrayDimensions::One(*d1)),
                ArrayDimensions::Three(d1, d2, _) => {
                    Type::UIntArray(*size, ArrayDimensions::Two(*d1, *d2))
                }
                ArrayDimensions::Four(d1, d2, d3, _) => {
                    Type::UIntArray(*size, ArrayDimensions::Three(*d1, *d2, *d3))
                }
                ArrayDimensions::Five(d1, d2, d3, d4, _) => {
                    Type::UIntArray(*size, ArrayDimensions::Four(*d1, *d2, *d3, *d4))
                }
                ArrayDimensions::Six(d1, d2, d3, d4, d5, _) => {
                    Type::UIntArray(*size, ArrayDimensions::Five(*d1, *d2, *d3, *d4, *d5))
                }
                ArrayDimensions::Seven(d1, d2, d3, d4, d5, d6, _) => {
                    Type::UIntArray(*size, ArrayDimensions::Six(*d1, *d2, *d3, *d4, *d5, *d6))
                }
                ArrayDimensions::Err => Type::Err,
            },
            _ => return None,
        };
        Some(ty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ArrayDimensions {
    One(usize),
    Two(usize, usize),
    Three(usize, usize, usize),
    Four(usize, usize, usize, usize),
    Five(usize, usize, usize, usize, usize),
    Six(usize, usize, usize, usize, usize, usize),
    Seven(usize, usize, usize, usize, usize, usize, usize),
    #[default]
    Err,
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
            ArrayDimensions::Err => write!(f, "Invalid array dimensions"),
        }
    }
}
