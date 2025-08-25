//! Numeric promotion / interoperability logic (initial extraction).
//!
//! This module centralizes the decision logic for binary operator result typing
//! and interoperability constraints so that operator inference arms do not
//! need to hard-code knowledge of particular type pairs (e.g., Double/Complex).
//!
//! Initially implemented for `+` and now generalized to cover `+ - * /` via a
//! shared lattice + rule table. Formerly behind a feature flag; now always enabled.
//!
//! DESIGN OVERVIEW
//! ----------------
//! Goal: Provide Double–Complex mixed arithmetic without globally unifying the two types
//! while preserving existing inference behavior for other numeric mixes and keeping
//! diagnostic ordering stable. We centralize operator result typing for `+ - * /` so that
//! adding future promotion paths (e.g., widening Int→BigInt or introducing user-defined
//! numeric kinds) requires only rule additions here rather than ad‑hoc logic in `rules.rs`.
//!
//! LATTICE
//! -------
//! The current numeric lattice is intentionally minimal:
//!
//!   Int   `BigInt`   Double  <  Complex
//!
//! Only `Double < Complex` is a widening edge. Int and `BigInt` do NOT implicitly widen and
//! there is currently no Int < Double edge to avoid silently changing precision. This keeps
//! existing numeric inference deterministic and conservative.
//!
//! RULE EVALUATION (PRECEDENCE)
//! ----------------------------
//! For each binary arithmetic op we classify operands into a `Shape`:
//!   * Infer (unresolved inference var)
//!   * Numeric(kind) (one of the concrete numeric kinds including Complex)
//!   * `OtherResolved` (resolved non-numeric) / Other
//!
//! We iterate an ordered rule list; the first matching rule yields a `PromotionDecision`:
//!  1. `complex_involvement`: If either side is Complex → result = Complex (eager)
//!  2. `same_concrete_numeric`: Same primitive numeric kind → that kind (eager)
//!  3. `infer_plus_non_double_primitive`: Infer with Int/BigInt → concrete primitive (eager)
//!  4. `both_infer`: Defer (allocate fresh placeholder) so later concrete operands (e.g. Double +
//!     Complex) can upgrade the result; reusing the lhs would strand the result as an unconstrained
//!     inference when only interoperability (not equality) constraints are emitted.
//!  5. (fallback) Attempt lattice LUB (e.g., Double + Complex) else defer
//!
//! DEFERRED RESOLUTION
//! -------------------
//! Some mixes (notably Double + Infer where the other side might later become Complex) must
//! postpone deciding the result until after constraint solving. In those cases we allocate a
//! fresh inference variable and record (result, lhs, rhs, op) for a post‑solve adjustment pass
//! in `Inferrer::solve`. Post‑solve we substitute any solved operands and:
//!   * If either operand is Complex → result := Complex
//!   * Else if both the same primitive → adopt that primitive
//!   * Else (error paths / unsatisfied constraints) adopt lhs to suppress cascading ambiguity
//!
//! INTEROPERABILITY CONSTRAINT
//! ---------------------------
//! We emit an `Interoperable` constraint (instead of plain Eq) for every arithmetic operation.
//! The solver treats `(Double, Complex)` pairs as satisfied without equating them, enabling
//! mixed usage while preventing accidental structural equivalence elsewhere.
//!
//! EXTENSIBILITY
//! -------------
//! Adding a widening (e.g., `Int` < `BigInt`) would involve updating `NumericKind`, `lub`, and
//! optionally inserting a targeted rule before `same_concrete_numeric` if needed for eager
//! behavior. Additional operators can reuse the same engine by invoking `apply_arith`.
//!
//! DIAGNOSTICS & STABILITY
//! -----------------------
//! - Rule ordering avoids introducing new ambiguities for previously valid code.
//! - Deferred results fall back to lhs type only in already error scenarios to prevent extra
//!   `AmbiguousTy` diagnostics.
//! - The Add/Sub/Mul/Div class constraints are still enforced separately, preserving existing
//!   `MissingClass`* error ordering.
//!
//! Testing: See tests exercising Double+Complex across all four ops plus invalid assignment
//! cases ensuring we don't collapse the types.

