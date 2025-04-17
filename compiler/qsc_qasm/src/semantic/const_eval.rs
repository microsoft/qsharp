// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module allows us to perform const evaluation at lowering time.
//! The purpose of this is to be able to compute the widths of types
//! and sizes of arrays. Therefore, those are the only const evaluation
//! paths that are implemented.

use super::ast::{
    BinOp, BinaryOpExpr, Cast, Expr, ExprKind, FunctionCall, IndexExpr, IndexedIdent, LiteralKind,
    UnaryOp, UnaryOpExpr,
};
use super::symbols::SymbolId;
use crate::semantic::Lowerer;
use crate::stdlib::angle;
use crate::{
    convert::safe_i64_to_f64,
    semantic::types::{ArrayDimensions, Type},
};
use miette::Diagnostic;
use num_bigint::BigInt;
use qsc_data_structures::span::Span;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Eq, Error, PartialEq)]
pub enum ConstEvalError {
    #[error("expression must be const")]
    #[diagnostic(code("Qasm.Compiler.ExprMustBeConst"))]
    ExprMustBeConst(#[label] Span),
    #[error("uint expression must evaluate to a non-negative value, but it evaluated to {0}")]
    #[diagnostic(code("Qasm.Compiler.NegativeUIntValue"))]
    NegativeUIntValue(i64, #[label] Span),
    #[error("{0} doesn't fit in {1}")]
    #[diagnostic(code("Qasm.Compiler.ValueOverflow"))]
    ValueOverflow(String, String, #[label] Span),
}

impl ConstEvalError {}

impl Expr {
    /// Tries to evaluate the expression. It takes the current `Lowerer` as
    /// the evaluation context to resolve symbols and push errors in case
    /// of failure.
    pub(crate) fn const_eval(&self, ctx: &mut Lowerer) -> Option<LiteralKind> {
        let ty = &self.ty;
        if !ty.is_const() {
            ctx.push_const_eval_error(ConstEvalError::ExprMustBeConst(self.span));
            return None;
        }

        match &*self.kind {
            ExprKind::Ident(symbol_id) => symbol_id.const_eval(ctx),
            ExprKind::IndexedIdentifier(indexed_ident) => indexed_ident.const_eval(ctx),
            ExprKind::UnaryOp(unary_op_expr) => unary_op_expr.const_eval(ctx),
            ExprKind::BinaryOp(binary_op_expr) => binary_op_expr.const_eval(ctx),
            ExprKind::Lit(literal_kind) => Some(literal_kind.clone()),
            ExprKind::FunctionCall(function_call) => function_call.const_eval(ctx, ty),
            ExprKind::Cast(cast) => cast.const_eval(ctx),
            ExprKind::IndexExpr(index_expr) => index_expr.const_eval(ctx, ty),
            ExprKind::Paren(expr) => expr.const_eval(ctx),
            // Measurements are non-const, so we don't need to implement them.
            ExprKind::Measure(_) | ExprKind::Err => None,
        }
    }
}

impl SymbolId {
    fn const_eval(self, ctx: &mut Lowerer) -> Option<LiteralKind> {
        let symbol = ctx.symbols[self].clone();
        symbol
            .get_const_expr() // get the value of the symbol (an Expr)
            .const_eval(ctx) // const eval that Expr
    }
}

impl IndexedIdent {
    #[allow(clippy::unused_self)]
    fn const_eval(&self, _ctx: &mut Lowerer) -> Option<LiteralKind> {
        None
    }
}

/// A helper macro for evaluating unary and binary operations of values
/// wrapped in the `semantic::LiteralKind` enum. Unwraps the value in the
/// `LiteralKind` and rewraps it in another `LiteralKind` variant while
/// applying some operation to it.
macro_rules! rewrap_lit {
    // This pattern is used for unary expressions.
    ($lit:expr, $pat:pat, $out:expr) => {
        if let $pat = $lit {
            Some($out)
        } else {
            unreachable!("if we hit this there is a bug in the type system")
        }
    };
}

impl UnaryOpExpr {
    fn const_eval(&self, ctx: &mut Lowerer) -> Option<LiteralKind> {
        use LiteralKind::{Angle, Bit, Bitstring, Bool, Float, Int};
        let operand_ty = &self.expr.ty;
        let lit = self.expr.const_eval(ctx)?;

        match &self.op {
            UnaryOp::Neg => match operand_ty {
                Type::Int(..) => rewrap_lit!(lit, Int(val), Int(-val)),
                Type::Float(..) => rewrap_lit!(lit, Float(val), Float(-val)),
                Type::Angle(..) => rewrap_lit!(lit, Angle(val), Angle(-val)),
                _ => None,
            },
            UnaryOp::NotB => match operand_ty {
                Type::Int(size, _) | Type::UInt(size, _) => rewrap_lit!(lit, Int(val), {
                    let mask = (1 << (*size)?) - 1;
                    Int(!val & mask)
                }),
                Type::Angle(..) => rewrap_lit!(lit, Angle(val), Angle(!val)),
                Type::Bit(..) => rewrap_lit!(lit, Bit(val), Bit(!val)),
                Type::BitArray(..) => {
                    rewrap_lit!(lit, Bitstring(val, size), {
                        let mask = BigInt::from((1 << size) - 1);
                        Bitstring(!val & mask, size)
                    })
                }
                // Angle is treated like a unit in the QASM3 Spec, but we are currently
                // treating it as a float, so we can't apply bitwise negation to it.
                _ => None,
            },
            UnaryOp::NotL => match operand_ty {
                Type::Bool(..) => rewrap_lit!(lit, Bool(val), Bool(!val)),
                _ => None,
            },
        }
    }
}

/// By this point it is guaranteed that the lhs and rhs are of the same type.
/// Any conversions have been made explicit by inserting casts during lowering.
/// Note: the type of the binary expression doesn't need to be the same as the
///       operands, for example, comparison operators can have integer operands
///       but their type is boolean.
/// We can write a simpler implementation under that assumption.
///
/// There are some exceptions:
///  1. The rhs in Shl and Shr must be of type `UInt`.
///  2. Angle can be multiplied and divided by `UInt`.
fn assert_binary_op_ty_invariant(op: BinOp, lhs_ty: &Type, rhs_ty: &Type) {
    // Exceptions:
    if matches!(
        (op, lhs_ty, rhs_ty),
        (BinOp::Shl | BinOp::Shr, _, _)
            | (BinOp::Mul | BinOp::Div, Type::Angle(..), Type::UInt(..))
            | (BinOp::Mul, Type::UInt(..), Type::Angle(..))
    ) {
        return;
    }

    assert_eq!(lhs_ty, rhs_ty);
}

impl BinaryOpExpr {
    #[allow(clippy::too_many_lines)]
    fn const_eval(&self, ctx: &mut Lowerer) -> Option<LiteralKind> {
        use LiteralKind::{Angle, Bit, Bitstring, Bool, Float, Int};

        let lhs = self.lhs.const_eval(ctx);
        let rhs = self.rhs.const_eval(ctx);
        let (lhs, rhs) = (lhs?, rhs?);
        let lhs_ty = &self.lhs.ty;

        assert_binary_op_ty_invariant(self.op, &self.lhs.ty, &self.rhs.ty);

        match &self.op {
            // Bit Shifts
            BinOp::Shl => {
                assert!(
                    matches!(self.rhs.ty, Type::UInt(..)),
                    "shift left rhs should have been casted to uint during lowering"
                );
                let LiteralKind::Int(rhs) = rhs else {
                    unreachable!("if we hit this there is a bug in the type system");
                };
                if rhs < 0 {
                    ctx.push_const_eval_error(ConstEvalError::NegativeUIntValue(
                        rhs,
                        self.rhs.span,
                    ));
                    return None;
                }

                match lhs_ty {
                    Type::UInt(Some(size), _) => rewrap_lit!(lhs, Int(lhs), {
                        let mask = (1 << size) - 1;
                        Int((lhs << rhs) & mask)
                    }),
                    Type::UInt(..) => rewrap_lit!(lhs, Int(lhs), Int(lhs << rhs)),
                    Type::Angle(..) => {
                        rewrap_lit!(lhs, Angle(lhs), Angle(lhs << rhs))
                    }
                    Type::Bit(..) => rewrap_lit!(lhs, Bit(lhs), {
                        // The Spec says "The shift operators shift bits off the end."
                        // Therefore if the rhs is > 0 the value becomes zero.
                        Bit(rhs == 0 && lhs)
                    }),
                    Type::BitArray(..) => {
                        rewrap_lit!(lhs, Bitstring(lhs, size), {
                            let mask = BigInt::from((1 << size) - 1);
                            Bitstring((lhs << rhs) & mask, size)
                        })
                    }
                    _ => None,
                }
            }
            BinOp::Shr => {
                assert!(
                    matches!(self.rhs.ty, Type::UInt(..)),
                    "shift right rhs should have been casted to uint during lowering"
                );
                let LiteralKind::Int(rhs) = rhs else {
                    unreachable!("if we hit this there is a bug in the type system");
                };
                if rhs < 0 {
                    ctx.push_const_eval_error(ConstEvalError::NegativeUIntValue(
                        rhs,
                        self.rhs.span,
                    ));
                    return None;
                }

                match lhs_ty {
                    Type::UInt(..) => rewrap_lit!(lhs, Int(lhs), Int(lhs >> rhs)),
                    Type::Angle(..) => {
                        rewrap_lit!(lhs, Angle(lhs), Angle(lhs >> rhs))
                    }
                    Type::Bit(..) => rewrap_lit!(lhs, Bit(lhs), {
                        // The Spec says "The shift operators shift bits off the end."
                        // Therefore if the rhs is > 0 the value becomes zero.
                        Bit(rhs == 0 && lhs)
                    }),
                    Type::BitArray(..) => {
                        rewrap_lit!(lhs, Bitstring(lhs, size), Bitstring(lhs >> rhs, size))
                    }
                    _ => None,
                }
            }

            // Bitwise
            BinOp::AndB => match lhs_ty {
                Type::UInt(..) => rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Int(lhs & rhs)),
                Type::Angle(..) => {
                    rewrap_lit!((lhs, rhs), (Angle(lhs), Angle(rhs)), Angle(lhs & rhs))
                }
                Type::Bit(..) => rewrap_lit!((lhs, rhs), (Bit(lhs), Bit(rhs)), Bit(lhs & rhs)),
                Type::BitArray(..) => rewrap_lit!(
                    (lhs, rhs),
                    (Bitstring(lhs, lsize), Bitstring(rhs, rsize)),
                    Bitstring(lhs & rhs, lsize.min(rsize))
                ),
                _ => None,
            },
            BinOp::OrB => match lhs_ty {
                Type::UInt(..) => rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Int(lhs | rhs)),
                Type::Angle(..) => {
                    rewrap_lit!((lhs, rhs), (Angle(lhs), Angle(rhs)), Angle(lhs | rhs))
                }
                Type::Bit(..) => rewrap_lit!((lhs, rhs), (Bit(lhs), Bit(rhs)), Bit(lhs | rhs)),
                Type::BitArray(..) => rewrap_lit!(
                    (lhs, rhs),
                    (Bitstring(lhs, lsize), Bitstring(rhs, rsize)),
                    Bitstring(lhs | rhs, lsize.max(rsize))
                ),
                _ => None,
            },
            BinOp::XorB => match lhs_ty {
                Type::UInt(..) => rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Int(lhs ^ rhs)),
                Type::Angle(..) => {
                    rewrap_lit!((lhs, rhs), (Angle(lhs), Angle(rhs)), Angle(lhs ^ rhs))
                }
                Type::Bit(..) => rewrap_lit!((lhs, rhs), (Bit(lhs), Bit(rhs)), Bit(lhs ^ rhs)),
                Type::BitArray(..) => rewrap_lit!(
                    (lhs, rhs),
                    (Bitstring(lhs, lsize), Bitstring(rhs, rsize)),
                    Bitstring(lhs ^ rhs, lsize.max(rsize))
                ),
                _ => None,
            },

            // Logical
            BinOp::AndL => match lhs_ty {
                Type::Bool(..) => rewrap_lit!((lhs, rhs), (Bool(lhs), Bool(rhs)), Bool(lhs && rhs)),
                _ => None,
            },
            BinOp::OrL => match lhs_ty {
                Type::Bool(..) => rewrap_lit!((lhs, rhs), (Bool(lhs), Bool(rhs)), Bool(lhs || rhs)),
                _ => None,
            },

            // Comparison
            BinOp::Eq => match lhs_ty {
                Type::Int(..) | Type::UInt(..) => {
                    rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Bool(lhs == rhs))
                }
                Type::Float(..) => {
                    rewrap_lit!((lhs, rhs), (Float(lhs), Float(rhs)), {
                        #[allow(clippy::float_cmp)]
                        Bool(lhs == rhs)
                    })
                }
                Type::Angle(..) => {
                    rewrap_lit!((lhs, rhs), (Angle(lhs), Angle(rhs)), Bool(lhs == rhs))
                }
                Type::Bit(..) => rewrap_lit!((lhs, rhs), (Bit(lhs), Bit(rhs)), Bool(lhs == rhs)),
                Type::BitArray(..) => rewrap_lit!(
                    (lhs, rhs),
                    (Bitstring(lhs, _), Bitstring(rhs, _)),
                    Bool(lhs == rhs)
                ),
                _ => None,
            },
            BinOp::Neq => match lhs_ty {
                Type::Int(..) | Type::UInt(..) => {
                    rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Bool(lhs != rhs))
                }
                Type::Float(..) => {
                    rewrap_lit!((lhs, rhs), (Float(lhs), Float(rhs)), {
                        #[allow(clippy::float_cmp)]
                        Bool(lhs != rhs)
                    })
                }
                Type::Angle(..) => {
                    rewrap_lit!((lhs, rhs), (Angle(lhs), Angle(rhs)), Bool(lhs != rhs))
                }
                Type::Bit(..) => rewrap_lit!((lhs, rhs), (Bit(lhs), Bit(rhs)), Bool(lhs != rhs)),
                Type::BitArray(..) => rewrap_lit!(
                    (lhs, rhs),
                    (Bitstring(lhs, _), Bitstring(rhs, _)),
                    Bool(lhs != rhs)
                ),
                _ => None,
            },
            BinOp::Gt => match lhs_ty {
                Type::Int(..) | Type::UInt(..) => {
                    rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Bool(lhs > rhs))
                }
                Type::Float(..) => {
                    rewrap_lit!((lhs, rhs), (Float(lhs), Float(rhs)), Bool(lhs > rhs))
                }
                Type::Angle(..) => {
                    rewrap_lit!((lhs, rhs), (Angle(lhs), Angle(rhs)), Bool(lhs > rhs))
                }
                // This was originally `lhs > rhs` but clippy suggested this expression.
                Type::Bit(..) => rewrap_lit!((lhs, rhs), (Bit(lhs), Bit(rhs)), Bool(lhs && !rhs)),
                Type::BitArray(..) => rewrap_lit!(
                    (lhs, rhs),
                    (Bitstring(lhs, _), Bitstring(rhs, _)),
                    Bool(lhs > rhs)
                ),
                _ => None,
            },
            BinOp::Gte => match lhs_ty {
                Type::Int(..) | Type::UInt(..) => {
                    rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Bool(lhs >= rhs))
                }
                Type::Float(..) => {
                    rewrap_lit!((lhs, rhs), (Float(lhs), Float(rhs)), Bool(lhs >= rhs))
                }
                Type::Angle(..) => {
                    rewrap_lit!((lhs, rhs), (Angle(lhs), Angle(rhs)), Bool(lhs >= rhs))
                }
                Type::Bit(..) => rewrap_lit!((lhs, rhs), (Bit(lhs), Bit(rhs)), Bool(lhs >= rhs)),
                Type::BitArray(..) => rewrap_lit!(
                    (lhs, rhs),
                    (Bitstring(lhs, _), Bitstring(rhs, _)),
                    Bool(lhs >= rhs)
                ),
                _ => None,
            },
            BinOp::Lt => match lhs_ty {
                Type::Int(..) | Type::UInt(..) => {
                    rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Bool(lhs < rhs))
                }
                Type::Float(..) => {
                    rewrap_lit!((lhs, rhs), (Float(lhs), Float(rhs)), Bool(lhs < rhs))
                }
                Type::Angle(..) => {
                    rewrap_lit!((lhs, rhs), (Angle(lhs), Angle(rhs)), Bool(lhs < rhs))
                }
                // This was originally `lhs < rhs` but clippy suggested this expression.
                Type::Bit(..) => rewrap_lit!((lhs, rhs), (Bit(lhs), Bit(rhs)), Bool(!lhs & rhs)),
                Type::BitArray(..) => rewrap_lit!(
                    (lhs, rhs),
                    (Bitstring(lhs, _), Bitstring(rhs, _)),
                    Bool(lhs < rhs)
                ),
                _ => None,
            },
            BinOp::Lte => match lhs_ty {
                Type::Int(..) | Type::UInt(..) => {
                    rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Bool(lhs <= rhs))
                }
                Type::Float(..) => {
                    rewrap_lit!((lhs, rhs), (Float(lhs), Float(rhs)), Bool(lhs <= rhs))
                }
                Type::Angle(..) => {
                    rewrap_lit!((lhs, rhs), (Angle(lhs), Angle(rhs)), Bool(lhs <= rhs))
                }
                Type::Bit(..) => rewrap_lit!((lhs, rhs), (Bit(lhs), Bit(rhs)), Bool(lhs <= rhs)),
                Type::BitArray(..) => rewrap_lit!(
                    (lhs, rhs),
                    (Bitstring(lhs, _), Bitstring(rhs, _)),
                    Bool(lhs <= rhs)
                ),
                _ => None,
            },

            // Arithmetic
            BinOp::Add => match lhs_ty {
                Type::Int(..) | Type::UInt(..) => {
                    rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Int(lhs + rhs))
                }
                Type::Float(..) => {
                    rewrap_lit!((lhs, rhs), (Float(lhs), Float(rhs)), Float(lhs + rhs))
                }
                Type::Angle(..) => {
                    rewrap_lit!((lhs, rhs), (Angle(lhs), Angle(rhs)), Angle(lhs + rhs))
                }
                _ => None,
            },
            BinOp::Sub => match lhs_ty {
                Type::Int(..) | Type::UInt(..) => {
                    rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Int(lhs - rhs))
                }
                Type::Float(..) => {
                    rewrap_lit!((lhs, rhs), (Float(lhs), Float(rhs)), Float(lhs - rhs))
                }
                Type::Angle(..) => {
                    rewrap_lit!((lhs, rhs), (Angle(lhs), Angle(rhs)), Angle(lhs - rhs))
                }
                _ => None,
            },
            BinOp::Mul => match lhs_ty {
                Type::Int(..) => rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Int(lhs * rhs)),
                Type::UInt(..) => match &self.rhs.ty {
                    Type::UInt(..) => {
                        rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Int(lhs * rhs))
                    }
                    Type::Angle(..) => rewrap_lit!((lhs, rhs), (Int(lhs), Angle(rhs)), {
                        if lhs < 0 {
                            ctx.push_const_eval_error(ConstEvalError::NegativeUIntValue(
                                lhs,
                                self.lhs.span,
                            ));
                            return None;
                        }
                        #[allow(clippy::cast_sign_loss)]
                        Angle(rhs * u64::try_from(lhs).ok()?)
                    }),

                    _ => None,
                },
                Type::Float(..) => {
                    rewrap_lit!((lhs, rhs), (Float(lhs), Float(rhs)), Float(lhs * rhs))
                }
                Type::Angle(..) => {
                    rewrap_lit!(
                        (lhs, rhs),
                        (Angle(lhs), Int(rhs)),
                        Angle(lhs * u64::try_from(rhs).ok()?)
                    )
                }
                _ => None,
            },
            BinOp::Div => match lhs_ty {
                Type::Int(..) | Type::UInt(..) => {
                    rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Int(lhs / rhs))
                }
                Type::Float(..) => {
                    rewrap_lit!((lhs, rhs), (Float(lhs), Float(rhs)), Float(lhs / rhs))
                }
                Type::Angle(..) => match &self.rhs.ty {
                    Type::UInt(..) => {
                        rewrap_lit!(
                            (lhs, rhs),
                            (Angle(lhs), Int(rhs)),
                            Angle(lhs / u64::try_from(rhs).ok()?)
                        )
                    }
                    Type::Angle(..) => {
                        rewrap_lit!(
                            (lhs, rhs),
                            (Angle(lhs), Angle(rhs)),
                            Int((lhs / rhs).try_into().ok()?)
                        )
                    }
                    _ => None,
                },
                _ => None,
            },
            BinOp::Mod => match lhs_ty {
                Type::Int(..) | Type::UInt(..) => {
                    rewrap_lit!((lhs, rhs), (Int(lhs), Int(rhs)), Int(lhs % rhs))
                }
                _ => None,
            },
            BinOp::Exp => match lhs_ty {
                Type::Int(..) | Type::UInt(..) => {
                    rewrap_lit!(
                        (lhs, rhs),
                        (Int(lhs), Int(rhs)),
                        Int(lhs.wrapping_pow(u32::try_from(rhs).ok()?))
                    )
                }
                Type::Float(..) => {
                    rewrap_lit!((lhs, rhs), (Float(lhs), Float(rhs)), Float(lhs.powf(rhs)))
                }
                _ => None,
            },
        }
    }
}

