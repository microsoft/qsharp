//! Focused tests for the numeric promotion lattice / LUB logic.
//! These target only the abstraction in `promotion` and avoid higher level parser noise.

use super::*;
use crate::typeck::infer::{ArithOp, Inferrer};
use qsc_data_structures::span::Span;
use qsc_hir::ty::Prim;

fn run(op: ArithOp, lhs: &Ty, rhs: &Ty) -> PromotionDecision {
    let mut inf = Inferrer::new();
    let out = apply_arith(
        &mut inf,
        op,
        Span::default(),
        Span::default(),
        Span::default(),
        lhs,
        rhs,
    );
    out.decision
}

fn eager_ty(decision: PromotionDecision) -> Ty {
    match decision {
        PromotionDecision::Eager(t) => t,
        PromotionDecision::ReuseLhsInfer => panic!("expected eager, got reuse lhs infer"),
        PromotionDecision::Deferred => panic!("expected eager, got deferred"),
    }
}

#[test]
fn lub_double_complex_add() {
    let t = eager_ty(run(
        ArithOp::Add,
        &Ty::Prim(Prim::Double),
        &numeric::complex_ty(),
    ));
    assert!(numeric::is_complex(&t));
}

#[test]
fn lub_double_complex_sub() {
    let t = eager_ty(run(
        ArithOp::Sub,
        &Ty::Prim(Prim::Double),
        &numeric::complex_ty(),
    ));
    assert!(numeric::is_complex(&t));
}

#[test]
fn same_kind_primitive_int() {
    let t = eager_ty(run(
        ArithOp::Mul,
        &Ty::Prim(Prim::Int),
        &Ty::Prim(Prim::Int),
    ));
    assert!(matches!(t, Ty::Prim(Prim::Int)));
}

#[test]
fn same_kind_complex_preserve_identity() {
    let complex = numeric::complex_ty();
    let t = eager_ty(run(ArithOp::Div, &complex.clone(), &complex.clone()));
    // Should be structurally equal (and we expect we reused lhs instance for identity-sensitive UDT cases)
    assert!(t == complex);
}

#[test]
fn infer_then_lub_deferred_finalize() {
    // Case: Double + (infer that later becomes Complex). We simulate by first deferring, then finalizing.
    // Directly exercise finalize_deferred_arith.
    let complex = numeric::complex_ty();
    let res = finalize_deferred_arith(&Ty::Prim(Prim::Double), &complex).expect("should upgrade");
    assert!(numeric::is_complex(&res));
}

#[test]
fn non_lub_mismatch_no_interop() {
    // Int + BigInt has no lattice edge; expect Deferred result decision (promotion engine cannot pick).
    let mut inf = Inferrer::new();
    let out = apply_arith(
        &mut inf,
        ArithOp::Add,
        Span::default(),
        Span::default(),
        Span::default(),
        &Ty::Prim(Prim::Int),
        &Ty::Prim(Prim::BigInt),
    );
    match out.decision {
        PromotionDecision::Deferred => (),
        _ => panic!("expected deferred for Int + BigInt (no LUB)"),
    }
}

#[test]
fn interoperable_without_eq_positive_both_orders() {
    let d = Ty::Prim(Prim::Double);
    let c = numeric::complex_ty();
    assert!(interoperable_without_eq(&d, &c));
    assert!(interoperable_without_eq(&c, &d));
}

#[test]
fn interoperable_without_eq_negative() {
    let i = Ty::Prim(Prim::Int);
    let b = Ty::Prim(Prim::BigInt);
    assert!(!interoperable_without_eq(&i, &b));
    assert!(!interoperable_without_eq(&b, &i));
}

#[test]
fn finalize_deferred_arith_none_for_non_lub_mix() {
    // Int + BigInt: finalize should yield None (no LUB edge)
    assert!(finalize_deferred_arith(&Ty::Prim(Prim::Int), &Ty::Prim(Prim::BigInt)).is_none());
}
