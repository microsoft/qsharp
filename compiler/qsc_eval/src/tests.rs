// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;

use crate::Evaluator;

fn check_expression(expr: &str, expect: &Expect) {
    let (package, context) = qsc_frontend::compile(&[], expr);
    assert!(context.errors().is_empty());
    let mut eval = Evaluator::new(&package, &context);
    match eval.run() {
        Ok(result) => expect.assert_eq(&result.to_string()),
        Err(e) => expect.assert_debug_eq(&e),
    }
}

#[test]
fn array_expr() {
    check_expression("[1, 2, 3]", &expect!["[1, 2, 3]"]);
}

#[test]
fn array_repeat_expr() {
    check_expression("[4, size = 3]", &expect!["[4, 4, 4]"]);
}

#[test]
fn array_repeat_type_error_expr() {
    check_expression(
        "[4, size = true]",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 11,
                    hi: 15,
                },
                kind: Type(
                    "Int",
                    "Bool",
                ),
            }
        "#]],
    );
}

#[test]
fn block_expr() {
    check_expression(
        indoc! { "{
            let x = 1;
            let y = x;
            y
        }"},
        &expect!["1"],
    );
}

#[test]
fn block_shadowing_expr() {
    check_expression(
        indoc! { "{
            let x = 1;
            let x = 2;
            x
        }"},
        &expect!["2"],
    );
}

#[test]
fn block_nested_shadowing_expr() {
    check_expression(
        indoc! { "{
            let x = 1;
            let y = {
                let x = 2;
                x
            };
            (y, x)
        }"},
        &expect!["(2, 1)"],
    );
}

#[test]
fn block_let_bind_tuple_expr() {
    check_expression(
        indoc! {"{
            let x = (1, 2);
            let (y, z) = x;
            (z, y)
        }"},
        &expect!["(2, 1)"],
    );
}

