// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::run_stdlib_test;
use indoc::indoc;
use num_bigint::BigInt;
use qsc::interpret::Value;

//
// Canon namespace
//

#[test]
fn check_apply_to_each() {
    run_stdlib_test(
        indoc! {r#"{
            use register = Qubit[3];
            Microsoft.Quantum.Canon.ApplyToEach(X, register);
            let results = Microsoft.Quantum.Measurement.MeasureEachZ(register);
            ResetAll(register);
            results
        }"#},
        &Value::Array(
            vec![
                Value::Result(true),
                Value::Result(true),
                Value::Result(true),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_apply_to_each_a() {
    run_stdlib_test(
        indoc! {r#"{
            use register = Qubit[3];
            Microsoft.Quantum.Canon.ApplyToEach(X, register);
            Adjoint Microsoft.Quantum.Canon.ApplyToEachA(X, register);
            let results = Microsoft.Quantum.Measurement.MeasureEachZ(register);
            ResetAll(register);
            results
        }"#},
        &Value::Array(
            vec![
                Value::Result(false),
                Value::Result(false),
                Value::Result(false),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_apply_to_each_c_applied() {
    run_stdlib_test(
        indoc! {r#"{
            use control = Qubit();
            use register = Qubit[3];
            Controlled Microsoft.Quantum.Canon.ApplyToEachC([control], (X, register));
            let results = Microsoft.Quantum.Measurement.MeasureEachZ(register);
            ResetAll(register);
            Reset(control);
            results
        }"#},
        &Value::Array(
            vec![
                Value::Result(false),
                Value::Result(false),
                Value::Result(false),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_apply_to_each_c_not_applied() {
    run_stdlib_test(
        indoc! {r#"{
            use control = Qubit();
            use register = Qubit[3];
            X(control);
            Controlled Microsoft.Quantum.Canon.ApplyToEachC([control], (X, register));
            let results = Microsoft.Quantum.Measurement.MeasureEachZ(register);
            ResetAll(register);
            Reset(control);
            results
        }"#},
        &Value::Array(
            vec![
                Value::Result(true),
                Value::Result(true),
                Value::Result(true),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_apply_to_each_ca_applied() {
    run_stdlib_test(
        indoc! {r#"{
            use control = Qubit();
            use register = Qubit[3];
            Microsoft.Quantum.Canon.ApplyToEach(X, register);
            Controlled Adjoint Microsoft.Quantum.Canon.ApplyToEachCA([control], (X, register));
            let results = Microsoft.Quantum.Measurement.MeasureEachZ(register);
            ResetAll(register);
            Reset(control);
            results
        }"#},
        &Value::Array(
            vec![
                Value::Result(true),
                Value::Result(true),
                Value::Result(true),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_apply_to_each_ca_not_applied() {
    run_stdlib_test(
        indoc! {r#"{
            use control = Qubit();
            use register = Qubit[3];
            X(control);
            Microsoft.Quantum.Canon.ApplyToEach(X, register);
            Controlled Adjoint Microsoft.Quantum.Canon.ApplyToEachCA([control], (X, register));
            let results = Microsoft.Quantum.Measurement.MeasureEachZ(register);
            ResetAll(register);
            Reset(control);
            results
        }"#},
        &Value::Array(
            vec![
                Value::Result(false),
                Value::Result(false),
                Value::Result(false),
            ]
            .into(),
        ),
    );
}

//
// Sign, Abs, Min, Max, etc.
//

#[test]
fn check_sign_i() {
    run_stdlib_test("Microsoft.Quantum.Math.SignI(0)", &Value::Int(0));
    run_stdlib_test("Microsoft.Quantum.Math.SignI(1000)", &Value::Int(1));
    run_stdlib_test("Microsoft.Quantum.Math.SignI(-1000)", &Value::Int(-1));
}

#[test]
fn check_sign_d() {
    run_stdlib_test("Microsoft.Quantum.Math.SignD(0.0)", &Value::Int(0));
    run_stdlib_test("Microsoft.Quantum.Math.SignD(0.005)", &Value::Int(1));
    run_stdlib_test("Microsoft.Quantum.Math.SignD(-0.005)", &Value::Int(-1));
}

#[test]
fn check_sign_l() {
    run_stdlib_test("Microsoft.Quantum.Math.SignL(0L)", &Value::Int(0));
    run_stdlib_test(
        "Microsoft.Quantum.Math.SignL(9999999999999999999999999999999999999999L)",
        &Value::Int(1),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.SignL(-9999999999999999999999999999999999999999L)",
        &Value::Int(-1),
    );
}

#[test]
fn check_abs_i() {
    run_stdlib_test("Microsoft.Quantum.Math.AbsI(0)", &Value::Int(0));
    run_stdlib_test("Microsoft.Quantum.Math.AbsI(1000)", &Value::Int(1000));
    run_stdlib_test("Microsoft.Quantum.Math.AbsI(-1000)", &Value::Int(1000));
}

#[test]
fn check_abs_d() {
    run_stdlib_test("Microsoft.Quantum.Math.AbsD(0.0)", &Value::Double(0.0));
    run_stdlib_test("Microsoft.Quantum.Math.AbsD(0.005)", &Value::Double(0.005));
    run_stdlib_test("Microsoft.Quantum.Math.AbsD(-0.005)", &Value::Double(0.005));
}

#[test]
fn check_abs_l() {
    run_stdlib_test(
        "Microsoft.Quantum.Math.AbsL(0L)",
        &Value::BigInt(BigInt::from(0)),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.AbsL(9999L)",
        &Value::BigInt(BigInt::from(9999)),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.AbsL(-9999L)",
        &Value::BigInt(BigInt::from(9999)),
    );
}

#[test]
fn check_max_i() {
    run_stdlib_test("Microsoft.Quantum.Math.MaxI(-5,7)", &Value::Int(7));
    run_stdlib_test("Microsoft.Quantum.Math.MaxI(-7,0)", &Value::Int(0));
}

#[test]
fn check_max_d() {
    run_stdlib_test("Microsoft.Quantum.Math.MaxD(-5.0,7.0)", &Value::Double(7.0));
    run_stdlib_test("Microsoft.Quantum.Math.MaxD(-7.0,0.0)", &Value::Double(0.0));
}

#[test]
fn check_max_l() {
    run_stdlib_test(
        "Microsoft.Quantum.Math.MaxL(-5L,7L)",
        &Value::BigInt(BigInt::from(7)),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.MaxL(-7L,0L)",
        &Value::BigInt(BigInt::from(0)),
    );
}

#[test]
fn check_min_i() {
    run_stdlib_test("Microsoft.Quantum.Math.MinI(-5,7)", &Value::Int(-5));
    run_stdlib_test("Microsoft.Quantum.Math.MinI(-7,0)", &Value::Int(-7));
}

#[test]
fn check_min_d() {
    run_stdlib_test(
        "Microsoft.Quantum.Math.MinD(-5.0,7.0)",
        &Value::Double(-5.0),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.MinD(-7.0,0.0)",
        &Value::Double(-7.0),
    );
}

#[test]
fn check_min_l() {
    run_stdlib_test(
        "Microsoft.Quantum.Math.MinL(-5L,7L)",
        &Value::BigInt(BigInt::from(-5)),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.MinL(-7L,0L)",
        &Value::BigInt(BigInt::from(-7)),
    );
}

//
// Trigonometric functions
//

#[test]
fn check_pi() {
    run_stdlib_test(
        "Microsoft.Quantum.Math.PI()",
        &Value::Double(std::f64::consts::PI),
    );
}

#[test]
fn check_e() {
    run_stdlib_test(
        "Microsoft.Quantum.Math.E()",
        &Value::Double(std::f64::consts::E),
    );
}

#[test]
fn check_arccosh() {
    run_stdlib_test("Microsoft.Quantum.Math.ArcCosh(1.0)", &Value::Double(0.0));
}

#[test]
fn check_arcsinh() {
    run_stdlib_test("Microsoft.Quantum.Math.ArcSinh(0.0)", &Value::Double(0.0));
}

#[test]
fn check_arctanh() {
    run_stdlib_test("Microsoft.Quantum.Math.ArcTanh(0.0)", &Value::Double(0.0));
}

//
// Sqrt, Log, exp, etc.
//

#[test]
fn check_log10() {
    run_stdlib_test("Microsoft.Quantum.Math.Log10(1.0)", &Value::Double(0.0));
    run_stdlib_test("Microsoft.Quantum.Math.Log10(10.0)", &Value::Double(1.0));
}

#[test]
fn check_lg() {
    run_stdlib_test("Microsoft.Quantum.Math.Lg(1.0)", &Value::Double(0.0));
    run_stdlib_test("Microsoft.Quantum.Math.Lg(2.0)", &Value::Double(1.0));
}

#[test]
fn check_ceiling() {
    run_stdlib_test("Microsoft.Quantum.Math.Ceiling(3.1)", &Value::Int(4));
    run_stdlib_test("Microsoft.Quantum.Math.Ceiling(-3.7)", &Value::Int(-3));
}

#[test]
fn check_floor() {
    run_stdlib_test("Microsoft.Quantum.Math.Floor(3.7)", &Value::Int(3));
    run_stdlib_test("Microsoft.Quantum.Math.Floor(-3.1)", &Value::Int(-4));
}

#[test]
fn check_round() {
    run_stdlib_test("Microsoft.Quantum.Math.Round(3.1)", &Value::Int(3));
    run_stdlib_test("Microsoft.Quantum.Math.Round(-3.7)", &Value::Int(-4));
}

//
// Modular arithmetic
//

#[test]
fn check_modulus_i() {
    run_stdlib_test("Microsoft.Quantum.Math.ModulusI(20, 3)", &Value::Int(2));
    run_stdlib_test("Microsoft.Quantum.Math.ModulusI(-20, 3)", &Value::Int(1));
}

#[test]
fn check_modulus_l() {
    run_stdlib_test(
        "Microsoft.Quantum.Math.ModulusL(20L, 3L)",
        &Value::BigInt(BigInt::from(2)),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.ModulusL(-20L, 3L)",
        &Value::BigInt(BigInt::from(1)),
    );
}

#[test]
fn check_exp_mod_i() {
    run_stdlib_test("Microsoft.Quantum.Math.ExpModI(1,10,10)", &Value::Int(1));
    run_stdlib_test("Microsoft.Quantum.Math.ExpModI(10,0,10)", &Value::Int(1));
    run_stdlib_test("Microsoft.Quantum.Math.ExpModI(2,10,10)", &Value::Int(4));
}

#[test]
fn check_exp_mod_l() {
    run_stdlib_test(
        "Microsoft.Quantum.Math.ExpModL(1L,10L,10L)",
        &Value::BigInt(BigInt::from(1)),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.ExpModL(10L,0L,10L)",
        &Value::BigInt(BigInt::from(1)),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.ExpModL(2L,10L,10L)",
        &Value::BigInt(BigInt::from(4)),
    );
}

#[test]
fn check_inverse_mod_i() {
    run_stdlib_test("Microsoft.Quantum.Math.InverseModI(2,5)", &Value::Int(3));
    run_stdlib_test("Microsoft.Quantum.Math.InverseModI(3,10)", &Value::Int(7));
    run_stdlib_test("Microsoft.Quantum.Math.InverseModI(-1,5)", &Value::Int(4));
}

#[test]
fn check_inverse_mod_l() {
    run_stdlib_test(
        "Microsoft.Quantum.Math.InverseModL(2L,5L)",
        &Value::BigInt(BigInt::from(3)),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.InverseModL(3L,10L)",
        &Value::BigInt(BigInt::from(7)),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.InverseModL(-1L,5L)",
        &Value::BigInt(BigInt::from(4)),
    );
}

//
// GCD, etc.
//
#[test]
fn check_gcd_i() {
    run_stdlib_test(
        "Microsoft.Quantum.Math.GreatestCommonDivisorI(0,0)",
        &Value::Int(0),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.GreatestCommonDivisorI(2*3*5,2*3*7)",
        &Value::Int(2 * 3),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.GreatestCommonDivisorI(39088169,63245986)",
        &Value::Int(1),
    );
}

#[test]
fn check_gcd_l() {
    run_stdlib_test(
        "Microsoft.Quantum.Math.GreatestCommonDivisorL(0L,0L)",
        &Value::BigInt(BigInt::from(0)),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.GreatestCommonDivisorL(2L*3L*5L,2L*3L*7L)",
        &Value::BigInt(BigInt::from(2 * 3)),
    );
    run_stdlib_test("Microsoft.Quantum.Math.GreatestCommonDivisorL(222232244629420445529739893461909967206666939096499764990979600L,359579325206583560961765665172189099052367214309267232255589801L)", &Value::BigInt(
        BigInt::from(1)));
}

#[test]
fn check_cfc_i() {
    // NOTE: It is not important if the function returns -3/-4 or 3/4,
    // we can ignore this implementation details or update a function
    // to return canonical result.
    run_stdlib_test(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 2)",
        &Value::Tuple(vec![Value::Int(-1), Value::Int(-1)].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 3)",
        &Value::Tuple(vec![Value::Int(2), Value::Int(3)].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 4)",
        &Value::Tuple(vec![Value::Int(-3), Value::Int(-4)].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 7)",
        &Value::Tuple(vec![Value::Int(5), Value::Int(7)].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 25)",
        &Value::Tuple(vec![Value::Int(-18), Value::Int(-25)].into()),
    );
}

#[test]
fn check_fst_snd() {
    run_stdlib_test("Fst(7,6)", &Value::Int(7));
    run_stdlib_test("Snd(7,6)", &Value::Int(6));
}

#[test]
fn check_bitsize_i() {
    run_stdlib_test("Microsoft.Quantum.Math.BitSizeI(0)", &Value::Int(0));
    run_stdlib_test("Microsoft.Quantum.Math.BitSizeI(1)", &Value::Int(1));
    run_stdlib_test("Microsoft.Quantum.Math.BitSizeI(2)", &Value::Int(2));
    run_stdlib_test("Microsoft.Quantum.Math.BitSizeI(3)", &Value::Int(2));
    run_stdlib_test(
        "Microsoft.Quantum.Math.BitSizeI(0x7FFFFFFFFFFFFFFF)",
        &Value::Int(63),
    );
}

//
// Core namespace
//

#[test]
fn check_repeated() {
    run_stdlib_test("Repeated(Zero, 0)", &Value::Array(vec![].into()));
    run_stdlib_test(
        "Repeated(One, 1)",
        &Value::Array(vec![Value::Result(true)].into()),
    );
    run_stdlib_test(
        "Repeated(1, 2)",
        &Value::Array(vec![Value::Int(1), Value::Int(1)].into()),
    );
    run_stdlib_test(
        "Repeated(true, 3)",
        &Value::Array(vec![Value::Bool(true), Value::Bool(true), Value::Bool(true)].into()),
    );
}

//
// Arrays namespace
//

#[test]
fn check_chunks() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Chunks(1, [10, 11, 12, 13, 14, 15])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(10)].into()),
                Value::Array(vec![Value::Int(11)].into()),
                Value::Array(vec![Value::Int(12)].into()),
                Value::Array(vec![Value::Int(13)].into()),
                Value::Array(vec![Value::Int(14)].into()),
                Value::Array(vec![Value::Int(15)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Chunks(2, [])",
        &Value::Array(vec![].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Chunks(2, [10])",
        &Value::Array(vec![Value::Array(vec![Value::Int(10)].into())].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Chunks(2, [10, 11, 12, 13, 14, 15])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(10), Value::Int(11)].into()),
                Value::Array(vec![Value::Int(12), Value::Int(13)].into()),
                Value::Array(vec![Value::Int(14), Value::Int(15)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Chunks(3, [10, 11, 12, 13, 14, 15])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(10), Value::Int(11), Value::Int(12)].into()),
                Value::Array(vec![Value::Int(13), Value::Int(14), Value::Int(15)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Chunks(4, [10, 11, 12, 13, 14, 15])",
        &Value::Array(
            vec![
                Value::Array(
                    vec![
                        Value::Int(10),
                        Value::Int(11),
                        Value::Int(12),
                        Value::Int(13),
                    ]
                    .into(),
                ),
                Value::Array(vec![Value::Int(14), Value::Int(15)].into()),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_diagnonal() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Diagonal([])",
        &Value::Array(vec![].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Diagonal([[1]])",
        &Value::Array(vec![Value::Int(1)].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Diagonal([[1, 2, 3], [4, 5, 6], [7, 8, 9]])",
        &Value::Array(vec![Value::Int(1), Value::Int(5), Value::Int(9)].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Diagonal([[1, 2, 3], [4, 5, 6]])",
        &Value::Array(vec![Value::Int(1), Value::Int(5)].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Diagonal([[1, 2], [3, 4], [5, 6]])",
        &Value::Array(vec![Value::Int(1), Value::Int(4)].into()),
    );
}

#[test]
fn check_element_at() {
    run_stdlib_test("Microsoft.Quantum.Arrays.ElementAt(0, [0])", &Value::Int(0));
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.ElementAt(1, [0, 2, 4])",
        &Value::Int(2),
    );

    run_stdlib_test(
        "Microsoft.Quantum.Arrays.ElementAt(5, [2, 3, 5, 7, 11, 13, 17, 19, 23, 29])",
        &Value::Int(13),
    );

    run_stdlib_test(
        "Microsoft.Quantum.Arrays.ElementAt(9, [0, 1, 1, 2, 3, 5, 8, 13, 21, 34])",
        &Value::Int(34),
    );
}

#[test]
fn check_elements_at() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.ElementsAt(0..0, [0])",
        &Value::Array(vec![Value::Int(0)].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.ElementsAt(0..1, [0, 2, 4])",
        &Value::Array(vec![Value::Int(0), Value::Int(2)].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.ElementsAt(1..2..9, [2, 3, 5, 7, 11, 13, 17, 19, 23, 29])",
        &Value::Array(
            vec![
                Value::Int(3),
                Value::Int(7),
                Value::Int(13),
                Value::Int(19),
                Value::Int(29),
            ]
            .into(),
        ),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.ElementsAt(0..3..9, [0, 1, 1, 2, 3, 5, 8, 13, 21, 34])",
        &Value::Array(vec![Value::Int(0), Value::Int(2), Value::Int(8), Value::Int(34)].into()),
    );
}

#[test]
fn check_excluding() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Excluding([], [])",
        &Value::Array(vec![].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Excluding([], [10, 11, 12, 13, 14, 15])",
        &Value::Array(
            vec![
                Value::Int(10),
                Value::Int(11),
                Value::Int(12),
                Value::Int(13),
                Value::Int(14),
                Value::Int(15),
            ]
            .into(),
        ),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Excluding([1, 3, 4], [10, 11, 12, 13, 14, 15])",
        &Value::Array(vec![Value::Int(10), Value::Int(12), Value::Int(15)].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Excluding([3, 1, 4, 1], [10, 11, 12, 13, 14, 15])",
        &Value::Array(vec![Value::Int(10), Value::Int(12), Value::Int(15)].into()),
    );
}

#[test]
fn check_head() {
    run_stdlib_test("Microsoft.Quantum.Arrays.Head([5,6,7,8])", &Value::Int(5));
}

#[test]
fn check_index_range() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IndexRange([7,6,5,4])::Start",
        &Value::Int(0),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IndexRange([7,6,5,4])::Step",
        &Value::Int(1),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IndexRange([7,6,5,4])::End",
        &Value::Int(3),
    );
}

#[test]
fn check_interleaved() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Interleaved([1, 2, 3], [-1, -2, -3])",
        &Value::Array(
            vec![
                Value::Int(1),
                Value::Int(-1),
                Value::Int(2),
                Value::Int(-2),
                Value::Int(3),
                Value::Int(-3),
            ]
            .into(),
        ),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Interleaved([true, true], [false])",
        &Value::Array(vec![Value::Bool(true), Value::Bool(false), Value::Bool(true)].into()),
    );
}

#[test]
fn check_is_empty() {
    run_stdlib_test("Microsoft.Quantum.Arrays.IsEmpty([])", &Value::Bool(true));
    run_stdlib_test("Microsoft.Quantum.Arrays.IsEmpty([1])", &Value::Bool(false));
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IsEmpty([1, 2, 3, 4, 5])",
        &Value::Bool(false),
    );
}

#[test]
fn check_is_rectangular_array() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IsRectangularArray([])",
        &Value::Bool(true),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IsRectangularArray([[1]])",
        &Value::Bool(true),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IsRectangularArray([[1, 2], [3, 4]])",
        &Value::Bool(true),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IsRectangularArray([[1, 2, 3], [4, 5, 6]])",
        &Value::Bool(true),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IsRectangularArray([[1, 2], [3, 4, 5]])",
        &Value::Bool(false),
    );
}

#[test]
fn check_is_square_array() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IsSquareArray([])",
        &Value::Bool(true),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IsSquareArray([[1]])",
        &Value::Bool(true),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IsSquareArray([[1, 2], [3, 4]])",
        &Value::Bool(true),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IsSquareArray([[1, 2, 3], [4, 5, 6]])",
        &Value::Bool(false),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.IsSquareArray([[1, 2], [3, 4], [5, 6]])",
        &Value::Bool(false),
    );
}

#[test]
fn check_most() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Most([5,6,7,8])",
        &Value::Array(vec![Value::Int(5), Value::Int(6), Value::Int(7)].into()),
    );
}

#[test]
fn check_padded() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Padded(-5, 2, [10, 11, 12])",
        &Value::Array(
            vec![
                Value::Int(10),
                Value::Int(11),
                Value::Int(12),
                Value::Int(2),
                Value::Int(2),
            ]
            .into(),
        ),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Padded(5, 2, [10, 11, 12])",
        &Value::Array(
            vec![
                Value::Int(2),
                Value::Int(2),
                Value::Int(10),
                Value::Int(11),
                Value::Int(12),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_partitioned() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Partitioned([2, 1], [2, 3, 5, 7])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(2), Value::Int(3)].into()),
                Value::Array(vec![Value::Int(5)].into()),
                Value::Array(vec![Value::Int(7)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Partitioned([2, 2], [2, 3, 5, 7])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(2), Value::Int(3)].into()),
                Value::Array(vec![Value::Int(5), Value::Int(7)].into()),
                Value::Array(vec![].into()),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_sequence_i() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.SequenceI(0, 3)",
        &Value::Array(vec![Value::Int(0), Value::Int(1), Value::Int(2), Value::Int(3)].into()),
    );
}

#[test]
fn check_rest() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Rest([5,6,7,8])",
        &Value::Array(vec![Value::Int(6), Value::Int(7), Value::Int(8)].into()),
    );
}

#[test]
fn check_reversed() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Reversed([5,6,7,8])",
        &Value::Array(vec![Value::Int(8), Value::Int(7), Value::Int(6), Value::Int(5)].into()),
    );
}

#[test]
fn check_subarray() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Subarray([3, 0, 2, 1], [1, 2, 3, 4])",
        &Value::Array(vec![Value::Int(4), Value::Int(1), Value::Int(3), Value::Int(2)].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Subarray([1, 2, 2], [1, 2, 3, 4])",
        &Value::Array(vec![Value::Int(2), Value::Int(3), Value::Int(3)].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Subarray([0, 0, 0, 0, 0], [false])",
        &Value::Array(
            vec![
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_tail() {
    run_stdlib_test("Microsoft.Quantum.Arrays.Tail([5,6,7,8])", &Value::Int(8));
}

#[test]
fn check_transposed() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Transposed([[1, 2, 3], [4, 5, 6]])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(1), Value::Int(4)].into()),
                Value::Array(vec![Value::Int(2), Value::Int(5)].into()),
                Value::Array(vec![Value::Int(3), Value::Int(6)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Transposed([[1, 4], [2, 5], [3, 6]])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)].into()),
                Value::Array(vec![Value::Int(4), Value::Int(5), Value::Int(6)].into()),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_unzipped() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Unzipped([])",
        &Value::Tuple(vec![Value::Array(vec![].into()), Value::Array(vec![].into())].into()),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Unzipped([(5, true), (4, false), (3, true), (2, true), (1, false)])",
        &Value::Tuple(vec![
            Value::Array(vec![Value::Int(5), Value::Int(4), Value::Int(3), Value::Int(2), Value::Int(1)].into()),
            Value::Array(vec![Value::Bool(true), Value::Bool(false), Value::Bool(true), Value::Bool(true), Value::Bool(false)].into())].into()),
    );
}

#[test]
fn check_windows() {
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Windows(1, [1, 2, 3, 4, 5])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(1)].into()),
                Value::Array(vec![Value::Int(2)].into()),
                Value::Array(vec![Value::Int(3)].into()),
                Value::Array(vec![Value::Int(4)].into()),
                Value::Array(vec![Value::Int(5)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Windows(3, [1, 2, 3, 4, 5])",
        &Value::Array(
            vec![
                Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)].into()),
                Value::Array(vec![Value::Int(2), Value::Int(3), Value::Int(4)].into()),
                Value::Array(vec![Value::Int(3), Value::Int(4), Value::Int(5)].into()),
            ]
            .into(),
        ),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Arrays.Windows(5, [1, 2, 3, 4, 5])",
        &Value::Array(
            vec![Value::Array(
                vec![
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                    Value::Int(4),
                    Value::Int(5),
                ]
                .into(),
            )]
            .into(),
        ),
    );
}

#[test]
fn check_apply_xor_in_place() {
    run_stdlib_test(
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
        &Value::Array(
            vec![
                Value::Result(true),
                Value::Result(true),
                Value::Result(false),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_apply_cnot_chain_2() {
    run_stdlib_test(
        {
            "{
            use a = Qubit[2];
            mutable result = [];
            within {
                X(a[0]);
                X(a[1]);
                ApplyCNOTChain(a);
            }
            apply {
                set result = [M(a[0]),M(a[1])];
            }
            return result;
        }"
        },
        &Value::Array(vec![Value::Result(true), Value::Result(false)].into()),
    );
}

#[test]
fn check_apply_cnot_chain_3() {
    run_stdlib_test(
        {
            "{
            use a = Qubit[3];
            mutable result = [];
            within {
                X(a[0]);
                ApplyCNOTChain(a);
            }
            apply {
                set result = [M(a[0]),M(a[1]),M(a[2])];
            }
            return result;
        }"
        },
        &Value::Array(
            vec![
                Value::Result(true),
                Value::Result(true),
                Value::Result(true),
            ]
            .into(),
        ),
    );
}

//
// Mesurement namespace
//

#[test]
fn check_measure_each_z() {
    run_stdlib_test(
        indoc! {r#"{
            use register = Qubit[3];
            X(register[0]);
            X(register[2]);
            let results = Microsoft.Quantum.Measurement.MeasureEachZ(register);
            ResetAll(register);
            results
        }"#},
        &Value::Array(
            vec![
                Value::Result(true),
                Value::Result(false),
                Value::Result(true),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_mreset_x() {
    run_stdlib_test(
        indoc! {r#"{
            use register = Qubit[2];
            X(register[1]);
            Microsoft.Quantum.Canon.ApplyToEach(H, register);
            let r0 = Microsoft.Quantum.Measurement.MResetX(register[0]);
            let r1 = Microsoft.Quantum.Measurement.MResetX(register[1]);
            [r0, r1]
        }"#},
        &Value::Array(vec![Value::Result(false), Value::Result(true)].into()),
    );
}

#[test]
fn check_mreset_y() {
    run_stdlib_test(
        indoc! {r#"{
            use register = Qubit[2];
            X(register[1]);
            Microsoft.Quantum.Canon.ApplyToEach(H, register);
            Microsoft.Quantum.Canon.ApplyToEach(S, register);
            let r0 = Microsoft.Quantum.Measurement.MResetY(register[0]);
            let r1 = Microsoft.Quantum.Measurement.MResetY(register[1]);
            [r0, r1]
        }"#},
        &Value::Array(vec![Value::Result(false), Value::Result(true)].into()),
    );
}

#[test]
fn check_mreset_z() {
    run_stdlib_test(
        indoc! {r#"{
            use register = Qubit[2];
            X(register[1]);
            let r0 = Microsoft.Quantum.Measurement.MResetZ(register[0]);
            let r1 = Microsoft.Quantum.Measurement.MResetZ(register[1]);
            [r0, r1]
        }"#},
        &Value::Array(vec![Value::Result(false), Value::Result(true)].into()),
    );
}

#[test]
fn check_apply_cnot_chain_3a() {
    run_stdlib_test(
        {
            "{
            use a = Qubit[3];
            mutable result = [];
            within {
                X(a[0]);
                X(a[2]);
                ApplyCNOTChain(a);
            }
            apply {
                set result = [M(a[0]),M(a[1]),M(a[2])];
            }
            return result;
        }"
        },
        &Value::Array(
            vec![
                Value::Result(true),
                Value::Result(true),
                Value::Result(false),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_add_i_nc() {
    run_stdlib_test(
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
                Value::Result(false),
                Value::Result(false),
                Value::Result(false),
                Value::Result(true), // 3+5=8
            ]
            .into(),
        ),
    );
}

#[test]
fn check_add_i_c() {
    run_stdlib_test(
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
                Value::Result(false),
                Value::Result(true), // 2
                Value::Result(false),
                Value::Result(false),
                Value::Result(true), // 16
            ]
            .into(),
        ), // 10010b = 18
    );
}

#[test]
fn check_add_i_1_1() {
    run_stdlib_test(
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
        &Value::Result(true),
    );
}

#[test]
fn check_add_i_1_2() {
    run_stdlib_test(
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
                Value::Result(false),
                Value::Result(true), // 2
            ]
            .into(),
        ),
    );
}
