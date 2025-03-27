// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{cmp::max, rc::Rc};

use core::fmt;
use std::fmt::{Display, Formatter};

use crate::parser::ast as syntax;

use super::ast::LiteralKind;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Type {
    // scalar types
    Bit(bool),
    Bool(bool),
    Duration(bool),
    Stretch(bool),

    Angle(Option<u32>, bool),
    Complex(Option<u32>, bool),
    Float(Option<u32>, bool),
    Int(Option<u32>, bool),
    UInt(Option<u32>, bool),

    // quantum
    Qubit,
    HardwareQubit,

    // magic arrays
    BitArray(ArrayDimensions, bool),
    QubitArray(ArrayDimensions),

    // proper arrays
    BoolArray(ArrayDimensions),
    DurationArray(ArrayDimensions),
    AngleArray(Option<u32>, ArrayDimensions),
    ComplexArray(Option<u32>, ArrayDimensions),
    FloatArray(Option<u32>, ArrayDimensions),
    IntArray(Option<u32>, ArrayDimensions),
    UIntArray(Option<u32>, ArrayDimensions),

    // realistically the sizes could be u3
    Gate(u32, u32),
    Function(Rc<[Type]>, Option<Rc<Type>>),
    Range,
    Set,
    Void,
    #[default]
    Err,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Bit(is_const) => write!(f, "Bit({is_const})"),
            Type::Bool(is_const) => write!(f, "Bool({is_const})"),
            Type::Duration(is_const) => write!(f, "Duration({is_const})"),
            Type::Stretch(is_const) => write!(f, "Stretch({is_const})"),
            Type::Angle(width, is_const) => write!(f, "Angle({width:?}, {is_const})"),
            Type::Complex(width, is_const) => write!(f, "Complex({width:?}, {is_const})"),
            Type::Float(width, is_const) => write!(f, "Float({width:?}, {is_const})"),
            Type::Int(width, is_const) => write!(f, "Int({width:?}, {is_const})"),
            Type::UInt(width, is_const) => write!(f, "UInt({width:?}, {is_const})"),
            Type::Qubit => write!(f, "Qubit"),
            Type::HardwareQubit => write!(f, "HardwareQubit"),
            Type::BitArray(dims, is_const) => write!(f, "BitArray({dims:?}, {is_const})"),
            Type::QubitArray(dims) => write!(f, "QubitArray({dims:?})"),
            Type::BoolArray(dims) => write!(f, "BoolArray({dims:?})"),
            Type::DurationArray(dims) => write!(f, "DurationArray({dims:?})"),
            Type::AngleArray(width, dims) => write!(f, "AngleArray({width:?}, {dims:?})"),
            Type::ComplexArray(width, dims) => write!(f, "ComplexArray({width:?}, {dims:?})"),
            Type::FloatArray(width, dims) => write!(f, "FloatArray({width:?}, {dims:?})"),
            Type::IntArray(width, dims) => write!(f, "IntArray({width:?}, {dims:?})"),
            Type::UIntArray(width, dims) => write!(f, "UIntArray({width:?}, {dims:?})"),
            Type::Gate(cargs, qargs) => write!(f, "Gate({cargs}, {qargs})"),
            Type::Function(params_ty, return_ty) => {
                write!(f, "Function({params_ty:?}) -> {return_ty:?}")
            }
            Type::Range => write!(f, "Range"),
            Type::Set => write!(f, "Set"),
            Type::Void => write!(f, "Void"),
            Type::Err => write!(f, "Err"),
        }
    }
}

