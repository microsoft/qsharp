// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::cmp::max;

use oq3_semantics::types::{ArrayDims, IsConst, Type};

/// When two types are combined, the result is a type that can represent both.
/// For constness, the result is const iff both types are const.
fn relax_constness(lhs_ty: &Type, rhs_ty: &Type) -> IsConst {
    IsConst::from(lhs_ty.is_const() && rhs_ty.is_const())
}

/// Having no width means that the type is not a fixed-width type
/// and can hold any explicit width. If both types have a width,
/// the result is the maximum of the two. Otherwise, the result
/// is a type without a width.
fn promote_width(lhs_ty: &Type, rhs_ty: &Type) -> Option<u32> {
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
    if matches!(ty, Type::UInt(..) | Type::Angle(..)) {
        Some(Type::UInt(ty.width(), ty.is_const().into()))
    } else if let Type::BitArray(dims, _) = ty {
        match dims {
            ArrayDims::D1(d) => Some(Type::UInt(
                Some(u32::try_from(*d).ok()?),
                ty.is_const().into(),
            )),
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
        | (Type::ToDo, Type::ToDo)
        | (Type::Undefined, Type::Undefined) => true,
        (Type::Int(lhs_width, _), Type::Int(rhs_width, _))
        | (Type::UInt(lhs_width, _), Type::UInt(rhs_width, _))
        | (Type::Float(lhs_width, _), Type::Float(rhs_width, _))
        | (Type::Angle(lhs_width, _), Type::Angle(rhs_width, _))
        | (Type::Complex(lhs_width, _), Type::Complex(rhs_width, _)) => lhs_width == rhs_width,
        (Type::BitArray(lhs_dims, _), Type::BitArray(rhs_dims, _))
        | (Type::QubitArray(lhs_dims), Type::QubitArray(rhs_dims))
        | (Type::IntArray(lhs_dims), Type::IntArray(rhs_dims))
        | (Type::UIntArray(lhs_dims), Type::UIntArray(rhs_dims))
        | (Type::FloatArray(lhs_dims), Type::FloatArray(rhs_dims))
        | (Type::AngleArray(lhs_dims), Type::AngleArray(rhs_dims))
        | (Type::ComplexArray(lhs_dims), Type::ComplexArray(rhs_dims))
        | (Type::BoolArray(lhs_dims), Type::BoolArray(rhs_dims)) => lhs_dims == rhs_dims,
        (Type::Gate(lhs_cargs, lhs_qargs), Type::Gate(rhs_cargs, rhs_qargs)) => {
            lhs_cargs == rhs_cargs && lhs_qargs == rhs_qargs
        }
        _ => false,
    }
}