impl FunctionCall {
    #[allow(clippy::unused_self)]
    fn const_eval(&self, _ctx: &mut Lowerer, _ty: &Type) -> Option<LiteralKind> {
        None
    }
}

impl IndexExpr {
    #[allow(clippy::unused_self)]
    fn const_eval(&self, _ctx: &mut Lowerer, _ty: &Type) -> Option<LiteralKind> {
        None
    }
}

impl Cast {
    fn const_eval(&self, ctx: &mut Lowerer) -> Option<LiteralKind> {
        match &self.ty {
            Type::Bool(..) => cast_to_bool(self, ctx),
            Type::Int(..) => cast_to_int(self, ctx),
            Type::UInt(..) => cast_to_uint(self, ctx),
            Type::Float(..) => cast_to_float(self, ctx),
            Type::Angle(..) => cast_to_angle(self, ctx),
            Type::Bit(..) => cast_to_bit(self, ctx),
            Type::BitArray(..) => cast_to_bitarray(self, ctx),
            _ => None,
        }
    }
}

/// +---------------+-----------------------------------------+
/// | Allowed casts | Casting from                            |
/// +---------------+------+-----+------+-------+-------+-----+
/// | Casting to    | bool | int | uint | float | angle | bit |
/// +---------------+------+-----+------+-------+-------+-----+
/// | bool          | -    | Yes | Yes  | Yes   | Yes   | Yes |
/// +---------------+------+-----+------+-------+-------+-----+
fn cast_to_bool(cast: &Cast, ctx: &mut Lowerer) -> Option<LiteralKind> {
    use LiteralKind::{Angle, Bit, Bitstring, Bool, Float, Int};
    let lit = cast.expr.const_eval(ctx)?;

    match &cast.expr.ty {
        Type::Bool(..) => Some(lit),
        Type::Bit(..) => rewrap_lit!(lit, Bit(val), Bool(val)),
        Type::BitArray(..) => rewrap_lit!(lit, Bitstring(val, _), Bool(val != BigInt::ZERO)),
        Type::Int(..) | Type::UInt(..) => rewrap_lit!(lit, Int(val), Bool(val != 0)),
        Type::Float(..) => rewrap_lit!(lit, Float(val), Bool(val != 0.0)),
        Type::Angle(..) => rewrap_lit!(lit, Angle(val), Bool(val.into())),
        _ => None,
    }
}

