// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;

use crate::Evaluator;

fn check_expression(expr: &str, expect: &Expect) {
    let context = qsc_frontend::compile(&[], expr);
    assert!(context.errors().is_empty());
    let mut eval = Evaluator::default();
    expect.assert_debug_eq(&eval.run(&context));
}

#[test]
fn array_expr() {
    check_expression(
        "[1, 2, 3]",
        &expect![[r#"
        Ok(
            Array(
                [
                    Int(
                        1,
                    ),
                    Int(
                        2,
                    ),
                    Int(
                        3,
                    ),
                ],
            ),
        )
    "#]],
    );
}

#[test]
fn block_expr() {
    check_expression(
        indoc! { "{
            let x = 1;
            let y = 2;
            x + y
        }"},
        &expect![[r#"
            Err(
                Error {
                    span: Span {
                        lo: 6,
                        hi: 16,
                    },
                    kind: Unimplemented,
                },
            )
        "#]],
    );
}

#[test]
fn fail_expr() {
    check_expression(
        r#"fail "This is a failure""#,
        &expect![[r#"
        Err(
            Error {
                span: Span {
                    lo: 0,
                    hi: 24,
                },
                kind: UserFail(
                    "This is a failure",
                ),
            },
        )
    "#]],
    );
}

#[test]
fn fail_shortcut_expr() {
    check_expression(
        r#"{ fail "Got Here!"; fail "Shouldn't get here..."; }"#,
        &expect![[r#"
        Err(
            Error {
                span: Span {
                    lo: 2,
                    hi: 18,
                },
                kind: UserFail(
                    "Got Here!",
                ),
            },
        )
    "#]],
    );
}

#[test]
fn array_index_expr() {
    check_expression(
        "[1, 2, 3][1]",
        &expect![[r#"
        Ok(
            Int(
                2,
            ),
        )
    "#]],
    );
}

#[test]
fn array_index_out_of_range_expr() {
    check_expression(
        "[1, 2, 3][4]",
        &expect![[r#"
        Err(
            Error {
                span: Span {
                    lo: 10,
                    hi: 11,
                },
                kind: OutOfRange(
                    4,
                ),
            },
        )
    "#]],
    );
}

#[test]
fn literal_big_int_expr() {
    check_expression(
        "9_223_372_036_854_775_808L",
        &expect![[r#"
            Ok(
                BigInt(
                    9223372036854775808,
                ),
            )
        "#]],
    );
}

#[test]
fn literal_bool_false_expr() {
    check_expression(
        "false",
        &expect![[r#"
        Ok(
            Bool(
                false,
            ),
        )
    "#]],
    );
}

#[test]
fn literal_bool_true_expr() {
    check_expression(
        "true",
        &expect![[r#"
        Ok(
            Bool(
                true,
            ),
        )
    "#]],
    );
}

#[test]
fn literal_double_expr() {
    check_expression(
        "4.2",
        &expect![[r#"
        Ok(
            Double(
                4.2,
            ),
        )
    "#]],
    );
}

#[test]
fn literal_double_trailing_dot_expr() {
    check_expression(
        "4.",
        &expect![[r#"
        Ok(
            Double(
                4.0,
            ),
        )
    "#]],
    );
}

#[test]
fn literal_int_expr() {
    check_expression(
        "42",
        &expect![[r#"
        Ok(
            Int(
                42,
            ),
        )
    "#]],
    );
}

#[test]
fn literal_int_too_big_expr() {
    check_expression(
        "9_223_372_036_854_775_808",
        &expect![[r#"
            Err(
                Error {
                    span: Span {
                        lo: 0,
                        hi: 25,
                    },
                    kind: IntegerSize,
                },
            )
        "#]],
    );
}

#[test]
fn literal_pauli_i_expr() {
    check_expression(
        "PauliI",
        &expect![[r#"
        Ok(
            Pauli(
                I,
            ),
        )
    "#]],
    );
}

#[test]
fn literal_pauli_x_expr() {
    check_expression(
        "PauliX",
        &expect![[r#"
        Ok(
            Pauli(
                X,
            ),
        )
    "#]],
    );
}

#[test]
fn literal_pauli_y_expr() {
    check_expression(
        "PauliY",
        &expect![[r#"
        Ok(
            Pauli(
                Y,
            ),
        )
    "#]],
    );
}

#[test]
fn literal_pauli_z_expr() {
    check_expression(
        "PauliZ",
        &expect![[r#"
        Ok(
            Pauli(
                Z,
            ),
        )
    "#]],
    );
}

#[test]
fn literal_result_one_expr() {
    check_expression(
        "One",
        &expect![[r#"
        Ok(
            Result(
                true,
            ),
        )
    "#]],
    );
}

#[test]
fn literal_result_zero_expr() {
    check_expression(
        "Zero",
        &expect![[r#"
        Ok(
            Result(
                false,
            ),
        )
    "#]],
    );
}

#[test]
fn literal_string_expr() {
    check_expression(
        r#""foo""#,
        &expect![[r#"
        Ok(
            String(
                "foo",
            ),
        )
    "#]],
    );
}

#[test]
fn paren_expr() {
    check_expression(
        "(42)",
        &expect![[r#"
        Ok(
            Int(
                42,
            ),
        )
    "#]],
    );
}

#[test]
fn range_all_expr() {
    check_expression(
        "...",
        &expect![[r#"
        Ok(
            Range(
                None,
                None,
                None,
            ),
        )
    "#]],
    );
}

#[test]
fn range_end_expr() {
    check_expression(
        "...3",
        &expect![[r#"
            Ok(
                Range(
                    None,
                    None,
                    Some(
                        3,
                    ),
                ),
            )
        "#]],
    );
}

#[test]
fn range_step_end_expr() {
    check_expression(
        "...2..3",
        &expect![[r#"
            Ok(
                Range(
                    None,
                    Some(
                        2,
                    ),
                    Some(
                        3,
                    ),
                ),
            )
        "#]],
    );
}

#[test]
fn range_start_expr() {
    check_expression(
        "1...",
        &expect![[r#"
            Ok(
                Range(
                    Some(
                        1,
                    ),
                    None,
                    None,
                ),
            )
        "#]],
    );
}

#[test]
fn range_start_end_expr() {
    check_expression(
        "1..3",
        &expect![[r#"
            Ok(
                Range(
                    Some(
                        1,
                    ),
                    None,
                    Some(
                        3,
                    ),
                ),
            )
        "#]],
    );
}

#[test]
fn range_start_step_expr() {
    check_expression(
        "1..2...",
        &expect![[r#"
            Ok(
                Range(
                    Some(
                        1,
                    ),
                    Some(
                        2,
                    ),
                    None,
                ),
            )
        "#]],
    );
}

#[test]
fn range_start_step_end_expr() {
    check_expression(
        "1..2..3",
        &expect![[r#"
            Ok(
                Range(
                    Some(
                        1,
                    ),
                    Some(
                        2,
                    ),
                    Some(
                        3,
                    ),
                ),
            )
        "#]],
    );
}

#[test]
fn tuple_expr() {
    check_expression(
        "(1, 2, 3)",
        &expect![[r#"
        Ok(
            Tuple(
                [
                    Int(
                        1,
                    ),
                    Int(
                        2,
                    ),
                    Int(
                        3,
                    ),
                ],
            ),
        )
    "#]],
    );
}

#[test]
fn if_true_expr() {
    check_expression(
        r#"if true {fail "Got Here!";}"#,
        &expect![[r#"
            Err(
                Error {
                    span: Span {
                        lo: 9,
                        hi: 25,
                    },
                    kind: UserFail(
                        "Got Here!",
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn if_false_expr() {
    check_expression(
        r#"if false {fail "Shouldn't get here...";}"#,
        &expect![[r#"
            Ok(
                Tuple(
                    [],
                ),
            )
        "#]],
    );
}

#[test]
fn if_else_true_expr() {
    check_expression(
        r#"if true {fail "Got Here!";} else {fail "Shouldn't get here..."}"#,
        &expect![[r#"
            Err(
                Error {
                    span: Span {
                        lo: 9,
                        hi: 25,
                    },
                    kind: UserFail(
                        "Got Here!",
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn if_else_false_expr() {
    check_expression(
        r#"if false {fail "Shouldn't get here...";} else {fail "Got Here!"}"#,
        &expect![[r#"
            Err(
                Error {
                    span: Span {
                        lo: 47,
                        hi: 63,
                    },
                    kind: UserFail(
                        "Got Here!",
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn if_elif_true_true_expr() {
    check_expression(
        r#"if true {fail "Got Here!";} elif true {fail "Shouldn't get here..."}"#,
        &expect![[r#"
            Err(
                Error {
                    span: Span {
                        lo: 9,
                        hi: 25,
                    },
                    kind: UserFail(
                        "Got Here!",
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn if_elif_false_true_expr() {
    check_expression(
        r#"if false {fail "Shouldn't get here...";} elif true {fail "Got Here!"}"#,
        &expect![[r#"
            Err(
                Error {
                    span: Span {
                        lo: 52,
                        hi: 68,
                    },
                    kind: UserFail(
                        "Got Here!",
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn if_elif_false_false_expr() {
    check_expression(
        r#"if false {fail "Shouldn't get here...";} elif false {fail "Shouldn't get here..."}"#,
        &expect![[r#"
            Ok(
                Tuple(
                    [],
                ),
            )
        "#]],
    );
}

#[test]
fn if_elif_else_true_true_expr() {
    check_expression(
        r#"if true {fail "Got Here!";} elif true {fail "Shouldn't get here..."} else {fail "Shouldn't get here..."}"#,
        &expect![[r#"
            Err(
                Error {
                    span: Span {
                        lo: 9,
                        hi: 25,
                    },
                    kind: UserFail(
                        "Got Here!",
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn if_elif_else_false_true_expr() {
    check_expression(
        r#"if false {fail "Shouldn't get here...";} elif true {fail "Got Here!"} else {fail "Shouldn't get here..."}"#,
        &expect![[r#"
            Err(
                Error {
                    span: Span {
                        lo: 52,
                        hi: 68,
                    },
                    kind: UserFail(
                        "Got Here!",
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn if_elif_else_false_false_expr() {
    check_expression(
        r#"if false {fail "Shouldn't get here...";} elif false {fail "Shouldn't get here..."} else {fail "Got Here!"}"#,
        &expect![[r#"
            Err(
                Error {
                    span: Span {
                        lo: 89,
                        hi: 105,
                    },
                    kind: UserFail(
                        "Got Here!",
                    ),
                },
            )
        "#]],
    );
}
