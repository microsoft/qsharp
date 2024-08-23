// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::test_expression;
use expect_test::expect;
use qsc::interpret::Value;

#[test]
fn check_operations_are_equal() {
    test_expression(
        "{
            open Microsoft.Quantum.Diagnostics;
            open Microsoft.Quantum.Arrays;
            operation op1(xs: Qubit[]): Unit is Adj {
                CCNOT(xs[0], xs[1], xs[2]);
            }
            operation op2(xs: Qubit[]): Unit is Adj {
                Controlled X(Most(xs), Tail(xs));
            }
            operation op3(xs: Qubit[]): Unit is Adj {
                Controlled X(Rest(xs), Head(xs));
            }
            [CheckOperationsAreEqual(3, op1, op2),
             CheckOperationsAreEqual(3, op2, op1),
             CheckOperationsAreEqual(3, op1, op3),
             CheckOperationsAreEqual(3, op3, op1),
             CheckOperationsAreEqual(3, op2, op3),
             CheckOperationsAreEqual(3, op3, op2)]

        }",
        &Value::Array(
            vec![
                Value::Bool(true),
                Value::Bool(true),
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
fn check_dumpoperation_for_i() {
    let output = test_expression(
        "Microsoft.Quantum.Diagnostics.DumpOperation(1, qs => I(qs[0]))",
        &Value::unit(),
    );
    expect![[r#"
        MATRIX:
        1.0000+0.0000𝑖 0.0000+0.0000𝑖
        0.0000+0.0000𝑖 1.0000+0.0000𝑖
    "#]]
    .assert_eq(&output);
}

#[test]
fn check_dumpoperation_for_x() {
    let output = test_expression(
        "Microsoft.Quantum.Diagnostics.DumpOperation(1, qs => X(qs[0]))",
        &Value::unit(),
    );
    expect![[r#"
        MATRIX:
        0.0000+0.0000𝑖 1.0000+0.0000𝑖
        1.0000+0.0000𝑖 0.0000+0.0000𝑖
    "#]]
    .assert_eq(&output);
}

#[test]
fn check_dumpoperation_for_h() {
    let output = test_expression(
        "Microsoft.Quantum.Diagnostics.DumpOperation(1, qs => H(qs[0]))",
        &Value::unit(),
    );
    expect![[r#"
        MATRIX:
        0.7071+0.0000𝑖 0.7071+0.0000𝑖
        0.7071+0.0000𝑖 −0.7071+0.0000𝑖
    "#]]
    .assert_eq(&output);
}

#[test]
fn check_dumpoperation_for_y() {
    let output = test_expression(
        "Microsoft.Quantum.Diagnostics.DumpOperation(1, qs => Y(qs[0]))",
        &Value::unit(),
    );
    expect![[r#"
        MATRIX:
        0.0000+0.0000𝑖 0.0000−1.0000𝑖
        0.0000+1.0000𝑖 0.0000+0.0000𝑖
    "#]]
    .assert_eq(&output);
}

#[test]
fn check_dumpoperation_for_ccnot() {
    let output = test_expression(
        "Microsoft.Quantum.Diagnostics.DumpOperation(3, qs => CCNOT(qs[0], qs[1], qs[2]))",
        &Value::unit(),
    );
    expect![[r#"
        MATRIX:
        1.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖
        0.0000+0.0000𝑖 1.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖
        0.0000+0.0000𝑖 0.0000+0.0000𝑖 1.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖
        0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 1.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖
        0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 1.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖
        0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 1.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖
        0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 1.0000+0.0000𝑖
        0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 0.0000+0.0000𝑖 1.0000+0.0000𝑖 0.0000+0.0000𝑖
    "#]].assert_eq(&output);
}

#[test]
fn check_dumpoperation_with_extra_qubits_allocated() {
    let output = test_expression(
        "{use qs = Qubit[2]; Microsoft.Quantum.Diagnostics.DumpOperation(1, qs => H(qs[0]))}",
        &Value::unit(),
    );
    expect![[r#"
        MATRIX:
        0.7071+0.0000𝑖 0.7071+0.0000𝑖
        0.7071+0.0000𝑖 −0.7071+0.0000𝑖
    "#]]
    .assert_eq(&output);
}

#[test]
fn check_dumpoperation_with_extra_qubits_in_superposition() {
    let output = test_expression(
        "{use qs = Qubit[2]; H(qs[0]); Microsoft.Quantum.Diagnostics.DumpOperation(1, qs => H(qs[0])); Reset(qs[0]);}",
        &Value::unit(),
    );
    expect![[r#"
        MATRIX:
        0.7071+0.0000𝑖 0.7071+0.0000𝑖
        0.7071+0.0000𝑖 −0.7071+0.0000𝑖
    "#]]
    .assert_eq(&output);
}

#[test]
fn check_dumpoperation_with_extra_qubits_global_phase_reflected_in_matrix() {
    let output = test_expression(
        "{use qs = Qubit[2]; R(PauliI, Std.Math.PI() / 2.0, qs[0]); Microsoft.Quantum.Diagnostics.DumpOperation(1, qs => H(qs[0])); Reset(qs[0]);}",
        &Value::unit(),
    );
    expect![[r#"
        MATRIX:
        0.5000−0.5000𝑖 0.5000−0.5000𝑖
        0.5000−0.5000𝑖 −0.5000+0.5000𝑖
    "#]]
    .assert_eq(&output);
}

#[test]
fn check_dumpoperation_with_extra_qubits_relative_phase_not_reflected_in_matrix() {
    let output = test_expression(
        "{use qs = Qubit[2]; R1(Std.Math.PI() / 2.0, qs[0]); Microsoft.Quantum.Diagnostics.DumpOperation(1, qs => H(qs[0])); Reset(qs[0]);}",
        &Value::unit(),
    );
    expect![[r#"
        MATRIX:
        0.7071+0.0000𝑖 0.7071+0.0000𝑖
        0.7071+0.0000𝑖 −0.7071+0.0000𝑖
    "#]]
    .assert_eq(&output);
}
