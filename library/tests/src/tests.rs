// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::test_expression;
use indoc::indoc;
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
        &Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ONE, Value::RESULT_ONE].into()),
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
        &Value::Array(vec![Value::RESULT_ZERO, Value::RESULT_ZERO, Value::RESULT_ZERO].into()),
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
        &Value::Array(vec![Value::RESULT_ZERO, Value::RESULT_ZERO, Value::RESULT_ZERO].into()),
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
        &Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ONE, Value::RESULT_ONE].into()),
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
        &Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ONE, Value::RESULT_ONE].into()),
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
        &Value::Array(vec![Value::RESULT_ZERO, Value::RESULT_ZERO, Value::RESULT_ZERO].into()),
    );
}

//
// Core namespace
//

#[test]
fn check_fst_snd() {
    test_expression("Fst(7,6)", &Value::Int(7));
    test_expression("Snd(7,6)", &Value::Int(6));
}

#[test]
fn check_repeated() {
    test_expression("Repeated(Zero, 0)", &Value::Array(vec![].into()));
    test_expression(
        "Repeated(One, 1)",
        &Value::Array(vec![Value::RESULT_ONE].into()),
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
        &Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ZERO].into()),
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
        &Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ONE, Value::RESULT_ONE].into()),
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
        &Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ONE, Value::RESULT_ONE].into()),
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
        &Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ONE, Value::RESULT_ONE].into()),
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
        &Value::Array(vec![Value::RESULT_ZERO, Value::RESULT_ONE, Value::RESULT_ZERO].into()),
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
        &Value::Array(vec![Value::RESULT_ZERO, Value::RESULT_ONE, Value::RESULT_ZERO].into()),
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
        &Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ZERO].into()),
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
        &Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ZERO].into()),
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
        &Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ONE, Value::RESULT_ZERO].into()),
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