#![allow(dead_code)]

#[cfg(test)]
mod tests; // in `promotion/tests.rs` – lattice & LUB focused unit tests

use super::infer::{ArithOp, Inferrer};
use crate::typeck::numeric;
use qsc_data_structures::span::Span;
use qsc_hir::ty::{Prim, Ty};
use std::fmt;

/// Outcome of applying promotion logic for a binary operator.
pub struct PromotionOutcome {
    /// Whether an interoperability constraint should be emitted (caller is
    /// responsible for actually emitting it so diagnostic ordering stays local).
    pub needs_interop: bool,
    /// The result typing decision.
    pub decision: PromotionDecision,
}

/// How the result type should be materialized by the caller.
pub enum PromotionDecision {
    /// Use the given eager concrete type.
    Eager(Ty),
    /// Reuse the lhs inference variable directly.
    ReuseLhsInfer,
    /// Allocate a fresh placeholder inference var and record for deferred resolution.
    Deferred,
}

impl PromotionOutcome {
    fn eager(ty: Ty) -> Self {
        Self {
            needs_interop: true,
            decision: PromotionDecision::Eager(ty),
        }
    }
    fn reuse_lhs() -> Self {
        Self {
            needs_interop: true,
            decision: PromotionDecision::ReuseLhsInfer,
        }
    }
    fn deferred() -> Self {
        Self {
            needs_interop: true,
            decision: PromotionDecision::Deferred,
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Numeric lattice & rule engine (initial version)
// -------------------------------------------------------------------------------------------------

/// Numeric kinds relevant to promotion. (String and Array are excluded for now; they
/// participate in `+` but do not interoperate with numeric promotion paths.)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum NumericKind {
    Int,
    BigInt,
    Double,
    Complex,
}

impl fmt::Display for NumericKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NumericKind::Int => "Int",
                NumericKind::BigInt => "BigInt",
                NumericKind::Double => "Double",
                NumericKind::Complex => "Complex",
            }
        )
    }
}

fn kind_of(ty: &Ty) -> Option<NumericKind> {
    match ty {
        Ty::Prim(Prim::Int) => Some(NumericKind::Int),
        Ty::Prim(Prim::BigInt) => Some(NumericKind::BigInt),
        Ty::Prim(Prim::Double) => Some(NumericKind::Double),
        _ if numeric::is_complex(ty) => Some(NumericKind::Complex),
        _ => None,
    }
}

fn concrete_of(kind: NumericKind) -> Ty {
    match kind {
        NumericKind::Int => Ty::Prim(Prim::Int),
        NumericKind::BigInt => Ty::Prim(Prim::BigInt),
        NumericKind::Double => Ty::Prim(Prim::Double),
        NumericKind::Complex => numeric::complex_ty(),
    }
}

/// Least upper bound in the current lattice:
/// `Int`, `BigInt`, `Double` are isolated; only `Double < Complex`. `Int`/`BigInt` do NOT widen.
fn lub(a: NumericKind, b: NumericKind) -> Option<NumericKind> {
    if a == b {
        return Some(a);
    }
    match (a, b) {
        (NumericKind::Double, NumericKind::Complex)
        | (NumericKind::Complex, NumericKind::Double) => Some(NumericKind::Complex),
        _ => None,
    }
}

/// Pattern classification for operands to simplify rule matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    Infer,
    Numeric(NumericKind),
    OtherResolved, // resolved non-numeric (e.g., String, Array, etc.)
    Other,         // any other (shouldn't occur distinctly once resolved vs infer separated)
}

fn shape_of(ty: &Ty) -> Shape {
    if matches!(ty, Ty::Infer(_)) {
        return Shape::Infer;
    }
    if let Some(k) = kind_of(ty) {
        return Shape::Numeric(k);
    }
    // Distinguish resolved non-numeric from other (for future extension).
    Shape::OtherResolved
}