/// +---------------+-----------------------------------------+
/// | Allowed casts | Casting from                            |
/// +---------------+------+-----+------+-------+-------+-----+
/// | Casting to    | bool | int | uint | float | angle | bit |
/// +---------------+------+-----+------+-------+-------+-----+
/// | int           | Yes  | -   | Yes  | Yes   | No    | Yes |
/// +---------------+------+-----+------+-------+-------+-----+
fn cast_to_int(cast: &Cast, ctx: &mut Lowerer) -> Option<LiteralKind> {
    use LiteralKind::{Bit, Bitstring, Bool, Float, Int};
    let lit = cast.expr.const_eval(ctx)?;

    match &cast.expr.ty {
        Type::Bool(..) => rewrap_lit!(lit, Bool(val), Int(i64::from(val))),
        Type::Bit(..) => rewrap_lit!(lit, Bit(val), Int(i64::from(val))),
        Type::BitArray(..) => {
            rewrap_lit!(lit, Bitstring(val, _), Int(i64::try_from(val).ok()?))
        }
        // UInt Overflowing behavior.
        // This is tricky because the inner representation of UInt
        // is already an i64. Therefore, there is nothing to do.
        Type::Int(..) | Type::UInt(..) => Some(lit),
        Type::Float(..) => rewrap_lit!(lit, Float(val), {
            #[allow(clippy::cast_possible_truncation)]
            Int(val as i64)
        }),
        _ => None,
    }
}

