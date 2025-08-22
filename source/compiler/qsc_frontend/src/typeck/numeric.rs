//! Shared numeric type helpers (Complex, primitive predicates, etc.).
//! This module is always compiled so both legacy and feature-gated promotion
//! logic can rely on a single definition.

use qsc_hir::hir::{self, ItemId};
use qsc_hir::ty::{Prim, Ty};
use std::rc::Rc;

/// Return the canonical `Complex` type.
pub fn complex_ty() -> Ty {
    Ty::Udt(
        Rc::from("Complex"),
        hir::Res::Item(ItemId::get_complex_id()),
    )
}

pub fn is_double(ty: &Ty) -> bool {
    matches!(ty, Ty::Prim(Prim::Double))
}

pub fn is_complex(ty: &Ty) -> bool {
    ty.is_complex_udt()
}