#[test]
fn block_let_bind_tuple_arity_error_expr() {
    check_expression(
        indoc! {"{
            let (x, y, z) = (0, 1);
        }"},
        &expect![[r#"
            Error {
                span: Span {
                    lo: 10,
                    hi: 19,
                },
                kind: TupleArity(
                    3,
                    2,
                ),
            }
        "#]],
    );
}

#[test]
fn block_mutable_expr() {
    check_expression(
        indoc! {"{
            mutable x = 0;
            x
        }"},
        &expect!["0"],
    );
}

#[test]
fn block_mutable_update_expr() {
    check_expression(
        indoc! {"{
            mutable x = 0;
            set x = 1;
            x
        }"},
        &expect!["1"],
    );
}

#[test]
fn block_mutable_update_tuple_expr() {
    check_expression(
        indoc! {"{
            mutable x = (0, 1);
            set x = (1, 2);
            x
        }"},
        &expect!["(1, 2)"],
    );
}

#[test]
fn block_mutable_update_tuple_item_expr() {
    check_expression(
        indoc! {"{
            mutable (x, y) = (0, 1);
            set (x, y) = (1, 2);
            (x, y)
        }"},
        &expect!["(1, 2)"],
    );
}

#[test]
fn block_mutable_update_tuple_hole_expr() {
    check_expression(
        indoc! {"{
            mutable (x, y) = (0, 1);
            set (_, y) = (1, 2);
            (x, y)
        }"},
        &expect!["(0, 2)"],
    );
}

#[test]
fn block_mutable_update_tuple_arity_error_expr() {
    check_expression(
        indoc! {"{
            mutable (x, y) = (0, 1);
            set (x, y) = (1, 2, 3);
            x
        }"},
        &expect![[r#"
            Error {
                span: Span {
                    lo: 39,
                    hi: 45,
                },
                kind: TupleArity(
                    2,
                    3,
                ),
            }
        "#]],
    );
}

#[test]
fn block_mutable_nested_scopes_expr() {
    check_expression(
        indoc! {"{
            mutable x = 0;
            {
                mutable y = 1;
                set x = y;
            }
            x
        }"},
        &expect!["1"],
    );
}

#[test]
fn block_mutable_nested_scopes_shadowing_expr() {
    check_expression(
        indoc! {"{
            mutable x = 0;
            {
                mutable x = 1;
                set x = 2;
            }
            x
        }"},
        &expect!["0"],
    );
}

#[test]
fn block_mutable_immutable_expr() {
    check_expression(
        indoc! {"{
            let x = 0;
            set x = 1;
        }"},
        &expect![[r#"
            Error {
                span: Span {
                    lo: 25,
                    hi: 26,
                },
                kind: Mutability,
            }
        "#]],
    );
}

#[test]
fn assign_invalid_expr() {
    check_expression(
        "set 0 = 1",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 4,
                    hi: 5,
                },
                kind: Unassignable,
            }
        "#]],
    );
}

#[test]
fn fail_expr() {
    check_expression(
        r#"fail "This is a failure""#,
        &expect![[r#"
            Error {
                span: Span {
                    lo: 0,
                    hi: 24,
                },
                kind: UserFail(
                    "This is a failure",
                ),
            }
        "#]],
    );
}

#[test]
fn fail_shortcut_expr() {
    check_expression(
        r#"{ fail "Got Here!"; fail "Shouldn't get here..."; }"#,
        &expect![[r#"
            Error {
                span: Span {
                    lo: 2,
                    hi: 18,
                },
                kind: UserFail(
                    "Got Here!",
                ),
            }
        "#]],
    );
}

#[test]
fn array_index_expr() {
    check_expression("[1, 2, 3][1]", &expect!["2"]);
}

#[test]
fn array_slice_expr() {
    check_expression("[1, 2, 3, 4, 5][0..2]", &expect!["[1, 2, 3]"]);
}

#[test]
fn array_slice_step_expr() {
    check_expression("[1, 2, 3, 4, 5][0..2..2]", &expect!["[1, 3]"]);
}

#[test]
fn array_slice_start_expr() {
    check_expression("[1, 2, 3, 4, 5][2...]", &expect!["[3, 4, 5]"]);
}

#[test]
fn array_slice_end_expr() {
    check_expression("[1, 2, 3, 4, 5][...2]", &expect!["[1, 2, 3]"]);
}

// Cannot test until negation is supported.
// #[test]
#[allow(dead_code)]
fn array_slice_reverse_expr() {
    check_expression("[1, 2, 3, 4, 5][2..-1..0]", &expect!["[3, 2, 1]"]);
}

#[test]
fn array_slice_out_of_range_expr() {
    check_expression(
        "[1, 2, 3, 4, 5][0..7]",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 16,
                    hi: 20,
                },
                kind: OutOfRange(
                    5,
                ),
            }
        "#]],
    );
}

#[test]
fn array_index_out_of_range_expr() {
    check_expression(
        "[1, 2, 3][4]",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 10,
                    hi: 11,
                },
                kind: OutOfRange(
                    4,
                ),
            }
        "#]],
    );
}

#[test]
fn array_index_type_error_expr() {
    check_expression(
        "[1, 2, 3][false]",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 10,
                    hi: 15,
                },
                kind: IndexSyntax,
            }
        "#]],
    );
}

#[test]
fn literal_big_int_expr() {
    check_expression(
        "9_223_372_036_854_775_808L",
        &expect!["9223372036854775808"],
    );
}

#[test]
fn literal_bool_false_expr() {
    check_expression("false", &expect!["false"]);
}

#[test]
fn literal_bool_true_expr() {
    check_expression("true", &expect!["true"]);
}

#[test]
fn literal_double_expr() {
    check_expression("4.2", &expect!["4.2"]);
}

#[test]
fn literal_double_trailing_dot_expr() {
    check_expression("4.", &expect!["4.0"]);
}

#[test]
fn literal_int_expr() {
    check_expression("42", &expect!["42"]);
}

#[test]
fn literal_int_too_big_expr() {
    check_expression(
        "9_223_372_036_854_775_808",
        &expect!["-9223372036854775808"],
    );
}

#[test]
fn literal_pauli_i_expr() {
    check_expression("PauliI", &expect!["PauliI"]);
}

#[test]
fn literal_pauli_x_expr() {
    check_expression("PauliX", &expect!["PauliX"]);
}

#[test]
fn literal_pauli_y_expr() {
    check_expression("PauliY", &expect!["PauliY"]);
}

#[test]
fn literal_pauli_z_expr() {
    check_expression("PauliZ", &expect!["PauliZ"]);
}

#[test]
fn literal_result_one_expr() {
    check_expression("One", &expect!["One"]);
}

#[test]
fn literal_result_zero_expr() {
    check_expression("Zero", &expect!["Zero"]);
}

#[test]
fn literal_string_expr() {
    check_expression(r#""foo""#, &expect!["foo"]);
}

#[test]
fn paren_expr() {
    check_expression("(42)", &expect!["42"]);
}

#[test]
fn range_all_expr() {
    check_expression("...", &expect!["..."]);
}

#[test]
fn range_end_expr() {
    check_expression("...3", &expect!["...3"]);
}

#[test]
fn range_step_end_expr() {
    check_expression("...2..3", &expect!["...2..3"]);
}

#[test]
fn range_start_expr() {
    check_expression("1...", &expect!["1..."]);
}

#[test]
fn range_start_end_expr() {
    check_expression("1..3", &expect!["1..3"]);
}

#[test]
fn range_start_step_expr() {
    check_expression("1..2...", &expect!["1..2..."]);
}

#[test]
fn range_start_step_end_expr() {
    check_expression("1..2..3", &expect!["1..2..3"]);
}

#[test]
fn return_expr() {
    check_expression("return 4", &expect!["4"]);
}

#[test]
fn return_shortcut_expr() {
    check_expression(
        r#"{return 4; fail "Shouldn't get here...";}"#,
        &expect!["4"],
    );
}

#[test]
fn tuple_expr() {
    check_expression("(1, 2, 3)", &expect!["(1, 2, 3)"]);
}

#[test]
fn if_true_expr() {
    check_expression(r#"if true {return "Got Here!";}"#, &expect!["Got Here!"]);
}

#[test]
fn if_false_expr() {
    check_expression(
        r#"if false {return "Shouldn't get here...";}"#,
        &expect!["()"],
    );
}

#[test]
fn if_type_error_expr() {
    check_expression(
        "if 4 { 3 }",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 3,
                    hi: 4,
                },
                kind: Type(
                    "Bool",
                    "Int",
                ),
            }
        "#]],
    );
}

#[test]
fn if_else_true_expr() {
    check_expression(
        r#"if true {return "Got Here!";} else {return "Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_else_false_expr() {
    check_expression(
        r#"if false {return "Shouldn't get here...";} else {return "Got Here!"}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_true_true_expr() {
    check_expression(
        r#"if true {return "Got Here!";} elif true {return"Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_false_true_expr() {
    check_expression(
        r#"if false {return "Shouldn't get here...";} elif true {return "Got Here!"}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_false_false_expr() {
    check_expression(
        r#"if false {return "Shouldn't get here...";} elif false {return "Shouldn't get here..."}"#,
        &expect!["()"],
    );
}

#[test]
fn if_elif_else_true_true_expr() {
    check_expression(
        r#"if true {return "Got Here!";} elif true {return "Shouldn't get here..."} else {return "Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_else_false_true_expr() {
    check_expression(
        r#"if false {return "Shouldn't get here...";} elif true {return "Got Here!"} else {return "Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_else_false_false_expr() {
    check_expression(
        r#"if false {return "Shouldn't get here...";} elif false {return "Shouldn't get here..."} else {return "Got Here!"}"#,
        &expect!["Got Here!"],
    );
}