/// A promotion rule describes a match + action pair.
struct PromotionRule {
    name: &'static str,
    matcher: fn(Shape, Shape) -> bool,
    action: fn(&Ty, &Ty, Shape, Shape) -> PromotionDecision,
}

// Rule definitions (ordered). Simpler to keep as function returning slice for now.
fn arith_rules() -> &'static [PromotionRule] {
    // NOTE: Order matters – earlier rules have priority.
    static RULES: &[PromotionRule] = &[
        // 1. Any Complex involvement (eager Complex).
        PromotionRule {
            name: "complex_involvement",
            matcher: |l, r| {
                matches!(l, Shape::Numeric(NumericKind::Complex))
                    || matches!(r, Shape::Numeric(NumericKind::Complex))
            },
            action: |lhs, rhs, _l, _r| {
                // Preserve the existing Complex operand's concrete UDT representation (local vs core)
                let chosen = if numeric::is_complex(lhs) {
                    lhs.clone()
                } else {
                    rhs.clone()
                };
                PromotionDecision::Eager(chosen)
            },
        },
        // 2. Same concrete numeric (Int, BigInt, Double) -> that type.
        PromotionRule {
            name: "same_concrete_numeric",
            matcher: |l, r| matches!(l, Shape::Numeric(_)) && l == r,
            action: |_lhs, _rhs, l, _| match l {
                Shape::Numeric(k) => PromotionDecision::Eager(concrete_of(k)),
                _ => unreachable!(),
            },
        },
        // 3. One inference + one concrete non-Double primitive numeric (Int or BigInt) -> that primitive.
        PromotionRule {
            name: "infer_plus_non_double_primitive",
            matcher: |l, r| match (l, r) {
                (Shape::Infer, Shape::Numeric(k)) | (Shape::Numeric(k), Shape::Infer) => {
                    matches!(k, NumericKind::Int | NumericKind::BigInt)
                }
                _ => false,
            },
            action: |_lhs, _rhs, l, r| {
                let concrete = match (l, r) {
                    (Shape::Infer, Shape::Numeric(k)) | (Shape::Numeric(k), Shape::Infer) => {
                        concrete_of(k)
                    }
                    _ => unreachable!(),
                };
                PromotionDecision::Eager(concrete)
            },
        },
        // 4. Both inference -> defer. We need a distinct placeholder recorded for post‑solve so
        // that if one side later becomes Complex (and the other Double) we can upgrade the
        // result to Complex. Reusing the lhs would (a) prevent us from recording the arithmetic
        // triple and (b) risk leaving the result inference var unsolved because interoperability
        // does not force equality. This manifested in `lambda_implicit_return_with_call_complex`
        // where `a + b` stayed as an unresolved inference variable instead of upgrading.
        PromotionRule {
            name: "both_infer",
            matcher: |l, r| matches!(l, Shape::Infer) && matches!(r, Shape::Infer),
            action: |_lhs, _rhs, _l, _r| PromotionDecision::Deferred,
        },
        // 5. Mixed Double + inference (or inference + Double) – force deferral explicitly so we
        // don't accidentally hit LUB (which would eagerly choose Double) before the inference
        // variable has a chance to become Complex via other constraints (see test `bar`).
        PromotionRule {
            name: "infer_plus_double_defer",
            matcher: |l, r| {
                matches!(
                    (l, r),
                    (Shape::Infer, Shape::Numeric(NumericKind::Double))
                        | (Shape::Numeric(NumericKind::Double), Shape::Infer)
                )
            },
            // IMPORTANT: We must allocate a fresh deferred placeholder here rather than
            // reusing the lhs. Returning `ReuseLhsInfer` when the lhs is the concrete Double
            // (case: Double + infer) prematurely fixes the result to Double and prevents a
            // later upgrade to Complex if the inference variable acquires a Complex constraint
            // through other usages (see tests `bar` / `bar2`). Using `Deferred` ensures we
            // postpone committing to Double until post‑solve when we can examine the fully
            // substituted operands and (if needed) upgrade to Complex or adopt a common kind.
            action: |_lhs, _rhs, _l, _r| PromotionDecision::Deferred,
        },
        // (Remaining mixes considered by lattice / fallback.)
    ];
    RULES
}

