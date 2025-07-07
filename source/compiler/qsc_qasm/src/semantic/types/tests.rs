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

fn make_int_expr(val: i64) -> Expr {
    Expr::new(
        Span::default(),
        ExprKind::Lit(LiteralKind::Int(val)),
        Type::Int(None, true),
    )
}

#[test]
fn indexed_type_has_right_dimensions() {
    let base_ty_builder = || Type::Bool(false);
    let array_ty_builder = |dims| Type::BoolArray(dims);
    let dims = ArrayDimensions::Three(2, 3, 4);

    let index = Index::Expr(make_int_expr(0));
    let indexed_ty = indexed_type_builder(base_ty_builder, array_ty_builder, &dims, &index);

    expect!["array[bool, 3, 4]"].assert_eq(&format!("{indexed_ty}"));
}

#[test]
fn sliced_type_has_right_dimensions() {
    let base_ty_builder = || Type::Bool(false);
    let array_ty_builder = |dims| Type::BoolArray(dims);
    let dims = ArrayDimensions::Three(5, 1, 2);

    let index = Index::Range(Box::new(Range {
        span: Span::default(),
        start: Some(make_int_expr(1)),
        end: Some(make_int_expr(3)),
        step: None,
    }));
    let indexed_ty = indexed_type_builder(base_ty_builder, array_ty_builder, &dims, &index);

    expect!["array[bool, 3, 1, 2]"].assert_eq(&format!("{indexed_ty}"));
}
