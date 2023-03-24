// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{compile, PackageStore};
use qsc_passes::globals::extract_callables;

use crate::Evaluator;

fn check_expr(file: &str, expr: &str, expect: &Expect) {
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
        .expect("compile unit should be in package store");
    let globals = extract_callables(&store);
    let evaluator = Evaluator::from_store(&store, id, &globals);
    let expr = unit
        .package
        .entry
        .as_ref()
        .expect("entry expression should be present");
    match evaluator.eval_expr(expr) {
        Ok((result, _)) => expect.assert_eq(&result.to_string()),
        Err(e) => expect.assert_debug_eq(&e),
    }
}

#[test]
fn array_expr() {
    check_expr("", "[1, 2, 3]", &expect!["[1, 2, 3]"]);
}

#[test]
fn array_repeat_expr() {
    check_expr("", "[4, size = 3]", &expect!["[4, 4, 4]"]);
}

#[test]
fn array_repeat_type_error_expr() {
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
fn binop_add_array() {
    check_expr("", "[1, 2] + [3, 4]", &expect!["[1, 2, 3, 4]"]);
}

#[test]
fn binop_add_bigint() {
    check_expr(
        "",
        "2L + 9_223_372_036_854_775_808L",
        &expect!["9223372036854775810"],
    );
}

#[test]
fn binop_add_double() {
    check_expr("", "2.8 + 5.4", &expect!["8.2"]);
}

#[test]
fn binop_add_int() {
    check_expr("", "28 + 54", &expect!["82"]);
}

#[test]
fn binop_add_string() {
    check_expr("", r#""Hello," + " World!""#, &expect!["Hello, World!"]);
}

#[test]
fn binop_add_invalid() {
    check_expr(
        "",
        "(1, 3) + 5.4",
        &expect![[r#"
        Type(
            "Array, BigInt, Double, Int, or String",
            "Tuple",
            Span {
                lo: 0,
                hi: 6,
            },
        )
    "#]],
    );
}

#[test]
fn binop_add_mismatch() {
    check_expr(
        "",
        "1 + 5.4",
        &expect![[r#"
        Type(
            "Int",
            "Double",
            Span {
                lo: 4,
                hi: 7,
            },
        )
    "#]],
    );
}

#[test]
fn binop_andb_bigint() {
    check_expr("", "28L &&& 54L", &expect!["20"]);
}

#[test]
fn binop_andb_int() {
    check_expr("", "28 &&& 54", &expect!["20"]);
}

#[test]
fn binop_andb_invalid() {
    check_expr(
        "",
        "2.8 &&& 5.4",
        &expect![[r#"
        Type(
            "BigInt or Int",
            "Double",
            Span {
                lo: 0,
                hi: 3,
            },
        )
    "#]],
    );
}

#[test]
fn binop_andb_mismatch() {
    check_expr(
        "",
        "28 &&& 54L",
        &expect![[r#"
            Type(
                "Int",
                "BigInt",
                Span {
                    lo: 7,
                    hi: 10,
                },
            )
        "#]],
    );
}

#[test]
fn binop_andl() {
    check_expr("", "true and true", &expect!["true"]);
}

#[test]
fn binop_andl_false() {
    check_expr("", "true and false", &expect!["false"]);
}

#[test]
fn binop_andl_no_shortcut() {
    check_expr(
        "",
        r#"true and (fail "Should Fail")"#,
        &expect![[r#"
        UserFail(
            "Should Fail",
            Span {
                lo: 10,
                hi: 28,
            },
        )
    "#]],
    );
}

#[test]
fn binop_div_bigint() {
    check_expr("", "12L / 3L", &expect!["4"]);
}

#[test]
fn binop_div_int() {
    check_expr("", "12 / 3", &expect!["4"]);
}

#[test]
fn binop_div_double() {
    check_expr("", "(1.2) / (0.3)", &expect!["4.0"]);
}

#[test]
fn binop_eq_double() {
    check_expr("", "(1.2) / (0.3)", &expect!["4.0"]);
}

#[test]
fn binop_equal_array() {
    check_expr("", "[1, 2, 3] == [1, 2, 3]", &expect!["true"]);
}

#[test]
fn binop_equal_array_false_content() {
    check_expr("", "[1, 2, 3] == [1, 0, 3]", &expect!["false"]);
}

#[test]
fn binop_equal_array_false_length() {
    check_expr("", "[1, 2, 3] == [1, 2, 3, 4]", &expect!["false"]);
}

#[test]
fn binop_equal_bigint() {
    check_expr("", "18L == 18L", &expect!["true"]);
}

#[test]
fn binop_equal_bigint_false() {
    check_expr("", "18L == 8L", &expect!["false"]);
}

#[test]
fn binop_equal_type() {
    check_expr(
        "",
        "18L == 18",
        &expect![[r#"
            Type(
                "BigInt",
                "Int",
                Span {
                    lo: 7,
                    hi: 9,
                },
            )
        "#]],
    );
}

#[test]
fn binop_equal_bool() {
    check_expr("", "false == false", &expect!["true"]);
}

#[test]
fn binop_equal_bool_false() {
    check_expr("", "false == true", &expect!["false"]);
}

#[test]
fn binop_equal_double() {
    check_expr("", "1.254 == 1.254", &expect!["true"]);
}

#[test]
fn binop_equal_double_false() {
    check_expr("", "1.254 == 1.25", &expect!["false"]);
}

#[test]
fn binop_equal_int() {
    check_expr("", "42 == 42", &expect!["true"]);
}

#[test]
fn binop_equal_int_false() {
    check_expr("", "42 == 43", &expect!["false"]);
}

#[test]
fn binop_equal_pauli() {
    check_expr("", "PauliX == PauliX", &expect!["true"]);
}

#[test]
fn binop_equal_pauli_false() {
    check_expr("", "PauliX == PauliZ", &expect!["false"]);
}

#[test]
fn binop_equal_range() {
    check_expr("", "(0..4) == (0..4)", &expect!["true"]);
}

#[test]
fn binop_equal_range_false() {
    check_expr("", "(0..2..4) == (0..4)", &expect!["false"]);
}

#[test]
fn binop_equal_result() {
    check_expr("", "One == One", &expect!["true"]);
}

#[test]
fn binop_equal_result_false() {
    check_expr("", "One == Zero", &expect!["false"]);
}

#[test]
fn binop_equal_string() {
    check_expr("", r#""foo" == "foo""#, &expect!["true"]);
}

#[test]
fn binop_equal_string_false() {
    check_expr("", r#""foo" == "bar""#, &expect!["false"]);
}

#[test]
fn binop_equal_tuple() {
    check_expr("", "(1, 2, 3) == (1, 2, 3)", &expect!["true"]);
}

#[test]
fn binop_equal_tuple_false_content() {
    check_expr("", "(1, 2, 3) == (1, Zero, 3)", &expect!["false"]);
}

#[test]
fn binop_equal_tuple_false_arity() {
    check_expr("", "(1, 2, 3) == (1, 2, 3, 4)", &expect!["false"]);
}

#[test]
fn binop_exp_bigint() {
    check_expr("", "2L^3", &expect!["8"]);
}

#[test]
fn binop_exp_bigint_negative_exp() {
    check_expr(
        "",
        "2L^-3",
        &expect![[r#"
        Negative(
            -3,
            Span {
                lo: 3,
                hi: 5,
            },
        )
    "#]],
    );
}

#[test]
fn binop_exp_double() {
    check_expr("", "2.3^3.1", &expect!["13.22380059125472"]);
}

#[test]
fn binop_exp_double_negative_exp() {
    check_expr("", "2.3^-3.1", &expect!["0.07562122501010253"]);
}

#[test]
fn binop_exp_int() {
    check_expr("", "2^3", &expect!["8"]);
}

#[test]
fn binop_exp_int_negative_exp() {
    check_expr(
        "",
        "2^-3",
        &expect![[r#"
        Negative(
            -3,
            Span {
                lo: 2,
                hi: 4,
            },
        )
    "#]],
    );
}

#[test]
fn binop_gt_bigint() {
    check_expr("", "23L > 3L", &expect!["true"]);
}

#[test]
fn binop_gt_bigint_false() {
    check_expr("", "2L > 3L", &expect!["false"]);
}

#[test]
fn binop_gt_int() {
    check_expr("", "23 > 3", &expect!["true"]);
}

#[test]
fn binop_gt_int_false() {
    check_expr("", "2 > 3", &expect!["false"]);
}

#[test]
fn binop_gt_double() {
    check_expr("", "2.3 > 0.3", &expect!["true"]);
}

#[test]
fn binop_gt_double_false() {
    check_expr("", "0.2 > 0.3", &expect!["false"]);
}

#[test]
fn binop_gte_bigint() {
    check_expr("", "23L >= 3L", &expect!["true"]);
}

#[test]
fn binop_gte_bigint_false() {
    check_expr("", "2L >= 3L", &expect!["false"]);
}

#[test]
fn binop_gte_bigint_eq() {
    check_expr("", "3L >= 3L", &expect!["true"]);
}

#[test]
fn binop_gte_int() {
    check_expr("", "23 >= 3", &expect!["true"]);
}

#[test]
fn binop_gte_int_false() {
    check_expr("", "2 >= 3", &expect!["false"]);
}

#[test]
fn binop_gte_int_eq() {
    check_expr("", "3 >= 3", &expect!["true"]);
}

#[test]
fn binop_gte_double() {
    check_expr("", "2.3 >= 0.3", &expect!["true"]);
}

#[test]
fn binop_gte_double_false() {
    check_expr("", "0.2 >= 0.3", &expect!["false"]);
}

#[test]
fn binop_gte_double_eq() {
    check_expr("", "0.3 >= 0.3", &expect!["true"]);
}

#[test]
fn binop_lt_bigint_false() {
    check_expr("", "23L < 3L", &expect!["false"]);
}

#[test]
fn binop_lt_bigint() {
    check_expr("", "2L < 3L", &expect!["true"]);
}

#[test]
fn binop_lt_int_false() {
    check_expr("", "23 < 3", &expect!["false"]);
}

#[test]
fn binop_lt_int() {
    check_expr("", "2 < 3", &expect!["true"]);
}

#[test]
fn binop_lt_double_false() {
    check_expr("", "2.3 < 0.3", &expect!["false"]);
}

#[test]
fn binop_lt_double() {
    check_expr("", "0.2 < 0.3", &expect!["true"]);
}

#[test]
fn binop_lte_bigint_false() {
    check_expr("", "23L <= 3L", &expect!["false"]);
}

#[test]
fn binop_lte_bigint() {
    check_expr("", "2L <= 3L", &expect!["true"]);
}

#[test]
fn binop_lte_bigint_eq() {
    check_expr("", "3L <= 3L", &expect!["true"]);
}

#[test]
fn binop_lte_int_false() {
    check_expr("", "23 <= 3", &expect!["false"]);
}

#[test]
fn binop_lte_int() {
    check_expr("", "2 <= 3", &expect!["true"]);
}

#[test]
fn binop_lte_int_eq() {
    check_expr("", "3 <= 3", &expect!["true"]);
}

#[test]
fn binop_lte_double_false() {
    check_expr("", "2.3 <= 0.3", &expect!["false"]);
}

#[test]
fn binop_lte_double() {
    check_expr("", "0.2 <= 0.3", &expect!["true"]);
}

#[test]
fn binop_lte_double_eq() {
    check_expr("", "0.3 <= 0.3", &expect!["true"]);
}

#[test]
fn binop_mod_bigint() {
    check_expr("", "8L % 6L", &expect!["2"]);
}

#[test]
fn binop_mod_int() {
    check_expr("", "8 % 6", &expect!["2"]);
}

#[test]
fn binop_mod_double() {
    check_expr("", "8.411 % 6.833", &expect!["1.5779999999999994"]);
}

#[test]
fn binop_mul_bigint() {
    check_expr("", "8L * 6L", &expect!["48"]);
}

#[test]
fn binop_mul_int() {
    check_expr("", "8 * 6", &expect!["48"]);
}

#[test]
fn binop_mul_double() {
    check_expr("", "8.411 * 6.833", &expect!["57.472363"]);
}

#[test]
fn binop_neq_array() {
    check_expr("", "[1, 2, 3] != [1, 2, 3]", &expect!["false"]);
}

#[test]
fn binop_neq_array_true_content() {
    check_expr("", "[1, 2, 3] != [1, 0, 3]", &expect!["true"]);
}

#[test]
fn binop_neq_array_true_length() {
    check_expr("", "[1, 2, 3] != [1, 2, 3, 4]", &expect!["true"]);
}

#[test]
fn binop_neq_bigint() {
    check_expr("", "18L != 18L", &expect!["false"]);
}

#[test]
fn binop_neq_bigint_true() {
    check_expr("", "18L != 8L", &expect!["true"]);
}

#[test]
fn binop_neq_type() {
    check_expr(
        "",
        "18L != 18",
        &expect![[r#"
            Type(
                "BigInt",
                "Int",
                Span {
                    lo: 7,
                    hi: 9,
                },
            )
        "#]],
    );
}

#[test]
fn binop_neq_bool() {
    check_expr("", "false != false", &expect!["false"]);
}

#[test]
fn binop_neq_bool_true() {
    check_expr("", "false != true", &expect!["true"]);
}

#[test]
fn binop_neq_double() {
    check_expr("", "1.254 != 1.254", &expect!["false"]);
}

#[test]
fn binop_neq_double_true() {
    check_expr("", "1.254 != 1.25", &expect!["true"]);
}

#[test]
fn binop_neq_int() {
    check_expr("", "42 != 42", &expect!["false"]);
}

#[test]
fn binop_neq_int_true() {
    check_expr("", "42 != 43", &expect!["true"]);
}

#[test]
fn binop_neq_pauli() {
    check_expr("", "PauliX != PauliX", &expect!["false"]);
}

#[test]
fn binop_neq_pauli_true() {
    check_expr("", "PauliX != PauliZ", &expect!["true"]);
}

#[test]
fn binop_neq_range() {
    check_expr("", "(0..4) != (0..4)", &expect!["false"]);
}

#[test]
fn binop_neq_range_true() {
    check_expr("", "(0..2..4) != (0..4)", &expect!["true"]);
}

#[test]
fn binop_neq_result() {
    check_expr("", "One != One", &expect!["false"]);
}

#[test]
fn binop_neq_result_true() {
    check_expr("", "One != Zero", &expect!["true"]);
}

#[test]
fn binop_neq_string() {
    check_expr("", r#""foo" != "foo""#, &expect!["false"]);
}

#[test]
fn binop_neq_string_true() {
    check_expr("", r#""foo" != "bar""#, &expect!["true"]);
}

#[test]
fn binop_neq_tuple() {
    check_expr("", "(1, 2, 3) != (1, 2, 3)", &expect!["false"]);
}

#[test]
fn binop_neq_tuple_true_content() {
    check_expr("", "(1, 2, 3) != (1, Zero, 3)", &expect!["true"]);
}

#[test]
fn binop_neq_tuple_true_arity() {
    check_expr("", "(1, 2, 3) != (1, 2, 3, 4)", &expect!["true"]);
}

#[test]
fn binop_orb_bigint() {
    check_expr("", "28L ||| 54L", &expect!["62"]);
}

#[test]
fn binop_orb_int() {
    check_expr("", "28 ||| 54", &expect!["62"]);
}

#[test]
fn binop_orb_invalid() {
    check_expr(
        "",
        "2.8 ||| 5.4",
        &expect![[r#"
        Type(
            "BigInt or Int",
            "Double",
            Span {
                lo: 0,
                hi: 3,
            },
        )
    "#]],
    );
}

#[test]
fn binop_orb_mismatch() {
    check_expr(
        "",
        "28 ||| 54L",
        &expect![[r#"
            Type(
                "Int",
                "BigInt",
                Span {
                    lo: 7,
                    hi: 10,
                },
            )
        "#]],
    );
}

#[test]
fn binop_orl() {
    check_expr("", "true or true", &expect!["true"]);
}

#[test]
fn binop_orl_true_lhs() {
    check_expr("", "true or false", &expect!["true"]);
}

#[test]
fn binop_orl_true_rhs() {
    check_expr("", "false or true", &expect!["true"]);
}

#[test]
fn binop_orl_false() {
    check_expr("", "false or false", &expect!["false"]);
}

#[test]
fn binop_orl_shortcut() {
    check_expr("", r#"true or (fail "Shouldn't Fail")"#, &expect!["true"]);
}

#[test]
fn binop_shl_bigint() {
    check_expr("", "4L <<< 2", &expect!["16"]);
}

#[test]
fn binop_shl_bigint_negative() {
    check_expr("", "4L <<< -2", &expect!["1"]);
}

#[test]
fn binop_shl_int() {
    check_expr("", "4 <<< 2", &expect!["16"]);
}

#[test]
fn binop_shl_int_negative() {
    check_expr("", "4 <<< -2", &expect!["1"]);
}

#[test]
fn binop_shr_bigint() {
    check_expr("", "4L >>> 2", &expect!["1"]);
}

#[test]
fn binop_shr_bigint_negative() {
    check_expr("", "4L >>> -2", &expect!["16"]);
}

#[test]
fn binop_shr_int() {
    check_expr("", "4 >>> 2", &expect!["1"]);
}

#[test]
fn binop_shr_int_negative() {
    check_expr("", "4 >>> -2", &expect!["16"]);
}

#[test]
fn binop_sub_bigint() {
    check_expr("", "4L - 2L", &expect!["2"]);
}

#[test]
fn binop_sub_int() {
    check_expr("", "4 - 2", &expect!["2"]);
}

#[test]
fn binop_sub_double() {
    check_expr("", "4.7 - 2.5", &expect!["2.2"]);
}

#[test]
fn binop_xorb_bigint() {
    check_expr("", "28L ^^^ 54L", &expect!["42"]);
}

#[test]
fn binop_xorb_int() {
    check_expr("", "28 ^^^ 54", &expect!["42"]);
}

#[test]
fn binop_xorb_invalid() {
    check_expr(
        "",
        "2.8 ^^^ 5.4",
        &expect![[r#"
        Type(
            "BigInt or Int",
            "Double",
            Span {
                lo: 0,
                hi: 3,
            },
        )
    "#]],
    );
}

#[test]
fn binop_xorb_mismatch() {
    check_expr(
        "",
        "28 ^^^ 54L",
        &expect![[r#"
            Type(
                "Int",
                "BigInt",
                Span {
                    lo: 7,
                    hi: 10,
                },
            )
        "#]],
    );
}

#[test]
fn assignop_add_expr() {
    check_expr(
        "",
        indoc! {"{
            mutable x = 0;
            set x += 1;
            x
        }"},
        &expect!["1"],
    );
}

#[test]
fn assignop_sub_expr() {
    check_expr(
        "",
        indoc! {"{
            mutable x = 0;
            set x -= 1;
            x
        }"},
        &expect!["-1"],
    );
}

#[test]
fn assignop_orl_expr() {
    check_expr(
        "",
        indoc! {"{
            mutable x = false;
            set x or= true;
            x
        }"},
        &expect!["true"],
    );
}

#[test]
fn assignop_mutability_expr() {
    check_expr(
        "",
        indoc! {"{
            let x = false;
            set x or= true;
            x
        }"},
        &expect![[r#"
            Mutability(
                Span {
                    lo: 29,
                    hi: 30,
                },
            )
        "#]],
    );
}

#[test]
fn assignop_invalid_type_expr() {
    check_expr(
        "",
        indoc! {"{
            mutable x = false;
            set x += 1;
            x
        }"},
        &expect![[r#"
            Type(
                "Array, BigInt, Double, Int, or String",
                "Bool",
                Span {
                    lo: 33,
                    hi: 34,
                },
            )
        "#]],
    );
}

#[test]
fn fail_expr() {
    check_expr(
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
    check_expr(
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
    check_expr("", "[1, 2, 3][1]", &expect!["2"]);
}

#[test]
fn array_slice_start_end_expr() {
    check_expr("", "[1, 2, 3, 4, 5][0..2]", &expect!["[1, 2, 3]"]);
}

#[test]
fn array_slice_start_step_end_expr() {
    check_expr("", "[1, 2, 3, 4, 5][0..2..2]", &expect!["[1, 3]"]);
}

#[test]
fn array_slice_start_expr() {
    check_expr("", "[1, 2, 3, 4, 5][2...]", &expect!["[3, 4, 5]"]);
}

#[test]
fn array_slice_end_expr() {
    check_expr("", "[1, 2, 3, 4, 5][...2]", &expect!["[1, 2, 3]"]);
}

#[test]
fn array_slice_step_end_expr() {
    check_expr("", "[1, 2, 3, 4, 5][...2..3]", &expect!["[1, 3]"]);
}

#[test]
fn array_slice_step_expr() {
    check_expr("", "[1, 2, 3, 4, 5][...2...]", &expect!["[1, 3, 5]"]);
}

#[test]
fn array_slice_reverse_expr() {
    check_expr("", "[1, 2, 3, 4, 5][2..-1..0]", &expect!["[3, 2, 1]"]);
}

#[test]
fn array_slice_reverse_end_expr() {
    check_expr("", "[1, 2, 3, 4, 5][...-1..2]", &expect!["[5, 4, 3]"]);
}

#[test]
fn array_slice_reverse_start_expr() {
    check_expr("", "[1, 2, 3, 4, 5][2..-1...]", &expect!["[3, 2, 1]"]);
}

#[test]
fn array_slice_reverse_all_expr() {
    check_expr("", "[1, 2, 3, 4, 5][...-1...]", &expect!["[5, 4, 3, 2, 1]"]);
}

#[test]
fn array_slice_all_expr() {
    check_expr("", "[1, 2, 3, 4, 5][...]", &expect!["[1, 2, 3, 4, 5]"]);
}

#[test]
fn array_slice_none_expr() {
    check_expr("", "[1, 2, 3, 4, 5][1..0]", &expect!["[]"]);
}

#[test]
fn array_slice_reverse_none_expr() {
    check_expr("", "[1, 2, 3, 4, 5][0..-1..1]", &expect!["[]"]);
}

#[test]
fn array_slice_step_zero_expr() {
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
        "",
        "9_223_372_036_854_775_808L",
        &expect!["9223372036854775808"],
    );
}

#[test]
fn literal_bool_false_expr() {
    check_expr("", "false", &expect!["false"]);
}

#[test]
fn literal_bool_true_expr() {
    check_expr("", "true", &expect!["true"]);
}

#[test]
fn literal_double_expr() {
    check_expr("", "4.2", &expect!["4.2"]);
}

#[test]
fn literal_double_trailing_dot_expr() {
    check_expr("", "4.", &expect!["4.0"]);
}

#[test]
fn literal_int_expr() {
    check_expr("", "42", &expect!["42"]);
}

#[test]
fn literal_int_too_big_expr() {
    check_expr(
        "",
        "9_223_372_036_854_775_808",
        &expect!["-9223372036854775808"],
    );
}

#[test]
fn literal_pauli_i_expr() {
    check_expr("", "PauliI", &expect!["PauliI"]);
}

#[test]
fn literal_pauli_x_expr() {
    check_expr("", "PauliX", &expect!["PauliX"]);
}

#[test]
fn literal_pauli_y_expr() {
    check_expr("", "PauliY", &expect!["PauliY"]);
}

#[test]
fn literal_pauli_z_expr() {
    check_expr("", "PauliZ", &expect!["PauliZ"]);
}

#[test]
fn literal_result_one_expr() {
    check_expr("", "One", &expect!["One"]);
}

#[test]
fn literal_result_zero_expr() {
    check_expr("", "Zero", &expect!["Zero"]);
}

#[test]
fn literal_string_expr() {
    check_expr("", r#""foo""#, &expect!["foo"]);
}

#[test]
fn paren_expr() {
    check_expr("", "(42)", &expect!["42"]);
}

#[test]
fn range_all_expr() {
    check_expr("", "...", &expect!["..."]);
}

#[test]
fn range_end_expr() {
    check_expr("", "...3", &expect!["...3"]);
}

#[test]
fn range_step_end_expr() {
    check_expr("", "...2..3", &expect!["...2..3"]);
}

#[test]
fn range_start_expr() {
    check_expr("", "1...", &expect!["1..."]);
}

#[test]
fn range_start_end_expr() {
    check_expr("", "1..3", &expect!["1..3"]);
}

#[test]
fn range_start_step_expr() {
    check_expr("", "1..2...", &expect!["1..2..."]);
}

#[test]
fn range_start_step_end_expr() {
    check_expr("", "1..2..3", &expect!["1..2..3"]);
}

#[test]
fn return_expr() {
    check_expr("", "return 4", &expect!["4"]);
}

#[test]
fn return_shortcut_expr() {
    check_expr(
        "",
        r#"{return 4; fail "Shouldn't get here...";}"#,
        &expect!["4"],
    );
}

#[test]
fn tuple_expr() {
    check_expr("", "(1, 2, 3)", &expect!["(1, 2, 3)"]);
}

#[test]
fn unop_bitwise_not_big_int_expr() {
    check_expr(
        "",
        "~~~(9_223_372_036_854_775_808L)",
        &expect!["-9223372036854775809"],
    );
}

#[test]
fn unop_bitwise_not_bool_expr() {
    check_expr(
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
    check_expr("", "~~~(13)", &expect!["-14"]);
}

#[test]
fn unop_negate_big_int_expr() {
    check_expr(
        "",
        "-(9_223_372_036_854_775_808L)",
        &expect!["-9223372036854775808"],
    );
}

#[test]
fn unop_negate_bool_expr() {
    check_expr(
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
    check_expr("", "-(3.4)", &expect!["-3.4"]);
}

#[test]
fn unop_negate_int_expr() {
    check_expr("", "-(13)", &expect!["-13"]);
}

#[test]
fn unop_negate_int_overflow_expr() {
    check_expr(
        "",
        "-(9_223_372_036_854_775_808)",
        &expect!["-9223372036854775808"],
    );
}

#[test]
fn unop_negate_negative_int_expr() {
    check_expr("", "-(-(13))", &expect!["13"]);
}

#[test]
fn unop_not_bool_expr() {
    check_expr("", "not false", &expect!["true"]);
}

#[test]
fn unop_not_int_expr() {
    check_expr(
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
    check_expr(
        "",
        "+(9_223_372_036_854_775_808L)",
        &expect!["9223372036854775808"],
    );
}

#[test]
fn unop_positive_bool_expr() {
    check_expr(
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
    check_expr("", "+(3.4)", &expect!["3.4"]);
}

#[test]
fn unop_positive_int_expr() {
    check_expr("", "+(13)", &expect!["13"]);
}

#[test]
fn unop_adjoint_functor_expr() {
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
        "",
        r#"if true {return "Got Here!";}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_false_expr() {
    check_expr(
        "",
        r#"if false {return "Shouldn't get here...";}"#,
        &expect!["()"],
    );
}

#[test]
fn if_type_error_expr() {
    check_expr(
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
    check_expr(
        "",
        r#"if true {return "Got Here!";} else {return "Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_else_false_expr() {
    check_expr(
        "",
        r#"if false {return "Shouldn't get here...";} else {return "Got Here!"}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_true_true_expr() {
    check_expr(
        "",
        r#"if true {return "Got Here!";} elif true {return"Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_false_true_expr() {
    check_expr(
        "",
        r#"if false {return "Shouldn't get here...";} elif true {return "Got Here!"}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_false_false_expr() {
    check_expr(
        "",
        r#"if false {return "Shouldn't get here...";} elif false {return "Shouldn't get here..."}"#,
        &expect!["()"],
    );
}

#[test]
fn if_elif_else_true_true_expr() {
    check_expr(
        "",
        r#"if true {return "Got Here!";} elif true {return "Shouldn't get here..."} else {return "Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_else_false_true_expr() {
    check_expr(
        "",
        r#"if false {return "Shouldn't get here...";} elif true {return "Got Here!"} else {return "Shouldn't get here..."}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn if_elif_else_false_false_expr() {
    check_expr(
        "",
        r#"if false {return "Shouldn't get here...";} elif false {return "Shouldn't get here..."} else {return "Got Here!"}"#,
        &expect!["Got Here!"],
    );
}

#[test]
fn call_expr() {
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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
    check_expr(
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

#[test]
fn call_adjoint_self_expr() {
    check_expr(
        indoc! {r#"
            namespace Test {
                operation Foo() : Unit is Adj + Ctl {
                    body (...) {
                        fail "Body Implementation";
                    }
                    adjoint self;
                    controlled (ctls, ...) {
                        fail "Controlled Implementation";
                    }
                }
            }
        "#},
        "Adjoint Test.Foo()",
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