impl Type {
    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(
            self,
            Type::AngleArray(..)
                | Type::BitArray(..)
                | Type::BoolArray(..)
                | Type::ComplexArray(..)
                | Type::DurationArray(..)
                | Type::FloatArray(..)
                | Type::IntArray(..)
                | Type::QubitArray(..)
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
    pub fn width(&self) -> Option<u32> {
        match self {
            Type::Angle(w, _)
            | Type::Complex(w, _)
            | Type::Float(w, _)
            | Type::Int(w, _)
            | Type::UInt(w, _) => *w,
            _ => None,
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

    #[must_use]
    pub fn num_dims(&self) -> usize {
        match self {
            Type::AngleArray(_, dims)
            | Type::BitArray(dims, _)
            | Type::BoolArray(dims)
            | Type::DurationArray(dims)
            | Type::ComplexArray(_, dims)
            | Type::FloatArray(_, dims)
            | Type::IntArray(_, dims)
            | Type::QubitArray(dims)
            | Type::UIntArray(_, dims) => dims.num_dims(),
            _ => 0,
        }
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
            Type::BitArray(dims, is_const) => indexed_type_builder(
                || Type::Bit(*is_const),
                |d| Type::BitArray(d, *is_const),
                dims,
            ),
            Type::QubitArray(dims) => indexed_type_builder(|| Type::Qubit, Type::QubitArray, dims),
            Type::BoolArray(dims) => {
                indexed_type_builder(|| Type::Bool(false), Type::BoolArray, dims)
            }
            Type::AngleArray(size, dims) => indexed_type_builder(
                || Type::Angle(*size, false),
                |d| Type::AngleArray(*size, d),
                dims,
            ),
            Type::ComplexArray(size, dims) => indexed_type_builder(
                || Type::Complex(*size, false),
                |d| Type::ComplexArray(*size, d),
                dims,
            ),
            Type::DurationArray(dims) => {
                indexed_type_builder(|| Type::Duration(false), Type::DurationArray, dims)
            }
            Type::FloatArray(size, dims) => indexed_type_builder(
                || Type::Float(*size, false),
                |d| Type::FloatArray(*size, d),
                dims,
            ),
            Type::IntArray(size, dims) => indexed_type_builder(
                || Type::Int(*size, false),
                |d| Type::IntArray(*size, d),
                dims,
            ),
            Type::UIntArray(size, dims) => indexed_type_builder(
                || Type::UInt(*size, false),
                |d| Type::UIntArray(*size, d),
                dims,
            ),
            _ => return None,
        };
        Some(ty)
    }

    pub(crate) fn as_const(&self) -> Type {
        match self {
            Type::Bit(_) => Self::Bit(true),
            Type::Bool(_) => Self::Bool(true),
            Type::Duration(_) => Self::Duration(true),
            Type::Stretch(_) => Self::Stretch(true),
            Type::Angle(w, _) => Self::Angle(*w, true),
            Type::Complex(w, _) => Self::Complex(*w, true),
            Type::Float(w, _) => Self::Float(*w, true),
            Type::Int(w, _) => Self::Int(*w, true),
            Type::UInt(w, _) => Self::UInt(*w, true),
            Type::BitArray(dims, _) => Self::BitArray(dims.clone(), true),
            _ => self.clone(),
        }
    }

    pub(crate) fn as_non_const(&self) -> Type {
        match self {
            Type::Bit(_) => Self::Bit(false),
            Type::Bool(_) => Self::Bool(false),
            Type::Duration(_) => Self::Duration(false),
            Type::Stretch(_) => Self::Stretch(false),
            Type::Angle(w, _) => Self::Angle(*w, false),
            Type::Complex(w, _) => Self::Complex(*w, false),
            Type::Float(w, _) => Self::Float(*w, false),
            Type::Int(w, _) => Self::Int(*w, false),
            Type::UInt(w, _) => Self::UInt(*w, false),
            Type::BitArray(dims, _) => Self::BitArray(dims.clone(), false),
            _ => self.clone(),
        }
    }

    pub(crate) fn is_quantum(&self) -> bool {
        matches!(
            self,
            Type::HardwareQubit | Type::Qubit | Type::QubitArray(_)
        )
    }
}

fn indexed_type_builder(
    ty: impl Fn() -> Type,
    ty_array: impl Fn(ArrayDimensions) -> Type,
    dims: &ArrayDimensions,
) -> Type {
    match dims.clone() {
        ArrayDimensions::One(_) => ty(),
        ArrayDimensions::Two(d1, _) => ty_array(ArrayDimensions::One(d1)),
        ArrayDimensions::Three(d1, d2, _) => ty_array(ArrayDimensions::Two(d1, d2)),
        ArrayDimensions::Four(d1, d2, d3, _) => ty_array(ArrayDimensions::Three(d1, d2, d3)),
        ArrayDimensions::Five(d1, d2, d3, d4, _) => ty_array(ArrayDimensions::Four(d1, d2, d3, d4)),
        ArrayDimensions::Six(d1, d2, d3, d4, d5, _) => {
            ty_array(ArrayDimensions::Five(d1, d2, d3, d4, d5))
        }
        ArrayDimensions::Seven(d1, d2, d3, d4, d5, d6, _) => {
            ty_array(ArrayDimensions::Six(d1, d2, d3, d4, d5, d6))
        }
        ArrayDimensions::Err => Type::Err,
    }
}

#[derive(Debug, Clone, Default, Eq, Hash, PartialEq)]
pub enum ArrayDimensions {
    One(u32),
    Two(u32, u32),
    Three(u32, u32, u32),
    Four(u32, u32, u32, u32),
    Five(u32, u32, u32, u32, u32),
    Six(u32, u32, u32, u32, u32, u32),
    Seven(u32, u32, u32, u32, u32, u32, u32),
    #[default]
    Err,
}

impl From<u32> for ArrayDimensions {
    fn from(value: u32) -> Self {
        Self::One(value)
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
            ArrayDimensions::Err => write!(f, "Invalid array dimensions"),
        }
    }
}

impl ArrayDimensions {
    #[must_use]
    pub fn num_dims(&self) -> usize {
        match self {
            ArrayDimensions::One(_) => 1,
            ArrayDimensions::Two(_, _) => 2,
            ArrayDimensions::Three(_, _, _) => 3,
            ArrayDimensions::Four(_, _, _, _) => 4,
            ArrayDimensions::Five(_, _, _, _, _) => 5,
            ArrayDimensions::Six(_, _, _, _, _, _) => 6,
            ArrayDimensions::Seven(_, _, _, _, _, _, _) => 7,
            ArrayDimensions::Err => 0,
        }
    }
}

/// When two types are combined, the result is a type that can represent both.
/// For constness, the result is const iff both types are const.
#[must_use]
pub fn relax_constness(lhs_ty: &Type, rhs_ty: &Type) -> bool {
    lhs_ty.is_const() && rhs_ty.is_const()
}

/// Having no width means that the type is not a fixed-width type
/// and can hold any explicit width. If both types have a width,
/// the result is the maximum of the two. Otherwise, the result
/// is a type without a width.
#[must_use]
pub fn promote_width(lhs_ty: &Type, rhs_ty: &Type) -> Option<u32> {
    match (lhs_ty.width(), rhs_ty.width()) {
        (Some(w1), Some(w2)) => Some(max(w1, w2)),
        (Some(_) | None, None) | (None, Some(_)) => None,
    }
}

fn get_effective_width(lhs_ty: &Type, rhs_ty: &Type) -> Option<u32> {
    match (lhs_ty.width(), rhs_ty.width()) {
        (Some(w1), Some(w2)) => Some(max(w1, w2)),
        (Some(w), None) | (None, Some(w)) => Some(w),
        (None, None) => None,
    }
}

/// If both can be promoted to a common type, the result is that type.
/// If the types are not compatible, the result is `Type::Void`.
#[must_use]
pub fn promote_types(lhs_ty: &Type, rhs_ty: &Type) -> Type {
    if types_equal_except_const(lhs_ty, rhs_ty) {
        return lhs_ty.clone();
    }
    let ty = promote_types_symmetric(lhs_ty, rhs_ty);
    if ty != Type::Void {
        return ty;
    }
    let ty = promote_types_asymmetric(lhs_ty, rhs_ty);
    if ty == Type::Void {
        return promote_types_asymmetric(rhs_ty, lhs_ty);
    }
    ty
}

pub(crate) fn promote_to_uint_ty(
    lhs_ty: &Type,
    rhs_ty: &Type,
) -> (Option<Type>, Option<Type>, Option<Type>) {
    let is_const = relax_constness(lhs_ty, rhs_ty);
    let lhs_ty = get_uint_ty(lhs_ty);
    let rhs_ty = get_uint_ty(rhs_ty);
    match (lhs_ty, rhs_ty) {
        (Some(lhs_ty), Some(rhs_ty)) => {
            let width = get_effective_width(&lhs_ty, &rhs_ty);
            (
                Some(Type::UInt(width, is_const)),
                Some(lhs_ty),
                Some(rhs_ty),
            )
        }
        (Some(lhs_ty), None) => (None, Some(lhs_ty), None),
        (None, Some(rhs_ty)) => (None, None, Some(rhs_ty)),
        (None, None) => (None, None, None),
    }
}

fn get_uint_ty(ty: &Type) -> Option<Type> {
    if matches!(ty, Type::Int(..) | Type::UInt(..) | Type::Angle(..)) {
        Some(Type::UInt(ty.width(), ty.is_const()))
    } else if matches!(ty, Type::Bool(..) | Type::Bit(..)) {
        Some(Type::UInt(Some(1), ty.is_const()))
    } else if let Type::BitArray(dims, _) = ty {
        match dims {
            ArrayDimensions::One(d) => Some(Type::UInt(Some(*d), ty.is_const())),
            _ => None,
        }
    } else {
        None
    }
}

/// Promotes two types if they share a common base type with
/// their constness relaxed, and their width promoted.
/// If the types are not compatible, the result is `Type::Void`.
fn promote_types_symmetric(lhs_ty: &Type, rhs_ty: &Type) -> Type {
    let is_const = relax_constness(lhs_ty, rhs_ty);
    match (lhs_ty, rhs_ty) {
        (Type::Bit(..), Type::Bit(..)) => Type::Bit(is_const),
        (Type::Bool(..), Type::Bool(..)) => Type::Bool(is_const),
        (Type::Int(..), Type::Int(..)) => Type::Int(promote_width(lhs_ty, rhs_ty), is_const),
        (Type::UInt(..), Type::UInt(..)) => Type::UInt(promote_width(lhs_ty, rhs_ty), is_const),
        (Type::Angle(..), Type::Angle(..)) => Type::Angle(promote_width(lhs_ty, rhs_ty), is_const),
        (Type::Float(..), Type::Float(..)) => Type::Float(promote_width(lhs_ty, rhs_ty), is_const),
        (Type::Complex(..), Type::Complex(..)) => {
            Type::Complex(promote_width(lhs_ty, rhs_ty), is_const)
        }
        _ => Type::Void,
    }
}

/// Promotion follows casting rules. We only match one way, as the
/// both combinations are covered by calling this function twice
/// with the arguments swapped.
///
/// If the types are not compatible, the result is `Type::Void`.
///
/// The left-hand side is the type to promote from, and the right-hand
/// side is the type to promote to. So any promotion goes from lesser
/// type to greater type.
///
/// This is more complicated as we have C99 promotion for simple types,
/// but complex types like `Complex`, and `Angle` don't follow those rules.
fn promote_types_asymmetric(lhs_ty: &Type, rhs_ty: &Type) -> Type {
    let is_const = relax_constness(lhs_ty, rhs_ty);
    #[allow(clippy::match_same_arms)]
    match (lhs_ty, rhs_ty) {
        (Type::Bit(..), Type::Bool(..)) => Type::Bool(is_const),
        (Type::Bit(..), Type::Int(w, _)) => Type::Int(*w, is_const),
        (Type::Bit(..), Type::UInt(w, _)) => Type::UInt(*w, is_const),

        (Type::Bit(..), Type::Angle(w, _)) => Type::Angle(*w, is_const),

        (Type::Bool(..), Type::Int(w, _)) => Type::Int(*w, is_const),
        (Type::Bool(..), Type::UInt(w, _)) => Type::UInt(*w, is_const),
        (Type::Bool(..), Type::Float(w, _)) => Type::Float(*w, is_const),
        (Type::Bool(..), Type::Complex(w, _)) => Type::Complex(*w, is_const),

        (Type::UInt(..), Type::Int(..)) => Type::Int(promote_width(lhs_ty, rhs_ty), is_const),
        (Type::UInt(..), Type::Float(..)) => Type::Float(promote_width(lhs_ty, rhs_ty), is_const),
        (Type::UInt(..), Type::Complex(..)) => {
            Type::Complex(promote_width(lhs_ty, rhs_ty), is_const)
        }

        (Type::Int(..), Type::Float(..)) => Type::Float(promote_width(lhs_ty, rhs_ty), is_const),
        (Type::Int(..), Type::Complex(..)) => {
            Type::Complex(promote_width(lhs_ty, rhs_ty), is_const)
        }
        (Type::Angle(..), Type::Float(..)) => Type::Float(promote_width(lhs_ty, rhs_ty), is_const),
        (Type::Float(..), Type::Complex(..)) => {
            Type::Complex(promote_width(lhs_ty, rhs_ty), is_const)
        }
        _ => Type::Void,
    }
}

/// Compares two types for equality, ignoring constness.
pub(crate) fn types_equal_except_const(lhs: &Type, rhs: &Type) -> bool {
    match (lhs, rhs) {
        (Type::Bit(_), Type::Bit(_))
        | (Type::Qubit, Type::Qubit)
        | (Type::HardwareQubit, Type::HardwareQubit)
        | (Type::Bool(_), Type::Bool(_))
        | (Type::Duration(_), Type::Duration(_))
        | (Type::Stretch(_), Type::Stretch(_))
        | (Type::Range, Type::Range)
        | (Type::Set, Type::Set)
        | (Type::Void, Type::Void)
        | (Type::Err, Type::Err) => true,
        (Type::Int(lhs_width, _), Type::Int(rhs_width, _))
        | (Type::UInt(lhs_width, _), Type::UInt(rhs_width, _))
        | (Type::Float(lhs_width, _), Type::Float(rhs_width, _))
        | (Type::Angle(lhs_width, _), Type::Angle(rhs_width, _))
        | (Type::Complex(lhs_width, _), Type::Complex(rhs_width, _)) => lhs_width == rhs_width,
        (Type::BitArray(lhs_dims, _), Type::BitArray(rhs_dims, _))
        | (Type::BoolArray(lhs_dims), Type::BoolArray(rhs_dims))
        | (Type::QubitArray(lhs_dims), Type::QubitArray(rhs_dims)) => lhs_dims == rhs_dims,
        (Type::IntArray(lhs_width, lhs_dims), Type::IntArray(rhs_width, rhs_dims))
        | (Type::UIntArray(lhs_width, lhs_dims), Type::UIntArray(rhs_width, rhs_dims))
        | (Type::FloatArray(lhs_width, lhs_dims), Type::FloatArray(rhs_width, rhs_dims))
        | (Type::AngleArray(lhs_width, lhs_dims), Type::AngleArray(rhs_width, rhs_dims))
        | (Type::ComplexArray(lhs_width, lhs_dims), Type::ComplexArray(rhs_width, rhs_dims)) => {
            lhs_width == rhs_width && lhs_dims == rhs_dims
        }
        (Type::Gate(lhs_cargs, lhs_qargs), Type::Gate(rhs_cargs, rhs_qargs)) => {
            lhs_cargs == rhs_cargs && lhs_qargs == rhs_qargs
        }
        _ => false,
    }
}

/// Compares two types for equality, ignoring constness and width.
/// arrays are equal if their dimensions are equal.
pub(crate) fn base_types_equal(lhs: &Type, rhs: &Type) -> bool {
    match (lhs, rhs) {
        (Type::Bit(_), Type::Bit(_))
        | (Type::Qubit, Type::Qubit)
        | (Type::HardwareQubit, Type::HardwareQubit)
        | (Type::Bool(_), Type::Bool(_))
        | (Type::Duration(_), Type::Duration(_))
        | (Type::Stretch(_), Type::Stretch(_))
        | (Type::Range, Type::Range)
        | (Type::Set, Type::Set)
        | (Type::Void, Type::Void)
        | (Type::Err, Type::Err)
        | (Type::Int(_, _), Type::Int(_, _))
        | (Type::UInt(_, _), Type::UInt(_, _))
        | (Type::Float(_, _), Type::Float(_, _))
        | (Type::Angle(_, _), Type::Angle(_, _))
        | (Type::Complex(_, _), Type::Complex(_, _))
        | (Type::Gate(_, _), Type::Gate(_, _)) => true,
        (Type::BitArray(lhs_dims, _), Type::BitArray(rhs_dims, _))
        | (Type::BoolArray(lhs_dims), Type::BoolArray(rhs_dims))
        | (Type::QubitArray(lhs_dims), Type::QubitArray(rhs_dims)) => lhs_dims == rhs_dims,
        (Type::IntArray(_, lhs_dims), Type::IntArray(_, rhs_dims))
        | (Type::UIntArray(_, lhs_dims), Type::UIntArray(_, rhs_dims))
        | (Type::FloatArray(_, lhs_dims), Type::FloatArray(_, rhs_dims))
        | (Type::AngleArray(_, lhs_dims), Type::AngleArray(_, rhs_dims))
        | (Type::ComplexArray(_, lhs_dims), Type::ComplexArray(_, rhs_dims)) => {
            lhs_dims == rhs_dims
        }
        _ => false,
    }
}

#[must_use]
pub fn can_cast_literal(lhs_ty: &Type, ty_lit: &Type) -> bool {
    // todo: not sure if this top case is still needed after parser changes
    if matches!(lhs_ty, Type::Int(..)) && matches!(ty_lit, Type::UInt(..)) {
        return true;
    }
    // todo: not sure if this case is still needed after parser changes
    if matches!(lhs_ty, Type::UInt(..)) {
        return matches!(ty_lit, Type::Complex(..));
    }

    base_types_equal(lhs_ty, ty_lit)
        || matches!(
            (lhs_ty, ty_lit),
            (Type::Angle(_, _), Type::Float(_, _) | Type::Bit(..))
        )
        || matches!((lhs_ty, ty_lit), (Type::Bit(..), Type::Angle(..)))
        || matches!(
            (lhs_ty, ty_lit),
            (
                Type::Float(_, _) | Type::Complex(_, _),
                Type::Int(_, _) | Type::UInt(_, _)
            ) | (Type::Complex(_, _), Type::Float(_, _))
        )
        || {
            matches!(lhs_ty, Type::Bit(..) | Type::Bool(..))
                && matches!(ty_lit, Type::Bit(..) | Type::Bool(..))
        }
        || {
            match lhs_ty {
                Type::BitArray(dims, _) => {
                    matches!(dims, ArrayDimensions::One(_))
                        && matches!(ty_lit, Type::Int(_, _) | Type::UInt(_, _))
                }
                _ => false,
            }
        }
}

/// some literals can be cast to a specific type if the value is known
/// This is useful to avoid generating a cast expression in the AST
pub(crate) fn can_cast_literal_with_value_knowledge(lhs_ty: &Type, kind: &LiteralKind) -> bool {
    if matches!(lhs_ty, &Type::Bit(_)) {
        if let LiteralKind::Int(value) = kind {
            return *value == 0 || *value == 1;
        }
    }
    if matches!(lhs_ty, &Type::UInt(..)) {
        if let LiteralKind::Int(value) = kind {
            return *value >= 0;
        }
    }
    false
}

// https://openqasm.com/language/classical.html
pub(crate) fn unary_op_can_be_applied_to_type(op: syntax::UnaryOp, ty: &Type) -> bool {
    match op {
        syntax::UnaryOp::NotB => match ty {
            Type::Bit(_) | Type::UInt(_, _) | Type::Angle(_, _) => true,
            Type::BitArray(dims, _) | Type::UIntArray(_, dims) | Type::AngleArray(_, dims) => {
                // the spe says "registers of the same size" which is a bit ambiguous
                // but we can assume that it means that the array is a single dim
                matches!(dims, ArrayDimensions::One(_))
            }
            _ => false,
        },
        syntax::UnaryOp::NotL => matches!(ty, Type::Bool(_)),
        syntax::UnaryOp::Neg => {
            matches!(ty, Type::Int(_, _) | Type::Float(_, _) | Type::Angle(_, _))
        }
    }
}

/// Bit arrays can be compared, but need to be converted to int first
pub(crate) fn binop_requires_int_conversion_for_type(
    op: syntax::BinOp,
    lhs: &Type,
    rhs: &Type,
) -> bool {
    match op {
        syntax::BinOp::Eq
        | syntax::BinOp::Gt
        | syntax::BinOp::Gte
        | syntax::BinOp::Lt
        | syntax::BinOp::Lte
        | syntax::BinOp::Neq => match (lhs, rhs) {
            (Type::BitArray(lhs_dims, _), Type::BitArray(rhs_dims, _)) => {
                match (lhs_dims, rhs_dims) {
                    (ArrayDimensions::One(lhs_size), ArrayDimensions::One(rhs_size)) => {
                        lhs_size == rhs_size
                    }
                    _ => false,
                }
            }
            _ => false,
        },
        _ => false,
    }
}

/// Symmetric arithmetic conversions are applied to:
/// binary arithmetic *, /, %, +, -
/// relational operators <, >, <=, >=, ==, !=
/// binary bitwise arithmetic &, ^, |,
pub(crate) fn requires_symmetric_conversion(op: syntax::BinOp) -> bool {
    match op {
        syntax::BinOp::Add
        | syntax::BinOp::AndB
        | syntax::BinOp::AndL
        | syntax::BinOp::Div
        | syntax::BinOp::Eq
        | syntax::BinOp::Exp
        | syntax::BinOp::Gt
        | syntax::BinOp::Gte
        | syntax::BinOp::Lt
        | syntax::BinOp::Lte
        | syntax::BinOp::Mod
        | syntax::BinOp::Mul
        | syntax::BinOp::Neq
        | syntax::BinOp::OrB
        | syntax::BinOp::OrL
        | syntax::BinOp::Sub
        | syntax::BinOp::XorB => true,
        syntax::BinOp::Shl | syntax::BinOp::Shr => false,
    }
}

pub(crate) fn try_promote_with_casting(left_type: &Type, right_type: &Type) -> Type {
    let promoted_type = promote_types(left_type, right_type);

    if promoted_type != Type::Void {
        return promoted_type;
    }
    if let Some(value) = try_promote_bitarray_to_int(left_type, right_type) {
        return value;
    }
    // simple promotion failed, try a lossless cast
    // each side to double
    let promoted_rhs = promote_types(&Type::Float(None, false), right_type);
    let promoted_lhs = promote_types(left_type, &Type::Float(None, false));

    match (promoted_lhs, promoted_rhs) {
        (Type::Void, Type::Void) => Type::Float(None, false),
        (Type::Void, promoted_rhs) => promoted_rhs,
        (promoted_lhs, Type::Void) => promoted_lhs,
        (promoted_lhs, promoted_rhs) => {
            // return the greater of the two promoted types
            if matches!(promoted_lhs, Type::Complex(..)) {
                promoted_lhs
            } else if matches!(promoted_rhs, Type::Complex(..)) {
                promoted_rhs
            } else if matches!(promoted_lhs, Type::Float(..)) {
                promoted_lhs
            } else if matches!(promoted_rhs, Type::Float(..)) {
                promoted_rhs
            } else {
                Type::Float(None, false)
            }
        }
    }
}

fn try_promote_bitarray_to_int(left_type: &Type, right_type: &Type) -> Option<Type> {
    if matches!(
        (left_type, right_type),
        (Type::Int(..) | Type::UInt(..), Type::BitArray(..))
    ) {
        let Type::BitArray(ArrayDimensions::One(size), _) = right_type else {
            return None;
        };

        if left_type.width() != Some(*size) {
            return None;
        }

        return Some(left_type.clone());
    }

    if matches!(
        (left_type, right_type),
        (Type::BitArray(..), Type::Int(..) | Type::UInt(..))
    ) {
        let Type::BitArray(ArrayDimensions::One(size), _) = left_type else {
            return None;
        };

        if right_type.width() != Some(*size) {
            return None;
        }

        return Some(right_type.clone());
    }
    None
}

// integer promotions are applied only to both operands of
// the shift operators << and >>
pub(crate) fn binop_requires_symmetric_int_conversion(op: syntax::BinOp) -> bool {
    matches!(op, syntax::BinOp::Shl | syntax::BinOp::Shr)
}

pub(crate) fn is_complex_binop_supported(op: syntax::BinOp) -> bool {
    matches!(
        op,
        syntax::BinOp::Add
            | syntax::BinOp::Sub
            | syntax::BinOp::Mul
            | syntax::BinOp::Div
            | syntax::BinOp::Exp
    )
}
