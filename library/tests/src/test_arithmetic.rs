// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{test_expression, test_expression_with_lib};
use qsc::interpret::Value;

// Tests for Microsoft.Quantum.Arithmetic namespace

#[test]
fn check_apply_xor_in_place() {
    test_expression(
        {
            "{
            use a = Qubit[3];
            mutable result = [];
            within {
                Microsoft.Quantum.Arithmetic.ApplyXorInPlace(3, a);
            }
            apply {
                set result = [M(a[0]),M(a[1]),M(a[2])];
            }
            return result;
        }"
        },
        &Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ONE, Value::RESULT_ZERO].into()),
    );
}

#[test]
fn check_apply_xor_in_place_l() {
    test_expression(
        {
            "{
            open Microsoft.Quantum.Arithmetic;
            use q = Qubit[100];
            ApplyXorInPlaceL(953L <<< 50, q);
            let result = MeasureInteger(q[50...]);
            ResetAll(q);
            result
        }"
        },
        &Value::Int(953),
    );
}

#[test]
fn check_measure_integer() {
    test_expression(
        {
            "{
                open Microsoft.Quantum.Arithmetic;
                use q = Qubit[16];
                ApplyXorInPlace(45967, q);
                let result = MeasureInteger(q);
                ResetAll(q);
                return result;
            }"
        },
        &Value::Int(45967),
    );
}

#[test]
fn check_maj() {
    test_expression(
        {
            "{
                open Microsoft.Quantum.Arithmetic;
                use q = Qubit[3];
                mutable r = [];
                for i in 0..7 {
                    ApplyXorInPlace(i, q);
                    MAJ(q[0],q[1],q[2]);
                    set r += [MeasureInteger(q)];
                    ResetAll(q);
                }
                r
            }"
        },
        &Value::Array(
            vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(2),
                Value::Int(7),
                Value::Int(3),
                Value::Int(6),
                Value::Int(5),
                Value::Int(4),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_reflect_about_integer() {
    test_expression(
        {
            "{
                open Microsoft.Quantum.Arithmetic;
                open Microsoft.Quantum.Diagnostics;
                operation ManuallyReflectAboutFive(register : Qubit[]) : Unit is Adj + Ctl {
                    within {
                        X(register[1]);
                    } apply {
                        Controlled Z(register[0..1], register[2]);
                    }
                }
                CheckOperationsAreEqual(3,
                    ReflectAboutInteger(5, _),
                    ManuallyReflectAboutFive
                )
            }"
        },
        &Value::Bool(true),
    );
}

#[test]
fn check_add_i_nc() {
    test_expression(
        {
            "{  // RippleCarryAdderNoCarryTTK case
                use x = Qubit[4];
                use y = Qubit[4];
                open Microsoft.Quantum.Arithmetic;
                ApplyXorInPlace(3, x);
                ApplyXorInPlace(5, y);
                AddI(x,y); // 3+5=8
                let result = [M(y[0]),M(y[1]),M(y[2]),M(y[3])];
                ResetAll(x+y);
                return result;
        }"
        },
        &Value::Array(
            vec![
                Value::RESULT_ZERO,
                Value::RESULT_ZERO,
                Value::RESULT_ZERO,
                Value::RESULT_ONE, // 3+5=8
            ]
            .into(),
        ),
    );
}

#[test]
fn check_add_i_c() {
    test_expression(
        {
            "{  // RippleCarryAdderTTK case
                use x = Qubit[4];
                use y = Qubit[5];
                open Microsoft.Quantum.Arithmetic;
                ApplyXorInPlace(7, x);
                ApplyXorInPlace(11, y);
                AddI(x,y); // 7+11=18
                let result = [M(y[0]),M(y[1]),M(y[2]),M(y[3]),M(y[4])];
                ResetAll(x+y);
                return result;
        }"
        },
        &Value::Array(
            vec![
                Value::RESULT_ZERO,
                Value::RESULT_ONE, // 2
                Value::RESULT_ZERO,
                Value::RESULT_ZERO,
                Value::RESULT_ONE, // 16
            ]
            .into(),
        ), // 10010b = 18
    );
}

#[test]
fn check_add_i_1_1() {
    test_expression(
        {
            "{  // Shortest case
                use x = Qubit[1];
                use y = Qubit[1];
                open Microsoft.Quantum.Arithmetic;
                X(x[0]);
                AddI(x,y);
                let result = M(y[0]);
                ResetAll(x+y);
                return result;
        }"
        },
        &Value::RESULT_ONE,
    );
}

#[test]
fn check_add_i_1_2() {
    test_expression(
        {
            "{  // Shortest unequal length case
                use x = Qubit[1];
                use y = Qubit[2];
                open Microsoft.Quantum.Arithmetic;
                X(x[0]);
                X(y[0]);
                AddI(x,y);
                let result = [M(y[0]),M(y[1])];
                ResetAll(x+y);
                return result;
        }"
        },
        &Value::Array(
            vec![
                Value::RESULT_ZERO,
                Value::RESULT_ONE, // 2
            ]
            .into(),
        ),
    );
}

