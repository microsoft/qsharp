// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use oq3_semantics::types::Type;
use oq3_syntax::ast::{ArithOp, BinaryOp, Designator, Expr, Literal, LiteralKind};
use qsc::Span;

pub(crate) fn span_for_named_item<T: oq3_syntax::ast::HasName>(value: &T) -> Span {
    let Some(name) = value.name() else {
        return Span::default();
    };
    let Some(ident) = name.ident_token() else {
        return Span::default();
    };
    text_range_to_span(ident.text_range())
}

pub(crate) fn text_range_to_span(range: oq3_syntax::TextRange) -> Span {
    Span {
        lo: range.start().into(),
        hi: range.end().into(),
    }
}

pub(crate) fn span_for_syntax_node(node: &oq3_syntax::SyntaxNode) -> Span {
    text_range_to_span(node.text_range())
}

pub(crate) fn span_for_syntax_token(token: &oq3_syntax::SyntaxToken) -> Span {
    text_range_to_span(token.text_range())
}

pub(crate) fn safe_u128_to_f64(value: u128) -> Option<f64> {
    if value <= u128::from(i64::MAX as u64) {
        let value = i64::try_from(value).ok()?;
        #[allow(clippy::cast_possible_truncation)]
        if value <= f64::MAX as i64 {
            #[allow(clippy::cast_precision_loss)]
            Some(value as f64)
        } else {
            None
        }
    } else {
        None
    }
}

pub(crate) fn extract_dims_from_designator(designator: Option<Designator>) -> Option<u32> {
    let designator = designator?;
    match designator.expr() {
        Some(Expr::Literal(lit)) => match lit.kind() {
            LiteralKind::IntNumber(int) => {
                // qasm parser stores ints as u128
                let value = int.value().expect("Designator must be a literal integer");
                let value: u32 = u32::try_from(value).expect("Designator must fit in u32");
                Some(value)
            }
            _ => {
                unreachable!("designator must be a literal integer")
            }
        },
        None => None,
        _ => {
            unreachable!("designator must be a literal integer")
        }
    }
}

pub(crate) fn get_designator_from_scalar_type(
    ty: &oq3_syntax::ast::ScalarType,
) -> Option<oq3_syntax::ast::Designator> {
    if let Some(scalar_ty) = ty.scalar_type() {
        // we have a complex type, need to grab the inner
        // designator for the width
        scalar_ty.designator()
    } else {
        ty.designator()
    }
}

/// Symmetric arithmetic conversions are applied to:
/// binary arithmetic *, /, %, +, -
/// relational operators <, >, <=, >=, ==, !=
/// binary bitwise arithmetic &, ^, |,
pub(crate) fn requires_symmetric_conversion(op: BinaryOp) -> bool {
    match op {
        BinaryOp::LogicOp(_) | BinaryOp::CmpOp(_) => true,
        BinaryOp::ArithOp(arith_op) => match arith_op {
            ArithOp::Mul
            | ArithOp::Div
            | ArithOp::Rem
            | ArithOp::Add
            | ArithOp::Sub
            | ArithOp::BitAnd
            | ArithOp::BitXor
            | ArithOp::BitOr => true,
            ArithOp::Shl | ArithOp::Shr => false,
        },
        #[allow(clippy::match_same_arms)]
        BinaryOp::ConcatenationOp => {
            // concatenation is a special case where we can't have a symmetric conversion
            // as the lhs and rhs must be of the same type
            false
        }
        BinaryOp::Assignment { op: _ } => false,
    }
}

pub(crate) fn requires_types_already_match_conversion(op: BinaryOp) -> bool {
    match op {
        BinaryOp::ConcatenationOp => {
            // concatenation is a special case where we can't have a symmetric conversion
            // as the lhs and rhs must be of the same type
            true
        }
        _ => false,
    }
}

// integer promotions are applied only to both operands of
// the shift operators << and >>
pub(crate) fn binop_requires_int_conversion(op: BinaryOp) -> bool {
    match op {
        BinaryOp::ArithOp(arith_op) => matches!(arith_op, ArithOp::Shl | ArithOp::Shr),
        BinaryOp::Assignment { op } => matches!(op, Some(ArithOp::Shl | ArithOp::Shr)),
        _ => false,
    }
}

pub(crate) fn can_cast_literal_with_value_knowledge(lhs_ty: &Type, literal: &Literal) -> bool {
    if matches!(lhs_ty, &Type::Bit(..)) {
        if let LiteralKind::IntNumber(value) = literal.kind() {
            let value = value.value().expect("IntNumber must have a value");
            return value == 0 || value == 1;
        }
    }
    if matches!(lhs_ty, &Type::UInt(..)) {
        if let LiteralKind::IntNumber(_) = literal.kind() {
            // Value can't be negative as IntNumber is unsigned
            // any sign would come from a prefix expression
            return true;
        }
    }
    false
}