/// +---------------+-----------------------------------------+
/// | Allowed casts | Casting from                            |
/// +---------------+------+-----+------+-------+-------+-----+
/// | Casting to    | bool | int | uint | float | angle | bit |
/// +---------------+------+-----+------+-------+-------+-----+
/// | uint          | Yes  | Yes | -    | Yes   | No    | Yes |
/// +---------------+------+-----+------+-------+-------+-----+
fn cast_to_uint(cast: &Cast, ctx: &mut Lowerer) -> Option<LiteralKind> {
    use LiteralKind::{Bit, Bitstring, Bool, Float, Int};
    let lit = cast.expr.const_eval(ctx)?;

    match &cast.expr.ty {
        Type::Bool(..) => rewrap_lit!(lit, Bool(val), Int(i64::from(val))),
        Type::Bit(..) => rewrap_lit!(lit, Bit(val), Int(i64::from(val))),
        Type::BitArray(..) => {
            rewrap_lit!(lit, Bitstring(val, _), Int(i64::try_from(val).ok()?))
        }
        // UInt Overflowing behavior.
        // This is tricky because the inner representation of UInt
        // is already an i64. Therefore, there is nothing to do.
        Type::Int(..) | Type::UInt(..) => Some(lit),
        Type::Float(..) => rewrap_lit!(lit, Float(val), {
            #[allow(clippy::cast_possible_truncation)]
            Int(val as i64)
        }),
        _ => None,
    }
}

