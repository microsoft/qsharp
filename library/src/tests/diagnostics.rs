// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::test_expression;
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
fn check_start_stop_counting_operation_called_3_times() {
    test_expression(
        "{
            import Microsoft.Quantum.Diagnostics.StartCountingOperation;
            import Microsoft.Quantum.Diagnostics.StopCountingOperation;

            operation op1() : Unit {}
            operation op2() : Unit { op1(); }
            StartCountingOperation(op1);
            StartCountingOperation(op2);
            op1(); op1(); op2();
            (StopCountingOperation(op1), StopCountingOperation(op2))
        }",
        &Value::Tuple([Value::Int(3), Value::Int(1)].into()),
    );
}

#[test]
fn check_start_stop_counting_operation_called_0_times() {
    test_expression(
        "{
            import Microsoft.Quantum.Diagnostics.StartCountingOperation;
            import Microsoft.Quantum.Diagnostics.StopCountingOperation;

            operation op1() : Unit {}
            operation op2() : Unit { op1(); }
            StartCountingOperation(op1);
            StartCountingOperation(op2);
            (StopCountingOperation(op1), StopCountingOperation(op2))
        }",
        &Value::Tuple([Value::Int(0), Value::Int(0)].into()),
    );
}

#[test]
fn check_lambda_counted_separately_from_operation() {
    test_expression(
        "{
            import Microsoft.Quantum.Diagnostics.StartCountingOperation;
            import Microsoft.Quantum.Diagnostics.StopCountingOperation;

            operation op1() : Unit {}
            StartCountingOperation(op1);
            let lambda = () => op1();
            StartCountingOperation(lambda);
            op1();
            lambda();
            (StopCountingOperation(op1), StopCountingOperation(lambda))
        }",
        &Value::Tuple([Value::Int(2), Value::Int(1)].into()),
    );
}

#[test]
fn check_stop_counting_operation_without_start() {
    test_expression(
        "{
            import Microsoft.Quantum.Diagnostics.StopCountingOperation;

            operation op1() : Unit {}
            StopCountingOperation(op1)
        }",
        &Value::Int(-1),
    );
}

#[test]
fn check_multiple_controls_counted_together() {
    test_expression(
        "{
            import Microsoft.Quantum.Diagnostics.StartCountingOperation;
            import Microsoft.Quantum.Diagnostics.StopCountingOperation;

            operation op1() : Unit is Adj + Ctl {}
            StartCountingOperation(Controlled op1);
            Controlled op1([], ());
            Controlled Controlled op1([], ([], ()));
            Controlled Controlled Controlled op1([], ([], ([], ())));
            (StopCountingOperation(Controlled op1))
        }",
        &Value::Int(3),
    );
}

#[test]
fn check_counting_operation_differentiates_between_body_adj_ctl() {
    test_expression(
        "{
            import Microsoft.Quantum.Diagnostics.StartCountingOperation;
            import Microsoft.Quantum.Diagnostics.StopCountingOperation;

            operation op1() : Unit is Adj + Ctl {}
            StartCountingOperation(op1);
            StartCountingOperation(Adjoint op1);
            StartCountingOperation(Controlled op1);
            StartCountingOperation(Adjoint Controlled op1);
            op1();
            Adjoint op1(); Adjoint op1();
            Controlled op1([], ()); Controlled op1([], ()); Controlled op1([], ());
            Adjoint Controlled op1([], ()); Adjoint Controlled op1([], ());
            Controlled Adjoint op1([], ()); Controlled Adjoint op1([], ());
            (StopCountingOperation(op1), StopCountingOperation(Adjoint op1), StopCountingOperation(Controlled op1), StopCountingOperation(Adjoint Controlled op1))
        }",
        &Value::Tuple([Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)].into()),
    );
}

#[test]
fn check_start_stop_counting_function_called_3_times() {
    test_expression(
        "{
            import Microsoft.Quantum.Diagnostics.StartCountingFunction;
            import Microsoft.Quantum.Diagnostics.StopCountingFunction;

            function f1() : Unit {}
            function f2() : Unit { f1(); }
            StartCountingFunction(f1);
            StartCountingFunction(f2);
            f1(); f1(); f2();
            (StopCountingFunction(f1), StopCountingFunction(f2))
        }",
        &Value::Tuple([Value::Int(3), Value::Int(1)].into()),
    );
}

#[test]
fn check_start_stop_counting_function_called_0_times() {
    test_expression(
        "{
            import Microsoft.Quantum.Diagnostics.StartCountingFunction;
            import Microsoft.Quantum.Diagnostics.StopCountingFunction;

            function f1() : Unit {}
            function f2() : Unit { f1(); }
            StartCountingFunction(f1);
            StartCountingFunction(f2);
            (StopCountingFunction(f1), StopCountingFunction(f2))
        }",
        &Value::Tuple([Value::Int(0), Value::Int(0)].into()),
    );
}

#[test]
fn check_stop_counting_function_without_start() {
    test_expression(
        "{
            import Microsoft.Quantum.Diagnostics.StopCountingFunction;

            function f1() : Unit {}
            StopCountingFunction(f1)
        }",
        &Value::Int(-1),
    );
}
