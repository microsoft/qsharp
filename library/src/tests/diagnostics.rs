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