/// +---------------+-----------------------------------------+
/// | Allowed casts | Casting from                            |
/// +---------------+------+-----+------+-------+-------+-----+
/// | Casting to    | bool | int | uint | float | angle | bit |
/// +---------------+------+-----+------+-------+-------+-----+
/// | float         | Yes  | Yes | Yes  | -     | No    | No  |
/// +---------------+------+-----+------+-------+-------+-----+
fn cast_to_float(cast: &Cast, ctx: &mut Lowerer) -> Option<LiteralKind> {
    use LiteralKind::{Bool, Float, Int};
    let lit = cast.expr.const_eval(ctx)?;

    match &cast.expr.ty {
        Type::Bool(..) => rewrap_lit!(lit, Bool(val), Float(if val { 1.0 } else { 0.0 })),
        Type::Int(..) | Type::UInt(..) => rewrap_lit!(lit, Int(val), {
            #[allow(clippy::cast_precision_loss)]
            Float(safe_i64_to_f64(val)?)
        }),
        Type::Float(..) => Some(lit),
        _ => None,
    }
}

/// +---------------+-----------------------------------------+
/// | Allowed casts | Casting from                            |
/// +---------------+------+-----+------+-------+-------+-----+
/// | Casting to    | bool | int | uint | float | angle | bit |
/// +---------------+------+-----+------+-------+-------+-----+
/// | angle         | No   | No  | No   | Yes   | -     | Yes |
/// +---------------+------+-----+------+-------+-------+-----+
fn cast_to_angle(cast: &Cast, ctx: &mut Lowerer) -> Option<LiteralKind> {
    use LiteralKind::{Angle, Bit, Bitstring, Float};
    let lit = cast.expr.const_eval(ctx)?;

    match &cast.expr.ty {
        Type::Float(size, _) => rewrap_lit!(
            lit,
            Float(val),
            Angle(angle::Angle::from_f64_maybe_sized(val, *size))
        ),
        Type::Angle(..) => rewrap_lit!(
            lit,
            Angle(val),
            Angle(val.cast_to_maybe_sized(cast.ty.width()))
        ),
        Type::Bit(..) => rewrap_lit!(
            lit,
            Bit(val),
            Angle(angle::Angle {
                value: val.into(),
                size: 1
            })
        ),
        Type::BitArray(..) => rewrap_lit!(
            lit,
            Bitstring(val, size),
            Angle(angle::Angle {
                value: val.try_into().ok()?,
                size
            })
        ),
        _ => None,
    }
}