/// Apply promotion logic for an arithmetic operator using the rule table + lattice.
#[allow(clippy::match_same_arms)] // symmetric inference cases intentionally share bodies
pub fn apply_arith(
    _inferrer: &mut Inferrer,
    _op: ArithOp,
    _span: Span,
    _lhs_span: Span,
    _rhs_span: Span,
    lhs: &Ty,
    rhs: &Ty,
) -> PromotionOutcome {
    let lshape = shape_of(lhs);
    let rshape = shape_of(rhs);

    for rule in arith_rules() {
        if (rule.matcher)(lshape, rshape) {
            let decision = (rule.action)(lhs, rhs, lshape, rshape);
            return PromotionOutcome {
                needs_interop: true,
                decision,
            };
        }
    }

    // Attempt a lattice-based LUB if both numeric kinds are concrete (and not yet matched).
    if let (Shape::Numeric(k1), Shape::Numeric(k2)) = (lshape, rshape) {
        if let Some(lub_kind) = lub(k1, k2) {
            return PromotionOutcome::eager(concrete_of(lub_kind));
        }
    }

    // Fallback: defer.
    PromotionOutcome::deferred()
}

// (Helper predicates moved to numeric.rs)

/// Hook used by the caller when a deferred placeholder is created, so we can record it
/// for post-solve adjustment.
pub fn record_deferred_arith(
    inferrer: &mut Inferrer,
    op: ArithOp,
    placeholder: &Ty,
    lhs: &Ty,
    rhs: &Ty,
) {
    inferrer.record_arith_result(op, placeholder, lhs, rhs);
}

// -------------------------------------------------------------------------------------------------
// Post‑solve helpers (centralized here to keep Double/Complex knowledge localized)
// -------------------------------------------------------------------------------------------------

/// Returns true if the two (already substituted) types are considered interoperable without
/// requiring equality (currently only the Double/Complex mix). This is used by the constraint
/// solver when handling the `Interoperable` constraint variant.
pub fn interoperable_without_eq(a: &Ty, b: &Ty) -> bool {
    match (kind_of(a), kind_of(b)) {
        (Some(k1), Some(k2)) if k1 != k2 => lub(k1, k2).is_some(),
        _ => false,
    }
}

/// Decide the final concrete result type for a deferred arithmetic placeholder after all
/// inference constraints have been solved and operands substituted. Returns `Some(result)` if
/// an eager upgrade can now be made (Complex involvement, same primitive, or lattice LUB) or
/// `None` to indicate the caller should fall back to a conservative default (typically the
/// left operand to avoid introducing ambiguity diagnostics).
pub fn finalize_deferred_arith(lhs: &Ty, rhs: &Ty) -> Option<Ty> {
    let lk = kind_of(lhs);
    let rk = kind_of(rhs);
    match (lk, rk) {
        (Some(k1), Some(k2)) => {
            if k1 == k2 {
                // Same numeric kind – reuse lhs (keeps UDT identity if complex-like).
                return Some(lhs.clone());
            }
            if let Some(lub_kind) = lub(k1, k2) {
                // If the LUB is one of the operand kinds, reuse that operand to preserve any
                // non-primitive identity (e.g., Complex UDT instance). Otherwise construct anew.
                if k1 == lub_kind {
                    return Some(lhs.clone());
                }
                if k2 == lub_kind {
                    return Some(rhs.clone());
                }
                return Some(concrete_of(lub_kind));
            }
            None
        }
        _ => None,
    }
}
