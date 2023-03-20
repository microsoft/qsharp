// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{compile, PackageStore};
use qsc_passes::globals::extract_callables;

use crate::{evaluate, Scopes};

fn check_statement(file: &str, expr: &str, expect: &Expect) {
    let mut store = PackageStore::new();
    let unit = compile(&store, [], [file], expr);
    assert!(
        unit.context.errors().is_empty(),
        "Compilation errors: {:?}",
        unit.context.errors()
    );
    let id = store.insert(unit);
    let unit = store
        .get(id)
        .expect("Compile unit should be in package store");
    let globals = extract_callables(&store);
    match evaluate(
        unit.package
            .entry
            .as_ref()
            .expect("Entry statement should be provided."),
        &store,
        &globals,
        unit.context.resolutions(),
        id,
        Scopes::default(),
    ) {
        Ok((result, _)) => expect.assert_eq(&result.to_string()),
        Err(e) => expect.assert_debug_eq(&e),
    }
}

#[test]
fn array_expr() {
    check_statement("", "[1, 2, 3]", &expect!["[1, 2, 3]"]);
}

#[test]
fn array_repeat_expr() {
    check_statement("", "[4, size = 3]", &expect!["[4, 4, 4]"]);
}

#[test]
fn array_repeat_type_error_expr() {
    check_statement(
        "",
        "[4, size = true]",
        &expect![[r#"
            Type(
                "Int",
                "Bool",
                Span {
                    lo: 11,
                    hi: 15,
                },
            )
        "#]],
    );
}

#[test]
fn block_expr() {
    check_statement(
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
    check_statement(
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
    check_statement(
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
    check_statement(
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
    check_statement(
        "",
        indoc! {"{
            let (x, y, z) = (0, 1);
        }"},
        &expect![[r#"
            TupleArity(
                3,
                2,
                Span {
                    lo: 10,
                    hi: 19,
                },
            )
        "#]],
    );
}

#[test]
fn block_mutable_expr() {
    check_statement(
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
    check_statement(
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
    check_statement(
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
    check_statement(
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
    check_statement(
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
    check_statement(
        "",
        indoc! {"{
            mutable (x, y) = (0, 1);
            set (x, y) = (1, 2, 3);
            x
        }"},
        &expect![[r#"
            TupleArity(
                2,
                3,
                Span {
                    lo: 39,
                    hi: 45,
                },
            )
        "#]],
    );
}

#[test]
fn block_mutable_nested_scopes_expr() {
    check_statement(
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
    check_statement(
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
    check_statement(
        "",
        indoc! {"{
            let x = 0;
            set x = 1;
        }"},
        &expect![[r#"
            Mutability(
                Span {
                    lo: 25,
                    hi: 26,
                },
            )
        "#]],
    );
}

#[test]
fn block_qubit_use_expr() {
    check_statement(
        "",
        indoc! {"{
            use q = Qubit();
            q
        }"},
        &expect!["Qubit0"],
    );
}

#[test]
fn block_qubit_use_use_expr() {
    check_statement(
        "",
        indoc! {"{
            use q = Qubit();
            use q1 = Qubit();
            q1
        }"},
        &expect!["Qubit1"],
    );
}

#[test]
fn block_qubit_use_reuse_expr() {
    check_statement(
        "",
        indoc! {"{
            {
                use q = Qubit();
            }
            use q = Qubit();
            q
        }"},
        &expect!["Qubit0"],
    );
}

#[test]
fn block_qubit_use_scope_reuse_expr() {
    check_statement(
        "",
        indoc! {"{
            use q = Qubit() {
            }
            use q = Qubit();
            q
        }"},
        &expect!["Qubit0"],
    );
}

#[test]
fn block_qubit_use_array_expr() {
    check_statement(
        "",
        indoc! {"{
            use q = Qubit[3];
            q
        }"},
        &expect!["[Qubit0, Qubit1, Qubit2]"],
    );
}

#[test]
fn block_qubit_use_array_invalid_count_expr() {
    check_statement(
        "",
        indoc! {"{
            use q = Qubit[-3];
            q
        }"},
        &expect![[r#"
            Count(
                -3,
                Span {
                    lo: 20,
                    hi: 22,
                },
            )
        "#]],
    );
}

#[test]
fn block_qubit_use_array_invalid_type_expr() {
    check_statement(
        "",
        indoc! {"{
            use q = Qubit[false];
            q
        }"},
        &expect![[r#"
            Type(
                "Int",
                "Bool",
                Span {
                    lo: 20,
                    hi: 25,
                },
            )
        "#]],
    );
}

#[test]
fn block_qubit_use_tuple_expr() {
    check_statement(
        "",
        indoc! {"{
            use q = (Qubit[3], Qubit(), Qubit());
            q
        }"},
        &expect!["([Qubit0, Qubit1, Qubit2], Qubit3, Qubit4)"],
    );
}

#[test]
fn block_qubit_use_nested_tuple_expr() {
    check_statement(
        "",
        indoc! {"{
            use q = (Qubit[3], (Qubit(), Qubit()));
            q
        }"},
        &expect!["([Qubit0, Qubit1, Qubit2], (Qubit3, Qubit4))"],
    );
}

#[test]
fn block_qubit_use_tuple_invalid_arity_expr() {
    check_statement(
        "",
        indoc! {"{
            use (q, q1) = (Qubit[3], Qubit(), Qubit());
            q
        }"},
        &expect![[r#"
            TupleArity(
                2,
                3,
                Span {
                    lo: 10,
                    hi: 17,
                },
            )
        "#]],
    );
}

#[test]
fn assign_invalid_expr() {
    check_statement(
        "",
        "set 0 = 1",
        &expect![[r#"
            Unassignable(
                Span {
                    lo: 4,
                    hi: 5,
                },
            )
        "#]],
    );
}

#[test]
fn binop_equal_array() {
    check_statement("", "[1, 2, 3] == [1, 2, 3]", &expect!["true"]);
}

#[test]
fn binop_equal_array_false_content() {
    check_statement("", "[1, 2, 3] == [1, 0, 3]", &expect!["false"]);
}

#[test]
fn binop_equal_array_false_length() {
    check_statement("", "[1, 2, 3] == [1, 2, 3, 4]", &expect!["false"]);
}

#[test]
fn binop_equal_bigint() {
    check_statement("", "18L == 18L", &expect!["true"]);
}

#[test]
fn binop_equal_bigint_false() {
    check_statement("", "18L == 8L", &expect!["false"]);
}

#[test]
fn binop_equal_type() {
    check_statement(
        "",
        "18L == 18",
        &expect![[r#"
        Type(
            "BigInt",
            "Int",
            Span {
                lo: 0,
                hi: 9,
            },
        )
    "#]],
    );
}

#[test]
fn binop_equal_bool() {
    check_statement("", "false == false", &expect!["true"]);
}

#[test]
fn binop_equal_bool_false() {
    check_statement("", "false == true", &expect!["false"]);
}

#[test]
fn binop_equal_double() {
    check_statement("", "1.254 == 1.254", &expect!["true"]);
}

#[test]
fn binop_equal_double_false() {
    check_statement("", "1.254 == 1.25", &expect!["false"]);
}

#[test]
fn binop_equal_int() {
    check_statement("", "42 == 42", &expect!["true"]);
}

#[test]
fn binop_equal_int_false() {
    check_statement("", "42 == 43", &expect!["false"]);
}

#[test]
fn binop_equal_pauli() {
    check_statement("", "PauliX == PauliX", &expect!["true"]);
}

#[test]
fn binop_equal_pauli_false() {
    check_statement("", "PauliX == PauliZ", &expect!["false"]);
}

#[test]
fn binop_equal_range() {
    check_statement("", "(0..4) == (0..4)", &expect!["true"]);
}

#[test]
fn binop_equal_range_false() {
    check_statement("", "(0..2..4) == (0..4)", &expect!["false"]);
}

#[test]
fn binop_equal_result() {
    check_statement("", "One == One", &expect!["true"]);
}

#[test]
fn binop_equal_result_false() {
    check_statement("", "One == Zero", &expect!["false"]);
}

#[test]
fn binop_equal_string() {
    check_statement("", r#""foo" == "foo""#, &expect!["true"]);
}

#[test]
fn binop_equal_string_false() {
    check_statement("", r#""foo" == "bar""#, &expect!["false"]);
}

#[test]
fn binop_equal_tuple() {
    check_statement("", "(1, 2, 3) == (1, 2, 3)", &expect!["true"]);
}

#[test]
fn binop_equal_tuple_false_content() {
    check_statement("", "(1, 2, 3) == (1, Zero, 3)", &expect!["false"]);
}

#[test]
fn binop_equal_tuple_false_arity() {
    check_statement("", "(1, 2, 3) == (1, 2, 3, 4)", &expect!["false"]);
}

#[test]
fn fail_expr() {
    check_statement(
        "",
        r#"fail "This is a failure""#,
        &expect![[r#"
            UserFail(
                "This is a failure",
                Span {
                    lo: 0,
                    hi: 24,
                },
            )
        "#]],
    );
}

#[test]
fn fail_shortcut_expr() {
    check_statement(
        "",
        r#"{ fail "Got Here!"; fail "Shouldn't get here..."; }"#,
        &expect![[r#"
            UserFail(
                "Got Here!",
                Span {
                    lo: 2,
                    hi: 18,
                },
            )
        "#]],
    );
}

#[test]
fn array_index_expr() {
    check_statement("", "[1, 2, 3][1]", &expect!["2"]);
}

#[test]
fn array_slice_start_end_expr() {
    check_statement("", "[1, 2, 3, 4, 5][0..2]", &expect!["[1, 2, 3]"]);
}

#[test]
fn array_slice_start_step_end_expr() {
    check_statement("", "[1, 2, 3, 4, 5][0..2..2]", &expect!["[1, 3]"]);
}

#[test]
fn array_slice_start_expr() {
    check_statement("", "[1, 2, 3, 4, 5][2...]", &expect!["[3, 4, 5]"]);
}

#[test]
fn array_slice_end_expr() {
    check_statement("", "[1, 2, 3, 4, 5][...2]", &expect!["[1, 2, 3]"]);
}

#[test]
fn array_slice_step_end_expr() {
    check_statement("", "[1, 2, 3, 4, 5][...2..3]", &expect!["[1, 3]"]);
}

#[test]
fn array_slice_step_expr() {
    check_statement("", "[1, 2, 3, 4, 5][...2...]", &expect!["[1, 3, 5]"]);
}

#[test]
fn array_slice_reverse_expr() {
    check_statement("", "[1, 2, 3, 4, 5][2..-1..0]", &expect!["[3, 2, 1]"]);
}

#[test]
fn array_slice_reverse_end_expr() {
    check_statement("", "[1, 2, 3, 4, 5][...-1..2]", &expect!["[5, 4, 3]"]);
}

#[test]
fn array_slice_reverse_start_expr() {
    check_statement("", "[1, 2, 3, 4, 5][2..-1...]", &expect!["[3, 2, 1]"]);
}

#[test]
fn array_slice_reverse_all_expr() {
    check_statement("", "[1, 2, 3, 4, 5][...-1...]", &expect!["[5, 4, 3, 2, 1]"]);
}

#[test]
fn array_slice_all_expr() {
    check_statement("", "[1, 2, 3, 4, 5][...]", &expect!["[1, 2, 3, 4, 5]"]);
}

#[test]
fn array_slice_none_expr() {
    check_statement("", "[1, 2, 3, 4, 5][1..0]", &expect!["[]"]);
}

#[test]
fn array_slice_reverse_none_expr() {
    check_statement("", "[1, 2, 3, 4, 5][0..-1..1]", &expect!["[]"]);
}

#[test]
fn array_slice_step_zero_expr() {
    check_statement(
        "",
        "[1, 2, 3, 4, 5][...0...]",
        &expect![[r#"
            RangeStepZero(
                Span {
                    lo: 16,
                    hi: 23,
                },
            )
        "#]],
    );
}

#[test]
fn array_slice_out_of_range_expr() {
    check_statement(
        "",
        "[1, 2, 3, 4, 5][0..7]",
        &expect![[r#"
            OutOfRange(
                5,
                Span {
                    lo: 16,
                    hi: 20,
                },
            )
        "#]],
    );
}

#[test]
fn array_index_negative_expr() {
    check_statement(
        "",
        "[1, 2, 3][-2]",
        &expect![[r#"
            IndexVal(
                -2,
                Span {
                    lo: 10,
                    hi: 12,
                },
            )
        "#]],
    );
}

#[test]
fn array_index_out_of_range_expr() {
    check_statement(
        "",
        "[1, 2, 3][4]",
        &expect![[r#"
            OutOfRange(
                4,
                Span {
                    lo: 10,
                    hi: 11,
                },
            )
        "#]],
    );
}

#[test]
fn array_index_type_error_expr() {
    check_statement(
        "",
        "[1, 2, 3][false]",
        &expect![[r#"
            Type(
                "Int or Range",
                "Bool",
                Span {
                    lo: 10,
                    hi: 15,
                },
            )
        "#]],
    );
}

#[test]
fn literal_big_int_expr() {
    check_statement(
        "",
        "9_223_372_036_854_775_808L",
        &expect!["9223372036854775808"],
    );
}

#[test]
fn literal_bool_false_expr() {
    check_statement("", "false", &expect!["false"]);
}

#[test]
fn literal_bool_true_expr() {
    check_statement("", "true", &expect!["true"]);
}

#[test]
fn literal_double_expr() {
    check_statement("", "4.2", &expect!["4.2"]);
}

#[test]
fn literal_double_trailing_dot_expr() {
    check_statement("", "4.", &expect!["4.0"]);
}

#[test]
fn literal_int_expr() {
    check_statement("", "42", &expect!["42"]);
}

#[test]
fn literal_int_too_big_expr() {
    check_statement(
        "",
        "9_223_372_036_854_775_808",
        &expect!["-9223372036854775808"],
    );
}

#[test]
fn literal_pauli_i_expr() {
    check_statement("", "PauliI", &expect!["PauliI"]);
}

#[test]
fn literal_pauli_x_expr() {
    check_statement("", "PauliX", &expect!["PauliX"]);
}

#[test]
fn literal_pauli_y_expr() {
    check_statement("", "PauliY", &expect!["PauliY"]);
}

#[test]
fn literal_pauli_z_expr() {
    check_statement("", "PauliZ", &expect!["PauliZ"]);
}

#[test]
fn literal_result_one_expr() {
    check_statement("", "One", &expect!["One"]);
}

#[test]
fn literal_result_zero_expr() {
    check_statement("", "Zero", &expect!["Zero"]);
}

#[test]
fn literal_string_expr() {
    check_statement("", r#""foo""#, &expect!["foo"]);
}

#[test]
fn paren_expr() {
    check_statement("", "(42)", &expect!["42"]);
}

#[test]
fn range_all_expr() {
    check_statement("", "...", &expect!["..."]);
}

#[test]
fn range_end_expr() {
    check_statement("", "...3", &expect!["...3"]);
}

#[test]
fn range_step_end_expr() {
    check_statement("", "...2..3", &expect!["...2..3"]);
}

#[test]
fn range_start_expr() {
    check_statement("", "1...", &expect!["1..."]);
}

#[test]
fn range_start_end_expr() {
    check_statement("", "1..3", &expect!["1..3"]);
}

#[test]
fn range_start_step_expr() {
    check_statement("", "1..2...", &expect!["1..2..."]);
}

#[test]
fn range_start_step_end_expr() {
    check_statement("", "1..2..3", &expect!["1..2..3"]);
}

#[test]
fn return_expr() {
    check_statement("", "return 4", &expect!["4"]);
}

#[test]
fn return_shortcut_expr() {
    check_statement(
        "",
        r#"{return 4; fail "Shouldn't get here...";}"#,
        &expect!["4"],
    );
}

#[test]
fn tuple_expr() {
    check_statement("", "(1, 2, 3)", &expect!["(1, 2, 3)"]);
}

#[test]
fn unop_bitwise_not_big_int_expr() {
    check_statement(
        "",
        "~~~(9_223_372_036_854_775_808L)",
        &expect!["-9223372036854775809"],
    );
}

#[test]
fn unop_bitwise_not_bool_expr() {
    check_statement(
        "",
        "~~~(false)",
        &expect![[r#"
            Type(
                "Int or BigInt",
                "Bool",
                Span {
                    lo: 3,
                    hi: 10,
                },
            )
        "#]],
    );
}

#[test]
fn unop_bitwise_not_int_expr() {
    check_statement("", "~~~(13)", &expect!["-14"]);
}

#[test]
fn unop_negate_big_int_expr() {
    check_statement(
        "",
        "-(9_223_372_036_854_775_808L)",
        &expect!["-9223372036854775808"],
    );
}

#[test]
fn unop_negate_bool_expr() {
    check_statement(
        "",
        "-(false)",
        &expect![[r#"
            Type(
                "Int, BigInt, or Double",
                "Bool",
                Span {
                    lo: 1,
                    hi: 8,
                },
            )
        "#]],
    );
}

#[test]
fn unop_negate_double_expr() {
    check_statement("", "-(3.4)", &expect!["-3.4"]);
}

#[test]
fn unop_negate_int_expr() {
    check_statement("", "-(13)", &expect!["-13"]);
}

#[test]
fn unop_negate_int_overflow_expr() {
    check_statement(
        "",
        "-(9_223_372_036_854_775_808)",
        &expect!["-9223372036854775808"],
    );
}

#[test]
fn unop_negate_negative_int_expr() {
    check_statement("", "-(-(13))", &expect!["13"]);
}

#[test]
fn unop_not_bool_expr() {
    check_statement("", "not false", &expect!["true"]);
}

#[test]
fn unop_not_int_expr() {
    check_statement(
        "",
        "not 0",
        &expect![[r#"
            Type(
                "Bool",
                "Int",
                Span {
                    lo: 4,
                    hi: 5,
                },
            )
        "#]],
    );
}

#[test]
fn unop_positive_big_int_expr() {
    check_statement(
        "",
        "+(9_223_372_036_854_775_808L)",
        &expect!["9223372036854775808"],
    );
}

#[test]
fn unop_positive_bool_expr() {
    check_statement(
        "",
        "+(false)",
        &expect![[r#"
            Type(
                "Int, BigInt, or Double",
                "Bool",
                Span {
                    lo: 1,
                    hi: 8,
                },
            )
        "#]],
    );
}

#[test]
fn unop_positive_double_expr() {
    check_statement("", "+(3.4)", &expect!["3.4"]);
}

#[test]
fn unop_positive_int_expr() {
    check_statement("", "+(13)", &expect!["13"]);
}

#[test]
fn unop_adjoint_functor_expr() {
    check_statement(
        indoc! {"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body intrinsic;
                }
            }
        "},
        "Adjoint Test.Foo",
        &expect!["Adjoint <node 5 in package 0>"],
    );
}

#[test]
fn unop_controlled_functor_expr() {
    check_statement(
        indoc! {"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body intrinsic;
                }
            }
        "},
        "Controlled Test.Foo",
        &expect!["Controlled <node 5 in package 0>"],
    );
}

#[test]
fn unop_adjoint_adjoint_functor_expr() {
    check_statement(
        indoc! {"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body intrinsic;
                }
            }
        "},
        "Adjoint (Adjoint Test.Foo)",
        &expect!["<node 5 in package 0>"],
    );
}

#[test]
fn unop_controlled_adjoint_functor_expr() {
    check_statement(
        indoc! {"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body intrinsic;
                }
            }
        "},
        "Controlled Adjoint Test.Foo",
        &expect!["Controlled Adjoint <node 5 in package 0>"],
    );
}

#[test]
fn unop_adjoint_controlled_functor_expr() {
    check_statement(
        indoc! {"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body intrinsic;
                }
            }
        "},
        "Adjoint Controlled Test.Foo",
        &expect!["Controlled Adjoint <node 5 in package 0>"],
    );
}

#[test]
fn unop_controlled_controlled_functor_expr() {
    check_statement(
        indoc! {"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body intrinsic;
                }
            }
        "},
        "Controlled (Controlled Test.Foo)",
        &expect!["Controlled Controlled <node 5 in package 0>"],
    );
}

#[test]
fn if_true_expr() {
    check_statement(
        "",
        r#"if true {return "Got Here!";}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_false_expr() {
    check_statement(
        "",
        r#"if false {return "Shouldn't get here...";}"#,
        &expect!["()"],
    );
}

#[test]
fn if_type_error_expr() {
    check_statement(
        "",
        "if 4 { 3 }",
        &expect![[r#"
            Type(
                "Bool",
                "Int",
                Span {
                    lo: 3,
                    hi: 4,
                },
            )
        "#]],
    );
}

#[test]
fn if_else_true_expr() {
    check_statement(
        "",
        r#"if true {return "Got Here!";} else {return "Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_else_false_expr() {
    check_statement(
        "",
        r#"if false {return "Shouldn't get here...";} else {return "Got Here!"}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_true_true_expr() {
    check_statement(
        "",
        r#"if true {return "Got Here!";} elif true {return"Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_false_true_expr() {
    check_statement(
        "",
        r#"if false {return "Shouldn't get here...";} elif true {return "Got Here!"}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_false_false_expr() {
    check_statement(
        "",
        r#"if false {return "Shouldn't get here...";} elif false {return "Shouldn't get here..."}"#,
        &expect!["()"],
    );
}

#[test]
fn if_elif_else_true_true_expr() {
    check_statement(
        "",
        r#"if true {return "Got Here!";} elif true {return "Shouldn't get here..."} else {return "Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_else_false_true_expr() {
    check_statement(
        "",
        r#"if false {return "Shouldn't get here...";} elif true {return "Got Here!"} else {return "Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_else_false_false_expr() {
    check_statement(
        "",
        r#"if false {return "Shouldn't get here...";} elif false {return "Shouldn't get here..."} else {return "Got Here!"}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn call_expr() {
    check_statement(
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
    check_statement(
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
    check_statement(
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
    check_statement(
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
    check_statement(
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
    check_statement(
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
    check_statement(
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
            UserFail(
                "Adjoint Implementation",
                Span {
                    lo: 166,
                    hi: 195,
                },
            )
        "#]],
    );
}

#[test]
fn call_adjoint_adjoint_expr() {
    check_statement(
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
            UserFail(
                "Body Implementation",
                Span {
                    lo: 92,
                    hi: 118,
                },
            )
        "#]],
    );
}