/// +---------------+-----------------------------------------+
/// | Allowed casts | Casting from                            |
/// +---------------+------+-----+------+-------+-------+-----+
/// | Casting to    | bool | int | uint | float | angle | bit |
/// +---------------+------+-----+------+-------+-------+-----+
/// | bit           | Yes  | Yes | Yes  | No    | Yes   | -   |
/// +---------------+------+-----+------+-------+-------+-----+
fn cast_to_bit(cast: &Cast, ctx: &mut Lowerer) -> Option<LiteralKind> {
    use LiteralKind::{Angle, Bit, Bool, Int};
    let lit = cast.expr.const_eval(ctx)?;

    match &cast.expr.ty {
        Type::Bool(..) => rewrap_lit!(lit, Bool(val), Bit(val)),
        Type::Bit(..) => Some(lit),
        Type::Int(..) | Type::UInt(..) => rewrap_lit!(lit, Int(val), Bit(val != 0)),
        Type::Angle(..) => rewrap_lit!(lit, Angle(val), Bit(val.value != 0)),
        _ => None,
    }
}

/// +---------------+-----------------------------------------+
/// | Allowed casts | Casting from                            |
/// +---------------+------+-----+------+-------+-------+-----+
/// | Casting to    | bool | int | uint | float | angle | bit |
/// +---------------+------+-----+------+-------+-------+-----+
/// | bitarray      | Yes  | Yes | Yes  | No    | Yes   | -   |
/// +---------------+------+-----+------+-------+-------+-----+
fn cast_to_bitarray(cast: &Cast, ctx: &mut Lowerer) -> Option<LiteralKind> {
    use LiteralKind::{Angle, Bit, Bitstring, Bool, Int};
    let lit = cast.expr.const_eval(ctx)?;

    let Type::BitArray(dims, _) = &cast.ty else {
        unreachable!("we got here after matching Type::BitArray in Cast::const_eval");
    };

    let ArrayDimensions::One(size) = dims else {
        ctx.push_unsupported_error_message("multidimensional arrays", cast.span);
        return None;
    };
    let size = *size;

    match &cast.expr.ty {
        Type::Bool(..) => rewrap_lit!(lit, Bool(val), Bitstring(BigInt::from(val), size)),
        Type::Angle(..) => rewrap_lit!(lit, Angle(val), {
            let new_val = val.cast_to_maybe_sized(Some(size));
            Bitstring(new_val.value.into(), size)
        }),
        Type::Bit(..) => rewrap_lit!(lit, Bit(val), Bitstring(BigInt::from(val), size)),
        Type::BitArray(..) => rewrap_lit!(lit, Bitstring(val, rhs_size), {
            if rhs_size > size {
                ctx.push_const_eval_error(ConstEvalError::ValueOverflow(
                    cast.expr.ty.to_string(),
                    cast.ty.to_string(),
                    cast.span,
                ));
                return None;
            }
            Bitstring(val, size)
        }),
        Type::Int(..) | Type::UInt(..) => rewrap_lit!(lit, Int(val), {
            let actual_bits = number_of_bits(val);
            if actual_bits > size {
                ctx.push_const_eval_error(ConstEvalError::ValueOverflow(
                    cast.expr.ty.to_string(),
                    cast.ty.to_string(),
                    cast.span,
                ));
                return None;
            }
            Bitstring(BigInt::from(val), size)
        }),
        _ => None,
    }
}

fn number_of_bits(mut val: i64) -> u32 {
    let mut bits = 0;
    while val != 0 {
        val >>= 1;
        bits += 1;
    }
    bits
}