#[test]
fn check_inc_by_l_simple() {
    test_expression(
        {
            "{ // Simple cases for IncByLE
                open Microsoft.Quantum.Arithmetic;

                use y0 = Qubit[1];
                IncByL(0L,y0);
                let i0 = MeasureInteger(y0);
                ResetAll(y0);

                use y1 = Qubit[1];
                IncByL(1L,y1);
                let i1 = MeasureInteger(y1);
                ResetAll(y1);

                use y2 = Qubit[1];
                X(y2[0]);
                IncByL(0L,y2);
                let i2 = MeasureInteger(y2);
                ResetAll(y2);

                use y3 = Qubit[1];
                X(y3[0]);
                IncByL(1L,y3);
                let i3 = MeasureInteger(y3);
                ResetAll(y3);

                use y4 = Qubit[20];
                ApplyXorInPlace(279, y4);
                IncByL(7895L,y4);
                let i4 = MeasureInteger(y4);
                ResetAll(y4);

                return (i0, i1, i2, i3, i4);
        }"
        },
        &Value::Tuple(
            vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(1),
                Value::Int(0),
                Value::Int(279 + 7_895),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_inc_by_le_simple() {
    test_expression(
        {
            "{   // Simple cases for IncByLE
                open Microsoft.Quantum.Arithmetic;
                use x0 = Qubit[1];
                use y0 = Qubit[1];
                IncByLE(x0,y0);
                let i0 = MeasureInteger(y0);
                ResetAll(x0+y0);

                use x1 = Qubit[1];
                use y1 = Qubit[1];
                X(x1[0]);
                IncByLE(x1,y1);
                let i1 = MeasureInteger(y1);
                ResetAll(x1+y1);

                use x2 = Qubit[1];
                use y2 = Qubit[1];
                X(y2[0]);
                IncByLE(x2,y2);
                let i2 = MeasureInteger(y2);
                ResetAll(x2+y2);

                use x3 = Qubit[1];
                use y3 = Qubit[1];
                X(x3[0]);
                X(y3[0]);
                IncByLE(x3,y3);
                let i3 = MeasureInteger(y3);
                ResetAll(x3+y3);

                use x4 = Qubit[10];
                use y4 = Qubit[10];
                ApplyXorInPlace(279, x4);
                ApplyXorInPlace(383, y4);
                IncByLE(x4,y4);
                let i4 = MeasureInteger(y4);
                ResetAll(x4+y4);

                return (i0, i1, i2, i3, i4);
        }"
        },
        &Value::Tuple(
            vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(1),
                Value::Int(0),
                Value::Int(279 + 383),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_add_le_simple() {
    test_expression(
        {
            "{   // Simple cases for AddLE
                open Microsoft.Quantum.Arithmetic;

                use x0 = Qubit[1];
                use y0 = Qubit[1];
                use z0 = Qubit[1];
                AddLE(x0,y0,z0);
                let i0 = MeasureInteger(z0);
                ResetAll(x0+y0+z0);

                use x1 = Qubit[1];
                use y1 = Qubit[1];
                use z1 = Qubit[1];
                X(x1[0]);
                AddLE(x1,y1,z1);
                let i1 = MeasureInteger(z1);
                ResetAll(x1+y1+z1);

                use x2 = Qubit[1];
                use y2 = Qubit[1];
                use z2 = Qubit[1];
                X(y2[0]);
                AddLE(x2,y2,z2);
                let i2 = MeasureInteger(z2);
                ResetAll(x2+y2+z2);

                use x3 = Qubit[1];
                use y3 = Qubit[1];
                use z3 = Qubit[1];
                X(x3[0]);
                X(y3[0]);
                AddLE(x3,y3,z3);
                let i3 = MeasureInteger(z3);
                ResetAll(x3+y3+z3);

                use x4 = Qubit[10];
                use y4 = Qubit[10];
                use z4 = Qubit[10];
                ApplyXorInPlace(279, x4);
                ApplyXorInPlace(383, y4);
                AddLE(x4,y4,z4);
                let i4 = MeasureInteger(z4);
                ResetAll(x4+y4+z4);

                return (i0, i1, i2, i3, i4);
        }"
        },
        &Value::Tuple(
            vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(1),
                Value::Int(0),
                Value::Int(279 + 383),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_ripple_carry_inc_by_l_branching() {
    test_expression(
        {
            "{  // Branching cases for RippleCarryIncByL
                open Microsoft.Quantum.Arithmetic;

                use y0 = Qubit[10];
                ApplyXorInPlace(172, y0);
                RippleCarryIncByL(128L,y0);
                let i0 = MeasureInteger(y0);
                ResetAll(y0);

                use y1 = Qubit[10];
                ApplyXorInPlace(172, y1);
                RippleCarryIncByL(0L,y1);
                let i1 = MeasureInteger(y1);
                ResetAll(y1);

                return (i0, i1);
            }"
        },
        &Value::Tuple(vec![Value::Int(300), Value::Int(172)].into()),
    );
}

const RIPPLE_CARRY_INC_BY_L_TEST_LIB: &str = include_str!("resources/ripple_carry_inc_by_l.qs");

#[test]
fn check_ripple_carry_inc_by_l_exhaustive_bitwidth_1() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByL(1)",
        RIPPLE_CARRY_INC_BY_L_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_inc_by_l_exhaustive_bitwidth_2() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByL(2)",
        RIPPLE_CARRY_INC_BY_L_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_inc_by_l_exhaustive_bitwidth_3() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByL(3)",
        RIPPLE_CARRY_INC_BY_L_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_inc_by_l_exhaustive_bitwidth_4() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByL(4)",
        RIPPLE_CARRY_INC_BY_L_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_rupple_carry_inc_by_l_ctl_exhaustive_bitwidth_1() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByLCtl(1)",
        RIPPLE_CARRY_INC_BY_L_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_rupple_carry_inc_by_l_ctl_exhaustive_bitwidth_2() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByLCtl(2)",
        RIPPLE_CARRY_INC_BY_L_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_rupple_carry_inc_by_l_ctl_exhaustive_bitwidth_3() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByLCtl(3)",
        RIPPLE_CARRY_INC_BY_L_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_rupple_carry_inc_by_l_ctl_exhaustive_bitwidth_4() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByLCtl(4)",
        RIPPLE_CARRY_INC_BY_L_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_inc_by_le_branching() {
    test_expression(
        {
            "{  // Branching cases for RippleCarryIncByLE
                open Microsoft.Quantum.Arithmetic;

                use x0 = Qubit[1];
                use y0 = Qubit[2];
                ApplyXorInPlace(3, y0);
                RippleCarryIncByLE(x0,y0);
                let i0 = MeasureInteger(y0);
                ResetAll(x0+y0);

                use x1 = Qubit[3];
                use y1 = Qubit[10];
                ApplyXorInPlace(7, x1);
                ApplyXorInPlace(31, y1);
                RippleCarryIncByLE(x1,y1);
                let i1 = MeasureInteger(y1);
                ResetAll(x1+y1);

                use x2 = Qubit[3];
                use y2 = Qubit[4];
                ApplyXorInPlace(7, x2);
                ApplyXorInPlace(7, y2);
                RippleCarryIncByLE(x2,y2);
                let i2 = MeasureInteger(y2);
                ResetAll(x2+y2);

                return (i0, i1, i2);
            }"
        },
        &Value::Tuple(vec![Value::Int(3), Value::Int(38), Value::Int(14)].into()),
    );
}

const RIPPLE_CARRY_INC_BY_LE_TEST_LIB: &str = include_str!("resources/ripple_carry_inc_by_le.qs");

#[test]
fn check_ripple_carry_inc_by_le_exhaustive_bitwidth_1() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByLE(1)",
        RIPPLE_CARRY_INC_BY_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_inc_by_le_exhaustive_bitwidth_2() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByLE(2)",
        RIPPLE_CARRY_INC_BY_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_inc_by_le_exhaustive_bitwidth_3() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByLE(3)",
        RIPPLE_CARRY_INC_BY_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_inc_by_le_exhaustive_bitwidth_4() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByLE(4)",
        RIPPLE_CARRY_INC_BY_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_inc_by_le_ctl_exhaustive_bitwidth_1() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByLECtl(1)",
        RIPPLE_CARRY_INC_BY_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_inc_by_le_ctl_exhaustive_bitwidth_2() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByLECtl(2)",
        RIPPLE_CARRY_INC_BY_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_inc_by_le_ctl_exhaustive_bitwidth_3() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByLECtl(3)",
        RIPPLE_CARRY_INC_BY_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_inc_by_le_ctl_exhaustive_bitwidth_4() {
    test_expression_with_lib(
        "Test.TestRippleCarryIncByLECtl(4)",
        RIPPLE_CARRY_INC_BY_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_add_le_branching() {
    test_expression(
        {
            "{  // Branching cases for RippleCarryAddLE
                open Microsoft.Quantum.Arithmetic;

                use x0 = Qubit[2];
                use y0 = Qubit[2];
                use z0 = Qubit[3];
                ApplyXorInPlace(3, x0);
                ApplyXorInPlace(3, y0);
                RippleCarryAddLE(x0,y0,z0);
                let i0 = MeasureInteger(z0);
                ResetAll(x0+y0+z0);

                use x1 = Qubit[2];
                use y1 = Qubit[2];
                use z1 = Qubit[4];
                ApplyXorInPlace(3, x1);
                ApplyXorInPlace(3, y1);
                RippleCarryAddLE(x1,y1,z1);
                let i1 = MeasureInteger(z1);
                ResetAll(x1+y1+z1);

                use x2 = Qubit[2];
                use y2 = Qubit[2];
                use z2 = Qubit[4];
                ApplyXorInPlace(3, x2);
                ApplyXorInPlace(3, y2);
                X(z2[0]);
                RippleCarryAddLE(x2,y2,z2);
                let i2 = MeasureInteger(z2);
                ResetAll(x2+y2+z2);

                return (i0, i1, i2);
        }"
        },
        &Value::Tuple(vec![Value::Int(6), Value::Int(6), Value::Int(7)].into()),
    );
}

const RIPPLE_CARRY_ADD_LE_TEST_LIB: &str = include_str!("resources/ripple_carry_add_le.qs");

#[test]
fn check_ripple_carry_add_le_exhaustive_bitwidth_1() {
    test_expression_with_lib(
        "Test.TestRippleCarryAddLE(1)",
        RIPPLE_CARRY_ADD_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_add_le_exhaustive_bitwidth_2() {
    test_expression_with_lib(
        "Test.TestRippleCarryAddLE(2)",
        RIPPLE_CARRY_ADD_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_add_le_exhaustive_bitwidth_3() {
    test_expression_with_lib(
        "Test.TestRippleCarryAddLE(3)",
        RIPPLE_CARRY_ADD_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_ripple_carry_add_le_exhaustive_bitwidth_4() {
    test_expression_with_lib(
        "Test.TestRippleCarryAddLE(4)",
        RIPPLE_CARRY_ADD_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_lookahead_add_le_branching() {
    test_expression(
        {
            "{  // Branching cases for LookAheadAddLE
                open Microsoft.Quantum.Arithmetic;

                use x0 = Qubit[2];
                use y0 = Qubit[2];
                use z0 = Qubit[3];
                ApplyXorInPlace(3, x0);
                ApplyXorInPlace(3, y0);
                LookAheadAddLE(x0,y0,z0);
                let i0 = MeasureInteger(z0);
                ResetAll(x0+y0+z0);

                use x1 = Qubit[2];
                use y1 = Qubit[2];
                use z1 = Qubit[4];
                ApplyXorInPlace(3, x1);
                ApplyXorInPlace(3, y1);
                LookAheadAddLE(x1,y1,z1);
                let i1 = MeasureInteger(z1);
                ResetAll(x1+y1+z1);

                use x2 = Qubit[2];
                use y2 = Qubit[2];
                use z2 = Qubit[4];
                ApplyXorInPlace(3, x2);
                ApplyXorInPlace(3, y2);
                X(z2[0]);
                LookAheadAddLE(x2,y2,z2);
                let i2 = MeasureInteger(z2);
                ResetAll(x2+y2+z2);

                return (i0, i1, i2);
        }"
        },
        &Value::Tuple(vec![Value::Int(6), Value::Int(6), Value::Int(7)].into()),
    );
}

const LOOKAHEAD_ADD_LE_TEST_LIB: &str = include_str!("resources/lookahead_add_le.qs");

#[test]
fn check_lookahead_add_le_exhaustive_bitwidth_1() {
    test_expression_with_lib(
        "Test.TestLookAheadAddLE(1)",
        LOOKAHEAD_ADD_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_lookahead_add_le_exhaustive_bitwidth_2() {
    test_expression_with_lib(
        "Test.TestLookAheadAddLE(2)",
        LOOKAHEAD_ADD_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_lookahead_add_le_exhaustive_bitwidth_3() {
    test_expression_with_lib(
        "Test.TestLookAheadAddLE(3)",
        LOOKAHEAD_ADD_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_lookahead_add_le_exhaustive_bitwidth_4() {
    test_expression_with_lib(
        "Test.TestLookAheadAddLE(4)",
        LOOKAHEAD_ADD_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

const FOURIER_INC_BY_LE_TEST_LIB: &str = include_str!("resources/fourier_inc_by_le.qs");

#[test]
fn check_fourier_inc_by_le_exhaustive_bitwidth_1() {
    test_expression_with_lib(
        "Test.TestFourierIncByLE(1)",
        FOURIER_INC_BY_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_fourier_inc_by_le_exhaustive_bitwidth_2() {
    test_expression_with_lib(
        "Test.TestFourierIncByLE(2)",
        FOURIER_INC_BY_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_fourier_inc_by_le_exhaustive_bitwidth_3() {
    test_expression_with_lib(
        "Test.TestFourierIncByLE(3)",
        FOURIER_INC_BY_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}
