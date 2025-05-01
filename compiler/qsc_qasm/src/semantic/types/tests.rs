// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::indexed_type_builder;
use super::ArrayDimensions;
use super::Type;
use crate::semantic::ast::Expr;
use crate::semantic::ast::ExprKind;
use crate::semantic::ast::Index;
use crate::semantic::ast::LiteralKind;
use crate::semantic::ast::Range;
use expect_test::expect;
use qsc_data_structures::span::Span;

#[test]
fn indexed_type_has_right_dimensions() {
    let base_ty_builder = || Type::Bool(false);
    let array_ty_builder = |dims| Type::BoolArray(dims, false);
    let dims = ArrayDimensions::Three(2, 3, 4);

    let index = Expr {
        span: Span::default(),
        kind: Box::new(ExprKind::Lit(LiteralKind::Int(0))),
        ty: Type::Int(None, true),
    };
    let indices = Index::Expr(index);
    let indexed_ty = indexed_type_builder(base_ty_builder, array_ty_builder, &dims, &[indices]);

    expect!["BoolArray(Two(3, 4), false)"].assert_eq(&format!("{indexed_ty}"));
}

#[test]
fn sliced_type_has_right_dimensions() {
    let base_ty_builder = || Type::Bool(false);
    let array_ty_builder = |dims| Type::BoolArray(dims, false);
    let dims = ArrayDimensions::Three(5, 1, 2);

    let make_expr = |val| Expr {
        span: Span::default(),
        kind: Box::new(ExprKind::Lit(LiteralKind::Int(val))),
        ty: Type::Int(None, true),
    };
    let indices = Index::Range(Range {
        span: Span::default(),
        start: Some(make_expr(1)),
        end: Some(make_expr(3)),
        step: None,
    });
    let indexed_ty = indexed_type_builder(base_ty_builder, array_ty_builder, &dims, &[indices]);

    expect!["BoolArray(Three(3, 1, 2), false)"].assert_eq(&format!("{indexed_ty}"));
}
