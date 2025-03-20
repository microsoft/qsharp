// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module allows us to perform const evaluation at lowering time.
//! The purpose of this is to be able to compute the widths of types
//! and sizes of arrays. Therefore, those are the only const evaluation
//! paths that are implemented.

use super::{
    BinOp, BinaryOpExpr, Cast, Expr, ExprKind, FunctionCall, IndexExpr, IndexedIdent, LiteralKind,
    SymbolId, UnaryOp, UnaryOpExpr,
};
use crate::semantic::{symbols::SymbolTable, types::Type};
use num_bigint::BigInt;

impl Expr {
    pub fn const_eval(&self, symbols: &SymbolTable) -> Option<LiteralKind> {
        let ty = &self.ty;
        if !ty.is_const() {
            return None;
        }

        match &*self.kind {
            ExprKind::Ident(symbol_id) => symbol_id.const_eval(symbols),
            ExprKind::IndexedIdentifier(indexed_ident) => indexed_ident.const_eval(symbols),
            ExprKind::UnaryOp(unary_op_expr) => unary_op_expr.const_eval(symbols, ty),
            ExprKind::BinaryOp(binary_op_expr) => binary_op_expr.const_eval(symbols, ty),
            ExprKind::Lit(literal_kind) => Some(literal_kind.clone()),
            ExprKind::FunctionCall(function_call) => function_call.const_eval(symbols, ty),
            ExprKind::Cast(cast) => cast.const_eval(symbols, ty),
            ExprKind::IndexExpr(index_expr) => index_expr.const_eval(symbols, ty),
            ExprKind::Paren(expr) => expr.const_eval(symbols),
            // Measurements are non-const, so we don't need to implement them.
            ExprKind::Measure(_) | ExprKind::Err => None,
        }
    }
}

impl SymbolId {
    fn const_eval(self, symbols: &SymbolTable) -> Option<LiteralKind> {
        let symbol = symbols[self].clone();
        symbol
            .const_eval() // get the value of the symbol (an Expr)
            .const_eval(symbols) // const eval that Expr
    }
}

impl IndexedIdent {
    #[allow(clippy::unused_self)]
    fn const_eval(&self, _symbols: &SymbolTable) -> Option<LiteralKind> {
        None
    }
}

impl UnaryOpExpr {
    fn const_eval(&self, symbols: &SymbolTable, ty: &Type) -> Option<LiteralKind> {
        let lit = self.expr.const_eval(symbols)?;
        let op = &self.op;

        match ty {
            Type::Float(..) => {
                if let LiteralKind::Float(val) = lit {
                    match op {
                        UnaryOp::Neg => Some(LiteralKind::Float(-val)),
                        UnaryOp::NotB | UnaryOp::NotL => None,
                    }
                } else {
                    None
                }
            }
            Type::Int(..) => match lit {
                LiteralKind::Int(val) => match op {
                    UnaryOp::Neg => Some(LiteralKind::Int(-val)),
                    UnaryOp::NotB => Some(LiteralKind::Int(!val)),
                    UnaryOp::NotL => Some(LiteralKind::Int(i64::from(val == 0))),
                },
                LiteralKind::BigInt(val) => match op {
                    UnaryOp::Neg => Some(LiteralKind::BigInt(-val)),
                    UnaryOp::NotB => Some(LiteralKind::BigInt(!val)),
                    UnaryOp::NotL => Some(LiteralKind::Int(i64::from(val == BigInt::ZERO))),
                },
                _ => None,
            },
            Type::UInt(..) => match lit {
                LiteralKind::Int(val) => match op {
                    UnaryOp::Neg => {
                        // C99 first negates the value, as if it were signed,
                        let val = -val;

                        // and then converts it to unsigned, wrapping around if necessary.
                        // We convert to u64 to mimic the wrapping around behavior.
                        // TODO: we need to issue the same lint in Q#.
                        #[allow(clippy::cast_sign_loss)]
                        let u_val = val as u64;

                        // Finally we go back to i64 because that's our internal representation.
                        // TODO: we need to issue the same lint in Q#.
                        #[allow(clippy::cast_possible_wrap)]
                        Some(LiteralKind::Int(u_val as i64))
                    }
                    UnaryOp::NotB => {
                        // We convert our value to u64.
                        // TODO: we need to issue the same lint in Q#.
                        #[allow(clippy::cast_sign_loss)]
                        let u_val = val as u64;

                        // Then we apply the bitwise operator.
                        let u_val = !u_val;

                        // Finally we go back to i64 because that's our internal representation.
                        // TODO: we need to issue the same lint in Q#.
                        #[allow(clippy::cast_possible_wrap)]
                        Some(LiteralKind::Int(u_val as i64))
                    }
                    UnaryOp::NotL => Some(LiteralKind::Int(i64::from(val == 0))),
                },
                // It doesn't make sense to have operations that result in BigInts for array sizes
                // or type widths. So, we return `None`.
                _ => None,
            },
            _ => None,
        }
    }
}

