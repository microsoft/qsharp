// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::test_expression;
use indoc::indoc;
use num_bigint::BigInt;
use qsc::interpret::Value;

//
// Canon namespace
//

#[test]
fn check_apply_to_each() {
    test_expression(
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
    test_expression(
        indoc! {r#"{
            use register = Qubit[3];
            Microsoft.Quantum.Canon.ApplyToEach(X, register);
            Adjoint Microsoft.Quantum.Canon.ApplyToEachA(X, register);
            let results = Microsoft.Quantum.Measurement.MResetEachZ(register);
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
    test_expression(
        indoc! {r#"{
            use control = Qubit();
            use register = Qubit[3];
            Controlled Microsoft.Quantum.Canon.ApplyToEachC([control], (X, register));
            let results = Microsoft.Quantum.Measurement.MResetEachZ(register);
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
    test_expression(
        indoc! {r#"{
            use control = Qubit();
            use register = Qubit[3];
            X(control);
            Controlled Microsoft.Quantum.Canon.ApplyToEachC([control], (X, register));
            let results = Microsoft.Quantum.Measurement.MResetEachZ(register);
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
    test_expression(
        indoc! {r#"{
            use control = Qubit();
            use register = Qubit[3];
            Microsoft.Quantum.Canon.ApplyToEach(X, register);
            Controlled Adjoint Microsoft.Quantum.Canon.ApplyToEachCA([control], (X, register));
            let results = Microsoft.Quantum.Measurement.MResetEachZ(register);
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
    test_expression(
        indoc! {r#"{
            use control = Qubit();
            use register = Qubit[3];
            X(control);
            Microsoft.Quantum.Canon.ApplyToEach(X, register);
            Controlled Adjoint Microsoft.Quantum.Canon.ApplyToEachCA([control], (X, register));
            let results = Microsoft.Quantum.Measurement.MResetEachZ(register);
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
    test_expression("Microsoft.Quantum.Math.SignI(0)", &Value::Int(0));
    test_expression("Microsoft.Quantum.Math.SignI(1000)", &Value::Int(1));
    test_expression("Microsoft.Quantum.Math.SignI(-1000)", &Value::Int(-1));
}

#[test]
fn check_sign_d() {
    test_expression("Microsoft.Quantum.Math.SignD(0.0)", &Value::Int(0));
    test_expression("Microsoft.Quantum.Math.SignD(0.005)", &Value::Int(1));
    test_expression("Microsoft.Quantum.Math.SignD(-0.005)", &Value::Int(-1));
}

#[test]
fn check_sign_l() {
    test_expression("Microsoft.Quantum.Math.SignL(0L)", &Value::Int(0));
    test_expression(
        "Microsoft.Quantum.Math.SignL(9999999999999999999999999999999999999999L)",
        &Value::Int(1),
    );
    test_expression(
        "Microsoft.Quantum.Math.SignL(-9999999999999999999999999999999999999999L)",
        &Value::Int(-1),
    );
}

#[test]
fn check_abs_i() {
    test_expression("Microsoft.Quantum.Math.AbsI(0)", &Value::Int(0));
    test_expression("Microsoft.Quantum.Math.AbsI(1000)", &Value::Int(1000));
    test_expression("Microsoft.Quantum.Math.AbsI(-1000)", &Value::Int(1000));
}

#[test]
fn check_abs_d() {
    test_expression("Microsoft.Quantum.Math.AbsD(0.0)", &Value::Double(0.0));
    test_expression("Microsoft.Quantum.Math.AbsD(0.005)", &Value::Double(0.005));
    test_expression("Microsoft.Quantum.Math.AbsD(-0.005)", &Value::Double(0.005));
}

#[test]
fn check_abs_l() {
    test_expression(
        "Microsoft.Quantum.Math.AbsL(0L)",
        &Value::BigInt(BigInt::from(0)),
    );
    test_expression(
        "Microsoft.Quantum.Math.AbsL(9999L)",
        &Value::BigInt(BigInt::from(9999)),
    );
    test_expression(
        "Microsoft.Quantum.Math.AbsL(-9999L)",
        &Value::BigInt(BigInt::from(9999)),
    );
}

#[test]
fn check_max_i() {
    test_expression("Microsoft.Quantum.Math.MaxI(-5,7)", &Value::Int(7));
    test_expression("Microsoft.Quantum.Math.MaxI(-7,0)", &Value::Int(0));
}

#[test]
fn check_max_d() {
    test_expression("Microsoft.Quantum.Math.MaxD(-5.0,7.0)", &Value::Double(7.0));
    test_expression("Microsoft.Quantum.Math.MaxD(-7.0,0.0)", &Value::Double(0.0));
}

#[test]
fn check_max_l() {
    test_expression(
        "Microsoft.Quantum.Math.MaxL(-5L,7L)",
        &Value::BigInt(BigInt::from(7)),
    );
    test_expression(
        "Microsoft.Quantum.Math.MaxL(-7L,0L)",
        &Value::BigInt(BigInt::from(0)),
    );
}

#[test]
fn check_min_i() {
    test_expression("Microsoft.Quantum.Math.MinI(-5,7)", &Value::Int(-5));
    test_expression("Microsoft.Quantum.Math.MinI(-7,0)", &Value::Int(-7));
}

#[test]
fn check_min_d() {
    test_expression(
        "Microsoft.Quantum.Math.MinD(-5.0,7.0)",
        &Value::Double(-5.0),
    );
    test_expression(
        "Microsoft.Quantum.Math.MinD(-7.0,0.0)",
        &Value::Double(-7.0),
    );
}

#[test]
fn check_min_l() {
    test_expression(
        "Microsoft.Quantum.Math.MinL(-5L,7L)",
        &Value::BigInt(BigInt::from(-5)),
    );
    test_expression(
        "Microsoft.Quantum.Math.MinL(-7L,0L)",
        &Value::BigInt(BigInt::from(-7)),
    );
}

//
// Trigonometric functions
//

#[test]
fn check_pi() {
    test_expression(
        "Microsoft.Quantum.Math.PI()",
        &Value::Double(std::f64::consts::PI),
    );
}

#[test]
fn check_e() {
    test_expression(
        "Microsoft.Quantum.Math.E()",
        &Value::Double(std::f64::consts::E),
    );
}

#[test]
fn check_arccosh() {
    test_expression("Microsoft.Quantum.Math.ArcCosh(1.0)", &Value::Double(0.0));
}

#[test]
fn check_arcsinh() {
    test_expression("Microsoft.Quantum.Math.ArcSinh(0.0)", &Value::Double(0.0));
}

#[test]
fn check_arctanh() {
    test_expression("Microsoft.Quantum.Math.ArcTanh(0.0)", &Value::Double(0.0));
}

//
// Sqrt, Log, exp, etc.
//

#[test]
fn check_log10() {
    test_expression("Microsoft.Quantum.Math.Log10(1.0)", &Value::Double(0.0));
    test_expression("Microsoft.Quantum.Math.Log10(10.0)", &Value::Double(1.0));
}

#[test]
fn check_lg() {
    test_expression("Microsoft.Quantum.Math.Lg(1.0)", &Value::Double(0.0));
    test_expression("Microsoft.Quantum.Math.Lg(2.0)", &Value::Double(1.0));
}

#[test]
fn check_ceiling() {
    test_expression("Microsoft.Quantum.Math.Ceiling(3.1)", &Value::Int(4));
    test_expression("Microsoft.Quantum.Math.Ceiling(-3.7)", &Value::Int(-3));
}

#[test]
fn check_floor() {
    test_expression("Microsoft.Quantum.Math.Floor(3.7)", &Value::Int(3));
    test_expression("Microsoft.Quantum.Math.Floor(-3.1)", &Value::Int(-4));
}

#[test]
fn check_round() {
    test_expression("Microsoft.Quantum.Math.Round(3.1)", &Value::Int(3));
    test_expression("Microsoft.Quantum.Math.Round(-3.7)", &Value::Int(-4));
}

//
// Modular arithmetic
//

#[test]
fn check_modulus_i() {
    test_expression("Microsoft.Quantum.Math.ModulusI(20, 3)", &Value::Int(2));
    test_expression("Microsoft.Quantum.Math.ModulusI(-20, 3)", &Value::Int(1));
}

#[test]
fn check_modulus_l() {
    test_expression(
        "Microsoft.Quantum.Math.ModulusL(20L, 3L)",
        &Value::BigInt(BigInt::from(2)),
    );
    test_expression(
        "Microsoft.Quantum.Math.ModulusL(-20L, 3L)",
        &Value::BigInt(BigInt::from(1)),
    );
}

#[test]
fn check_exp_mod_i() {
    test_expression("Microsoft.Quantum.Math.ExpModI(1,10,10)", &Value::Int(1));
    test_expression("Microsoft.Quantum.Math.ExpModI(10,0,10)", &Value::Int(1));
    test_expression("Microsoft.Quantum.Math.ExpModI(2,10,10)", &Value::Int(4));
}

#[test]
fn check_exp_mod_l() {
    test_expression(
        "Microsoft.Quantum.Math.ExpModL(1L,10L,10L)",
        &Value::BigInt(BigInt::from(1)),
    );
    test_expression(
        "Microsoft.Quantum.Math.ExpModL(10L,0L,10L)",
        &Value::BigInt(BigInt::from(1)),
    );
    test_expression(
        "Microsoft.Quantum.Math.ExpModL(2L,10L,10L)",
        &Value::BigInt(BigInt::from(4)),
    );
}

#[test]
fn check_inverse_mod_i() {
    test_expression("Microsoft.Quantum.Math.InverseModI(2,5)", &Value::Int(3));
    test_expression("Microsoft.Quantum.Math.InverseModI(3,10)", &Value::Int(7));
    test_expression("Microsoft.Quantum.Math.InverseModI(-1,5)", &Value::Int(4));
}

#[test]
fn check_inverse_mod_l() {
    test_expression(
        "Microsoft.Quantum.Math.InverseModL(2L,5L)",
        &Value::BigInt(BigInt::from(3)),
    );
    test_expression(
        "Microsoft.Quantum.Math.InverseModL(3L,10L)",
        &Value::BigInt(BigInt::from(7)),
    );
    test_expression(
        "Microsoft.Quantum.Math.InverseModL(-1L,5L)",
        &Value::BigInt(BigInt::from(4)),
    );
}

//
// GCD, etc.
//
#[test]
fn check_gcd_i() {
    test_expression(
        "Microsoft.Quantum.Math.GreatestCommonDivisorI(0,0)",
        &Value::Int(0),
    );
    test_expression(
        "Microsoft.Quantum.Math.GreatestCommonDivisorI(2*3*5,2*3*7)",
        &Value::Int(2 * 3),
    );
    test_expression(
        "Microsoft.Quantum.Math.GreatestCommonDivisorI(39088169,63245986)",
        &Value::Int(1),
    );
}

#[test]
fn check_gcd_l() {
    test_expression(
        "Microsoft.Quantum.Math.GreatestCommonDivisorL(0L,0L)",
        &Value::BigInt(BigInt::from(0)),
    );
    test_expression(
        "Microsoft.Quantum.Math.GreatestCommonDivisorL(2L*3L*5L,2L*3L*7L)",
        &Value::BigInt(BigInt::from(2 * 3)),
    );
    test_expression("Microsoft.Quantum.Math.GreatestCommonDivisorL(222232244629420445529739893461909967206666939096499764990979600L,359579325206583560961765665172189099052367214309267232255589801L)", &Value::BigInt(
        BigInt::from(1)));
}

#[test]
fn check_cfc_i() {
    // NOTE: It is not important if the function returns -3/-4 or 3/4,
    // we can ignore this implementation details or update a function
    // to return canonical result.
    test_expression(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 2)",
        &Value::Tuple(vec![Value::Int(-1), Value::Int(-1)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 3)",
        &Value::Tuple(vec![Value::Int(2), Value::Int(3)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 4)",
        &Value::Tuple(vec![Value::Int(-3), Value::Int(-4)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 7)",
        &Value::Tuple(vec![Value::Int(5), Value::Int(7)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 25)",
        &Value::Tuple(vec![Value::Int(-18), Value::Int(-25)].into()),
    );
}

#[test]
fn check_fst_snd() {
    test_expression("Fst(7,6)", &Value::Int(7));
    test_expression("Snd(7,6)", &Value::Int(6));
}

#[test]
fn check_bitsize_i() {
    test_expression("Microsoft.Quantum.Math.BitSizeI(0)", &Value::Int(0));
    test_expression("Microsoft.Quantum.Math.BitSizeI(1)", &Value::Int(1));
    test_expression("Microsoft.Quantum.Math.BitSizeI(2)", &Value::Int(2));
    test_expression("Microsoft.Quantum.Math.BitSizeI(3)", &Value::Int(2));
    test_expression(
        "Microsoft.Quantum.Math.BitSizeI(0x7FFFFFFFFFFFFFFF)",
        &Value::Int(63),
    );
}

//
// Core namespace
//

#[test]
fn check_repeated() {
    test_expression("Repeated(Zero, 0)", &Value::Array(vec![].into()));
    test_expression(
        "Repeated(One, 1)",
        &Value::Array(vec![Value::Result(true)].into()),
    );
    test_expression(
        "Repeated(1, 2)",
        &Value::Array(vec![Value::Int(1), Value::Int(1)].into()),
    );
    test_expression(
        "Repeated(true, 3)",
        &Value::Array(vec![Value::Bool(true), Value::Bool(true), Value::Bool(true)].into()),
    );
}

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
fn check_apply_cnot_chain_2() {
    test_expression(
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
    test_expression(
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

#[test]
fn check_apply_p() {
    test_expression(
        {
            "{
            open Microsoft.Quantum.Measurement;
            use q = Qubit[3];
            ApplyP(PauliX, q[0]);
            H(q[1]); ApplyP(PauliY, q[1]);
            H(q[2]); S(q[2]); ApplyP(PauliZ, q[2]);
            return [MResetZ(q[0]),MResetX(q[1]),MResetY(q[2])];
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

#[test]
fn check_apply_pauli() {
    test_expression(
        {
            "{
            open Microsoft.Quantum.Measurement;
            use q = Qubit[3];
            H(q[1]);
            H(q[2]); S(q[2]);
            ApplyPauli([PauliX, PauliY, PauliZ], q);
            return [MResetZ(q[0]),MResetX(q[1]),MResetY(q[2])];
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

#[test]
fn check_apply_pauli_from_bit_string() {
    test_expression(
        {
            "{
            open Microsoft.Quantum.Measurement;
            use q = Qubit[3];
            ApplyPauliFromBitString(PauliX, false, [true, false, true], q);
            return MResetEachZ(q);
        }"
        },
        &Value::Array(
            vec![
                Value::Result(false),
                Value::Result(true),
                Value::Result(false),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_apply_pauli_from_int() {
    test_expression(
        {
            "{
            open Microsoft.Quantum.Measurement;
            use q = Qubit[3];
            ApplyPauliFromInt(PauliX, false, 5, q);
            return MResetEachZ(q);
        }"
        },
        &Value::Array(
            vec![
                Value::Result(false),
                Value::Result(true),
                Value::Result(false),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_apply_controlled_on_int() {
    test_expression(
        {
            "{
            open Microsoft.Quantum.Measurement;
            use c = Qubit[3];
            use t1 = Qubit();
            use t2 = Qubit();
            within {
                X(c[0]);
                X(c[2]);
            } apply {
                ApplyControlledOnInt(5, X, c, t1);
            }
            ApplyControlledOnInt(5, X, c, t2);
            return [MResetZ(t1), M(t2)];
        }"
        },
        &Value::Array(vec![Value::Result(true), Value::Result(false)].into()),
    );
}

#[test]
fn check_apply_controlled_on_bitstring() {
    test_expression(
        {
            "{
            open Microsoft.Quantum.Measurement;
            use c = Qubit[4];
            use t1 = Qubit();
            use t2 = Qubit();
            within {
                X(c[0]);
                X(c[2]);
            } apply {
                ApplyControlledOnBitString([true, false, true], X, c, t1);
            }
            ApplyControlledOnBitString([true, false, true], X, c, t2);
            return [MResetZ(t1), M(t2)];
        }"
        },
        &Value::Array(vec![Value::Result(true), Value::Result(false)].into()),
    );
}

#[test]
fn check_apply_cnot_chain_3a() {
    test_expression(
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
        &Value::Result(true),
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
                Value::Result(false),
                Value::Result(true), // 2
            ]
            .into(),
        ),
    );
}

#[test]
fn check_exp_with_cnot() {
    // This decomposition only holds if the magnitude of the angle used in Exp is correct and if the
    // sign convention between Rx, Rz, and Exp is consistent.
    test_expression(
        indoc! {r#"{
            open Microsoft.Quantum.Diagnostics;
            open Microsoft.Quantum.Math;

            use (aux, control, target) = (Qubit(), Qubit(), Qubit());
            within {
                H(aux);
                CNOT(aux, control);
                CNOT(aux, target);
            }
            apply {
                let theta  = PI() / 4.0;
                Rx(-2.0 * theta, target);
                Rz(-2.0 * theta, control);
                Adjoint Exp([PauliZ, PauliX], theta, [control, target]);

                Adjoint CNOT(control, target);
            }

            CheckAllZero([aux, control, target])
        }"#},
        &Value::Bool(true),
    );
}

#[test]
fn check_exp_with_swap() {
    // This decomposition only holds if the magnitude of the angle used in Exp is correct.
    test_expression(
        indoc! {r#"{
            open Microsoft.Quantum.Diagnostics;
            open Microsoft.Quantum.Math;

            use (aux, qs) = (Qubit(), Qubit[2]);
            within {
                H(aux);
                CNOT(aux, qs[0]);
                CNOT(aux, qs[1]);
            }
            apply {
                let theta  = PI() / 4.0;
                Exp([PauliX, PauliX], theta, qs);
                Exp([PauliY, PauliY], theta, qs);
                Exp([PauliZ, PauliZ], theta, qs);

                Adjoint SWAP(qs[0], qs[1]);
            }

            CheckAllZero([aux] + qs)
        }"#},
        &Value::Bool(true),
    );
}
