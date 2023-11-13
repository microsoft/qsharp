// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{test_expression, test_expression_with_lib};
use indoc::indoc;
use qsc::interpret::Value;

// Tests for Microsoft.Quantum.Canon namespace

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

#[test]
fn check_fst_snd() {
    test_expression("Fst(7,6)", &Value::Int(7));
    test_expression("Snd(7,6)", &Value::Int(6));
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

const QFT_LE_TEST_LIB: &str = include_str!("resources/qft_le.qs");

#[test]
fn check_qft_le_sample_1() {
    test_expression_with_lib(
        "Test.TestQFT(1)",
        QFT_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_qft_le_sample_2() {
    test_expression_with_lib(
        "Test.TestQFT(2)",
        QFT_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}
#[test]
fn check_qft_le_sample_3() {
    test_expression_with_lib(
        "Test.TestQFT(3)",
        QFT_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}
#[test]
fn check_qft_le_sample_4() {
    test_expression_with_lib(
        "Test.TestQFT(4)",
        QFT_LE_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_swap_reverse_register() {
    test_expression(
        {
            "{
                open Microsoft.Quantum.Arithmetic;
                use q = Qubit[10];
                ApplyXorInPlace(328, q);
                SwapReverseRegister(q);
                let r = MeasureInteger(q);
                ResetAll(q);
                r
        }"
        },
        &Value::Int(74),
    );
}