impl BinaryOpExpr {
    #[allow(clippy::too_many_lines)]
    fn const_eval(&self, symbols: &SymbolTable, ty: &Type) -> Option<LiteralKind> {
        if !self.lhs.ty.is_const() || !self.rhs.ty.is_const() {
            return None;
        }

        // By this point it is guaranteed that the lhs and rhs are of the same type.
        // Any conversions have been made explicit by inserting casts during lowering.
        // Note: the type of the binary expression doesn't need to be the same as the
        //       operands, for example, comparison operators can have integer operands
        //       but their type is boolean.
        // We can write a simpler implementation under that assumption.
        assert_eq!(self.lhs.ty, self.rhs.ty);

        let lhs = self.lhs.const_eval(symbols)?;
        let rhs = self.rhs.const_eval(symbols)?;
        let op = &self.op;

        match ty {
            Type::Float(..) => {
                if let (LiteralKind::Float(lhs), LiteralKind::Float(rhs)) = (lhs, rhs) {
                    Some(match op {
                        BinOp::Add => LiteralKind::Float(lhs + rhs),
                        BinOp::Div => LiteralKind::Float(lhs / rhs),
                        BinOp::Exp => LiteralKind::Float(lhs.powf(rhs)),
                        BinOp::Sub => LiteralKind::Float(lhs - rhs),
                        BinOp::Mul => LiteralKind::Float(lhs * rhs),
                        // If the output type is Float, we don't need to implement
                        // any of the comparison, bitwise, or mod operators.
                        BinOp::AndB
                        | BinOp::AndL
                        | BinOp::Eq
                        | BinOp::Gt
                        | BinOp::Gte
                        | BinOp::Lt
                        | BinOp::Lte
                        | BinOp::Mod
                        | BinOp::Neq
                        | BinOp::OrB
                        | BinOp::OrL
                        | BinOp::Shl
                        | BinOp::Shr
                        | BinOp::XorB => return None,
                    })
                } else {
                    None
                }
            }
            Type::Int(..) | Type::UInt(..) => {
                if let (LiteralKind::Int(lhs), LiteralKind::Int(rhs)) = (lhs, rhs) {
                    Some(match op {
                        // Arithmetic.
                        BinOp::Add => LiteralKind::Int(lhs + rhs),
                        BinOp::Sub => LiteralKind::Int(lhs - rhs),
                        BinOp::Mul => LiteralKind::Int(lhs * rhs),
                        BinOp::Div => LiteralKind::Int(lhs / rhs),
                        BinOp::Exp => LiteralKind::Int(lhs.wrapping_pow(u32::try_from(rhs).ok()?)),
                        BinOp::Mod => LiteralKind::Int(lhs % rhs),
                        // Bitwise.
                        BinOp::Shl => LiteralKind::Int(lhs << rhs),
                        BinOp::Shr => LiteralKind::Int(lhs >> rhs),
                        BinOp::AndB => LiteralKind::Int(lhs & rhs),
                        BinOp::OrB => LiteralKind::Int(lhs | rhs),
                        BinOp::XorB => LiteralKind::Int(lhs ^ rhs),
                        // Comparison.
                        BinOp::AndL
                        | BinOp::Eq
                        | BinOp::Gt
                        | BinOp::Gte
                        | BinOp::Lt
                        | BinOp::Lte
                        | BinOp::Neq
                        | BinOp::OrL => return None,
                    })
                } else {
                    None
                }
            }
            Type::Bool(..) | Type::Bit(..) => {
                match (lhs, rhs) {
                    (LiteralKind::Bool(lhs), LiteralKind::Bool(rhs)) => {
                        Some(match op {
                            // Logical and bitwise operators.
                            BinOp::AndB | BinOp::AndL => LiteralKind::Bool(lhs && rhs),
                            BinOp::OrB | BinOp::OrL => LiteralKind::Bool(lhs || rhs),
                            BinOp::XorB => LiteralKind::Bool(lhs ^ rhs),
                            // Comparison operators.
                            BinOp::Eq => LiteralKind::Bool(lhs == rhs),
                            BinOp::Neq => LiteralKind::Bool(lhs != rhs),
                            // This was originally `lhs > rhs` but clippy suggested this expression.
                            BinOp::Gt => LiteralKind::Bool(lhs && !rhs),
                            BinOp::Gte => LiteralKind::Bool(lhs >= rhs),
                            // This was originally `lhs < rhs` but clippy suggested this expression.
                            BinOp::Lt => LiteralKind::Bool(!lhs & rhs),
                            BinOp::Lte => LiteralKind::Bool(lhs <= rhs),
                            // Arithmetic.
                            BinOp::Add
                            | BinOp::Div
                            | BinOp::Exp
                            | BinOp::Mod
                            | BinOp::Mul
                            | BinOp::Shl
                            | BinOp::Shr
                            | BinOp::Sub => return None,
                        })
                    }
                    (LiteralKind::Int(lhs), LiteralKind::Int(rhs)) => {
                        Some(match op {
                            // Comparison operators.
                            BinOp::Eq => LiteralKind::Bool(lhs == rhs),
                            BinOp::Neq => LiteralKind::Bool(lhs != rhs),
                            BinOp::Gt => LiteralKind::Bool(lhs > rhs),
                            BinOp::Gte => LiteralKind::Bool(lhs >= rhs),
                            BinOp::Lt => LiteralKind::Bool(lhs < rhs),
                            BinOp::Lte => LiteralKind::Bool(lhs <= rhs),
                            // Logical and bitwise operators.
                            BinOp::AndB
                            | BinOp::AndL
                            | BinOp::OrB
                            | BinOp::OrL
                            | BinOp::XorB
                            // Arithmetic.
                            | BinOp::Add
                            | BinOp::Div
                            | BinOp::Exp
                            | BinOp::Mod
                            | BinOp::Mul
                            | BinOp::Shl
                            | BinOp::Shr
                            | BinOp::Sub => return None,
                        })
                    }
                    (LiteralKind::Float(lhs), LiteralKind::Float(rhs)) => {
                        Some(match op {
                            // Comparison operators.
                            // This is non-ideal because since this never reaches the Q# compiler
                            //  we are not issuing a double-comparison lint.
                            #[allow(clippy::float_cmp)]
                            BinOp::Eq => LiteralKind::Bool(lhs == rhs),
                            #[allow(clippy::float_cmp)]
                            BinOp::Neq => LiteralKind::Bool(lhs != rhs),
                            BinOp::Gt => LiteralKind::Bool(lhs > rhs),
                            BinOp::Gte => LiteralKind::Bool(lhs >= rhs),
                            BinOp::Lt => LiteralKind::Bool(lhs < rhs),
                            BinOp::Lte => LiteralKind::Bool(lhs <= rhs),
                            // Logical and bitwise operators.
                            BinOp::AndB
                            | BinOp::AndL
                            | BinOp::OrB
                            | BinOp::OrL
                            | BinOp::XorB
                            // Arithmetic.
                            | BinOp::Add
                            | BinOp::Div
                            | BinOp::Exp
                            | BinOp::Mod
                            | BinOp::Mul
                            | BinOp::Shl
                            | BinOp::Shr
                            | BinOp::Sub => return None,
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl FunctionCall {
    #[allow(clippy::unused_self)]
    fn const_eval(&self, _symbols: &SymbolTable, _ty: &Type) -> Option<LiteralKind> {
        None
    }
}

impl IndexExpr {
    #[allow(clippy::unused_self)]
    fn const_eval(&self, _symbols: &SymbolTable, _ty: &Type) -> Option<LiteralKind> {
        None
    }
}

impl Cast {
    fn const_eval(&self, symbols: &SymbolTable, ty: &Type) -> Option<LiteralKind> {
        let lit = self.expr.const_eval(symbols)?;
        let from_ty = &self.expr.ty;

        match ty {
            Type::Bool(_) => cast_to_bool(from_ty, lit),
            Type::Int(_, _) => cast_to_int(from_ty, lit),
            Type::UInt(_, _) => cast_to_uint(from_ty, lit),
            Type::Float(_, _) => cast_to_float(from_ty, lit),
            Type::Angle(_, _) => cast_to_angle(from_ty, lit),
            Type::Bit(_) => cast_to_bit(from_ty, lit),
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
fn cast_to_bool(ty: &Type, lit: LiteralKind) -> Option<LiteralKind> {
    match ty {
        Type::Bool(..) | Type::Bit(..) => Some(lit),
        Type::Int(..) | Type::UInt(..) => {
            if let LiteralKind::Int(val) = lit {
                Some(LiteralKind::Bool(val != 0))
            } else {
                None
            }
        }
        Type::Float(..) | Type::Angle(..) => {
            if let LiteralKind::Float(val) = lit {
                Some(LiteralKind::Bool(val != 0.0))
            } else {
                None
            }
        }
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
fn cast_to_int(ty: &Type, lit: LiteralKind) -> Option<LiteralKind> {
    match ty {
        Type::Bool(..) | Type::Bit(..) => {
            if let LiteralKind::Bool(val) = lit {
                Some(LiteralKind::Int(i64::from(val)))
            } else {
                None
            }
        }
        // TODO: UInt Overflowing behavior.
        //       This is tricky because the inner repersentation
        //       already is a i64. Therefore, there is nothing to do?
        Type::Int(..) | Type::UInt(..) => Some(lit),
        Type::Float(..) => {
            if let LiteralKind::Float(val) = lit {
                // TODO: we need to issue the same lint in Q#.
                #[allow(clippy::cast_possible_truncation)]
                Some(LiteralKind::Int(val as i64))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// +---------------+-----------------------------------------+
/// | Allowed casts | Casting from                            |
/// +---------------+------+-----+------+-------+-------+-----+
/// | Casting from  | bool | int | uint | float | angle | bit |
/// +---------------+------+-----+------+-------+-------+-----+
/// | uint          | Yes  | Yes | -    | Yes   | No    | Yes |
/// +---------------+------+-----+------+-------+-------+-----+
fn cast_to_uint(ty: &Type, lit: LiteralKind) -> Option<LiteralKind> {
    match ty {
        Type::Bool(..) | Type::Bit(..) => {
            if let LiteralKind::Bool(val) = lit {
                Some(LiteralKind::Int(i64::from(val)))
            } else {
                None
            }
        }
        // TODO: Int Overflowing behavior.
        //       This is tricky because the inner repersentation
        //       is a i64. Therefore, even we might end with the
        //       same result anyways. Need to think through this.
        Type::Int(..) | Type::UInt(..) => Some(lit),
        Type::Float(..) => {
            if let LiteralKind::Float(val) = lit {
                // TODO: we need to issue the same lint in Q#.
                #[allow(clippy::cast_possible_truncation)]
                Some(LiteralKind::Int(val as i64))
            } else {
                None
            }
        }
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
fn cast_to_float(ty: &Type, lit: LiteralKind) -> Option<LiteralKind> {
    match ty {
        Type::Bool(..) => {
            if let LiteralKind::Bool(val) = lit {
                Some(LiteralKind::Float(if val { 1.0 } else { 0.0 }))
            } else {
                None
            }
        }
        Type::Int(..) | Type::UInt(..) => {
            if let LiteralKind::Int(val) = lit {
                // TODO: we need to issue the same lint in Q#.
                #[allow(clippy::cast_precision_loss)]
                Some(LiteralKind::Float(val as f64))
            } else {
                None
            }
        }
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
fn cast_to_angle(ty: &Type, lit: LiteralKind) -> Option<LiteralKind> {
    match ty {
        Type::Float(..) | Type::Angle(..) => Some(lit),
        Type::Bit(..) => {
            if let LiteralKind::Bool(val) = lit {
                Some(LiteralKind::Float(if val { 1.0 } else { 0.0 }))
            } else {
                None
            }
        }
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
fn cast_to_bit(ty: &Type, lit: LiteralKind) -> Option<LiteralKind> {
    match ty {
        Type::Bool(..) | Type::Bit(..) => Some(lit),
        Type::Int(..) | Type::UInt(..) => {
            if let LiteralKind::Int(val) = lit {
                Some(LiteralKind::Bool(val != 0))
            } else {
                None
            }
        }
        Type::Angle(..) => {
            if let LiteralKind::Float(val) = lit {
                Some(LiteralKind::Bool(val != 0.0))
            } else {
                None
            }
        }
        _ => None,
    }
}
