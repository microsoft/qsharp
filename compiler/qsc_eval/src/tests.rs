// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{compile, PackageStore};

use crate::Evaluator;

fn check_expression(file: &str, expr: &str, expect: &Expect) {
    let mut store = PackageStore::new();
    let unit = compile(&store, [], [file], expr);
    assert!(
        unit.context.errors().is_empty(),
        "Compilation errors: {:?}",
        unit.context.errors()
    );
    let id = store.insert(unit);
    match Evaluator::new(&store, id).run() {
        Ok(result) => expect.assert_eq(&result.to_string()),
        Err(e) => expect.assert_debug_eq(&e),
    }
}

#[test]
fn array_expr() {
    check_expression("", "[1, 2, 3]", &expect!["[1, 2, 3]"]);
}

#[test]
fn array_repeat_expr() {
    check_expression("", "[4, size = 3]", &expect!["[4, 4, 4]"]);
}

#[test]
fn array_repeat_type_error_expr() {
    check_expression(
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
        "",
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
    check_expression("", "[1, 2, 3][1]", &expect!["2"]);
}

#[test]
fn array_slice_start_end_expr() {
    check_expression("", "[1, 2, 3, 4, 5][0..2]", &expect!["[1, 2, 3]"]);
}

#[test]
fn array_slice_start_step_end_expr() {
    check_expression("", "[1, 2, 3, 4, 5][0..2..2]", &expect!["[1, 3]"]);
}

#[test]
fn array_slice_start_expr() {
    check_expression("", "[1, 2, 3, 4, 5][2...]", &expect!["[3, 4, 5]"]);
}

#[test]
fn array_slice_end_expr() {
    check_expression("", "[1, 2, 3, 4, 5][...2]", &expect!["[1, 2, 3]"]);
}

#[test]
fn array_slice_step_end_expr() {
    check_expression("", "[1, 2, 3, 4, 5][...2..3]", &expect!["[1, 3]"]);
}

#[test]
fn array_slice_step_expr() {
    check_expression("", "[1, 2, 3, 4, 5][...2...]", &expect!["[1, 3, 5]"]);
}

#[test]
fn array_slice_reverse_expr() {
    check_expression("", "[1, 2, 3, 4, 5][2..-1..0]", &expect!["[3, 2, 1]"]);
}

#[test]
fn array_slice_reverse_end_expr() {
    check_expression("", "[1, 2, 3, 4, 5][...-1..2]", &expect!["[5, 4, 3]"]);
}

#[test]
fn array_slice_reverse_start_expr() {
    check_expression("", "[1, 2, 3, 4, 5][2..-1...]", &expect!["[3, 2, 1]"]);
}

#[test]
fn array_slice_reverse_all_expr() {
    check_expression("", "[1, 2, 3, 4, 5][...-1...]", &expect!["[5, 4, 3, 2, 1]"]);
}

#[test]
fn array_slice_all_expr() {
    check_expression("", "[1, 2, 3, 4, 5][...]", &expect!["[1, 2, 3, 4, 5]"]);
}

#[test]
fn array_slice_none_expr() {
    check_expression("", "[1, 2, 3, 4, 5][1..0]", &expect!["[]"]);
}

#[test]
fn array_slice_reverse_none_expr() {
    check_expression("", "[1, 2, 3, 4, 5][0..-1..1]", &expect!["[]"]);
}

#[test]
fn array_slice_step_zero_expr() {
    check_expression(
        "",
        "[1, 2, 3, 4, 5][...0...]",
        &expect![[r#"
        Error {
            span: Span {
                lo: 16,
                hi: 23,
            },
            kind: RangeStepZero,
        }
    "#]],
    );
}

#[test]
fn array_slice_out_of_range_expr() {
    check_expression(
        "",
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
fn array_index_negative_expr() {
    check_expression(
        "",
        "[1, 2, 3][-2]",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 10,
                    hi: 12,
                },
                kind: IndexVal(
                    -2,
                ),
            }
        "#]],
    );
}

#[test]
fn array_index_out_of_range_expr() {
    check_expression(
        "",
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
        "",
        "[1, 2, 3][false]",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 10,
                    hi: 15,
                },
                kind: Type(
                    "Int or Range",
                    "Bool",
                ),
            }
        "#]],
    );
}

#[test]
fn literal_big_int_expr() {
    check_expression(
        "",
        "9_223_372_036_854_775_808L",
        &expect!["9223372036854775808"],
    );
}

#[test]
fn literal_bool_false_expr() {
    check_expression("", "false", &expect!["false"]);
}

#[test]
fn literal_bool_true_expr() {
    check_expression("", "true", &expect!["true"]);
}

#[test]
fn literal_double_expr() {
    check_expression("", "4.2", &expect!["4.2"]);
}

#[test]
fn literal_double_trailing_dot_expr() {
    check_expression("", "4.", &expect!["4.0"]);
}

#[test]
fn literal_int_expr() {
    check_expression("", "42", &expect!["42"]);
}

#[test]
fn literal_int_too_big_expr() {
    check_expression(
        "",
        "9_223_372_036_854_775_808",
        &expect!["-9223372036854775808"],
    );
}

#[test]
fn literal_pauli_i_expr() {
    check_expression("", "PauliI", &expect!["PauliI"]);
}

#[test]
fn literal_pauli_x_expr() {
    check_expression("", "PauliX", &expect!["PauliX"]);
}

#[test]
fn literal_pauli_y_expr() {
    check_expression("", "PauliY", &expect!["PauliY"]);
}

#[test]
fn literal_pauli_z_expr() {
    check_expression("", "PauliZ", &expect!["PauliZ"]);
}

#[test]
fn literal_result_one_expr() {
    check_expression("", "One", &expect!["One"]);
}

#[test]
fn literal_result_zero_expr() {
    check_expression("", "Zero", &expect!["Zero"]);
}

#[test]
fn literal_string_expr() {
    check_expression("", r#""foo""#, &expect!["foo"]);
}

#[test]
fn paren_expr() {
    check_expression("", "(42)", &expect!["42"]);
}

#[test]
fn range_all_expr() {
    check_expression("", "...", &expect!["..."]);
}

#[test]
fn range_end_expr() {
    check_expression("", "...3", &expect!["...3"]);
}

#[test]
fn range_step_end_expr() {
    check_expression("", "...2..3", &expect!["...2..3"]);
}

#[test]
fn range_start_expr() {
    check_expression("", "1...", &expect!["1..."]);
}

#[test]
fn range_start_end_expr() {
    check_expression("", "1..3", &expect!["1..3"]);
}

#[test]
fn range_start_step_expr() {
    check_expression("", "1..2...", &expect!["1..2..."]);
}

#[test]
fn range_start_step_end_expr() {
    check_expression("", "1..2..3", &expect!["1..2..3"]);
}

#[test]
fn return_expr() {
    check_expression("", "return 4", &expect!["4"]);
}

#[test]
fn return_shortcut_expr() {
    check_expression(
        "",
        r#"{return 4; fail "Shouldn't get here...";}"#,
        &expect!["4"],
    );
}

#[test]
fn tuple_expr() {
    check_expression("", "(1, 2, 3)", &expect!["(1, 2, 3)"]);
}

#[test]
fn unop_bitwise_not_big_int_expr() {
    check_expression(
        "",
        "~~~(9_223_372_036_854_775_808L)",
        &expect!["-9223372036854775809"],
    );
}

#[test]
fn unop_bitwise_not_bool_expr() {
    check_expression(
        "",
        "~~~(false)",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 3,
                    hi: 10,
                },
                kind: Type(
                    "Int or BigInt",
                    "Bool",
                ),
            }
        "#]],
    );
}

