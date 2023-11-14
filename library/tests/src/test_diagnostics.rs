// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::test_expression;
use qsc::interpret::Value;

#[test]
fn check_operations_are_equal() {
    test_expression(
        "{
            open Microsoft.Quantum.Diagnostics;
            open Microsoft.Quantum.Arrays;
            operation op1(xs: Qubit[]): Unit is Adj+Ctl {
                CCNOT(xs[0], xs[1], xs[2]);
            }
            operation op2(xs: Qubit[]): Unit is Adj {
                Controlled X(Most(xs), Tail(xs));
            }
            operation op3(xs: Qubit[]): Unit is Adj {
                Controlled X(Rest(xs), Head(xs));
            }
            [CheckOperationsAreEqual(3, op1, op2), CheckOperationsAreEqual(3, op1, op3)]

        }",
        &Value::Array(vec![Value::Bool(true), Value::Bool(false)].into()),
    );
}