#[test]
fn unop_bitwise_not_int_expr() {
    check_expression("", "~~~(13)", &expect!["-14"]);
}

#[test]
fn unop_negate_big_int_expr() {
    check_expression(
        "",
        "-(9_223_372_036_854_775_808L)",
        &expect!["-9223372036854775808"],
    );
}

#[test]
fn unop_negate_bool_expr() {
    check_expression(
        "",
        "-(false)",
        &expect![[r#"
        Error {
            span: Span {
                lo: 1,
                hi: 8,
            },
            kind: Type(
                "Int, BigInt, or Double",
                "Bool",
            ),
        }
    "#]],
    );
}

#[test]
fn unop_negate_double_expr() {
    check_expression("", "-(3.4)", &expect!["-3.4"]);
}

#[test]
fn unop_negate_int_expr() {
    check_expression("", "-(13)", &expect!["-13"]);
}

#[test]
fn unop_negate_int_overflow_expr() {
    check_expression(
        "",
        "-(9_223_372_036_854_775_808)",
        &expect!["-9223372036854775808"],
    );
}

#[test]
fn unop_negate_negative_int_expr() {
    check_expression("", "-(-(13))", &expect!["13"]);
}

#[test]
fn unop_not_bool_expr() {
    check_expression("", "not false", &expect!["true"]);
}

#[test]
fn unop_not_int_expr() {
    check_expression(
        "",
        "not 0",
        &expect![[r#"
        Error {
            span: Span {
                lo: 4,
                hi: 5,
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
fn unop_positive_big_int_expr() {
    check_expression(
        "",
        "+(9_223_372_036_854_775_808L)",
        &expect!["9223372036854775808"],
    );
}

#[test]
fn unop_positive_bool_expr() {
    check_expression(
        "",
        "+(false)",
        &expect![[r#"
        Error {
            span: Span {
                lo: 1,
                hi: 8,
            },
            kind: Type(
                "Int, BigInt, or Double",
                "Bool",
            ),
        }
    "#]],
    );
}

#[test]
fn unop_positive_double_expr() {
    check_expression("", "+(3.4)", &expect!["3.4"]);
}

#[test]
fn unop_positive_int_expr() {
    check_expression("", "+(13)", &expect!["13"]);
}

#[test]
fn unop_adjoint_functor_expr() {
    check_expression(
        indoc! {"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body intrinsic;
                }
            }
        "},
        "Adjoint Test.Foo",
        &expect!["Adjoint <node 0 in package 5>"],
    );
}

#[test]
fn unop_controlled_functor_expr() {
    check_expression(
        indoc! {"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body intrinsic;
                }
            }
        "},
        "Controlled Test.Foo",
        &expect!["Controlled <node 0 in package 5>"],
    );
}

#[test]
fn unop_adjoint_adjoint_functor_expr() {
    check_expression(
        indoc! {"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body intrinsic;
                }
            }
        "},
        "Adjoint (Adjoint Test.Foo)",
        &expect!["<node 0 in package 5>"],
    );
}

#[test]
fn unop_controlled_adjoint_functor_expr() {
    check_expression(
        indoc! {"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body intrinsic;
                }
            }
        "},
        "Controlled Adjoint Test.Foo",
        &expect!["Controlled Adjoint <node 0 in package 5>"],
    );
}

#[test]
fn unop_adjoint_controlled_functor_expr() {
    check_expression(
        indoc! {"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body intrinsic;
                }
            }
        "},
        "Adjoint Controlled Test.Foo",
        &expect!["Controlled Adjoint <node 0 in package 5>"],
    );
}

#[test]
fn unop_controlled_controlled_functor_expr() {
    check_expression(
        indoc! {"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body intrinsic;
                }
            }
        "},
        "Controlled (Controlled Test.Foo)",
        &expect!["Controlled Controlled <node 0 in package 5>"],
    );
}

#[test]
fn if_true_expr() {
    check_expression(
        "",
        r#"if true {return "Got Here!";}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_false_expr() {
    check_expression(
        "",
        r#"if false {return "Shouldn't get here...";}"#,
        &expect!["()"],
    );
}

#[test]
fn if_type_error_expr() {
    check_expression(
        "",
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
        "",
        r#"if true {return "Got Here!";} else {return "Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_else_false_expr() {
    check_expression(
        "",
        r#"if false {return "Shouldn't get here...";} else {return "Got Here!"}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_true_true_expr() {
    check_expression(
        "",
        r#"if true {return "Got Here!";} elif true {return"Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_false_true_expr() {
    check_expression(
        "",
        r#"if false {return "Shouldn't get here...";} elif true {return "Got Here!"}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_false_false_expr() {
    check_expression(
        "",
        r#"if false {return "Shouldn't get here...";} elif false {return "Shouldn't get here..."}"#,
        &expect!["()"],
    );
}

#[test]
fn if_elif_else_true_true_expr() {
    check_expression(
        "",
        r#"if true {return "Got Here!";} elif true {return "Shouldn't get here..."} else {return "Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_else_false_true_expr() {
    check_expression(
        "",
        r#"if false {return "Shouldn't get here...";} elif true {return "Got Here!"} else {return "Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_else_false_false_expr() {
    check_expression(
        "",
        r#"if false {return "Shouldn't get here...";} elif false {return "Shouldn't get here..."} else {return "Got Here!"}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn call_expr() {
    check_expression(
        indoc! {"
            namespace Test {
                function Answer() : Int {
                    42
                }
            }
        "},
        "Test.Answer()",
        &expect!["42"],
    );
}

#[test]
fn call_return_expr() {
    check_expression(
        indoc! {"
            namespace Test {
                function Answer() : Int {
                    return 42;
                }
            }
        "},
        "Test.Answer()",
        &expect!["42"],
    );
}

#[test]
fn call_args_expr() {
    check_expression(
        indoc! {"
            namespace Test {
                function Echo(val : Int) : Int {
                    return val;
                }
            }
        "},
        "Test.Echo(42)",
        &expect!["42"],
    );
}

#[test]
fn call_multiple_args_expr() {
    check_expression(
        indoc! {"
            namespace Test {
                function Echo(val1 : Int, val2 : Int) : (Int, Int) {
                    return (val1, val2);
                }
            }
        "},
        "Test.Echo(42, 43)",
        &expect!["(42, 43)"],
    );
}

#[test]
fn call_tuple_args_expr() {
    check_expression(
        indoc! {"
            namespace Test {
                function MakeList(val1 : (Int, Int), val2 : Int) : Int[] {
                    let (v1, v2) = val1;
                    return [v1, v2, val2];
                }
            }
        "},
        "Test.MakeList((42, 43), 44)",
        &expect!["[42, 43, 44]"],
    );
}

#[test]
fn call_call_expr() {
    check_expression(
        indoc! {"
            namespace Test {
                function TupleToList(tup : (Int, Int)) : Int[] {
                    let (val, size) = tup;
                    return MakeList(val, size);
                }
                function MakeList(val : Int, size : Int) : Int[] {
                    return [val, size = size];
                }
            }
        "},
        "Test.TupleToList((3, 2))",
        &expect!["[3, 3]"],
    );
}

#[test]
fn call_adjoint_expr() {
    check_expression(
        indoc! {r#"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body (...) {
                        fail "Body Implementation";
                    }
                    adjoint (...) {
                        fail "Adjoint Implementation";
                    }
                    controlled (ctls, ...) {
                        fail "Controlled Implementation";
                    }
                    controlled adjoint (ctls, ...) {
                        fail "Controlled Adjoint Implementation";
                    }
                }
            }
        "#},
        "Adjoint Test.Foo()",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 166,
                    hi: 195,
                },
                kind: UserFail(
                    "Adjoint Implementation",
                ),
            }
        "#]],
    );
}

#[test]
fn call_adjoint_adjoint_expr() {
    check_expression(
        indoc! {r#"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body (...) {
                        fail "Body Implementation";
                    }
                    adjoint (...) {
                        fail "Adjoint Implementation";
                    }
                    controlled (ctls, ...) {
                        fail "Controlled Implementation";
                    }
                    controlled adjoint (ctls, ...) {
                        fail "Controlled Adjoint Implementation";
                    }
                }
            }
        "#},
        "Adjoint Adjoint Test.Foo()",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 92,
                    hi: 118,
                },
                kind: UserFail(
                    "Body Implementation",
                ),
            }
        "#]],
    );
}
