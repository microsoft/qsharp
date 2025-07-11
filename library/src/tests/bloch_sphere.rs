// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::test_expression;
use qsc::interpret::Value;

#[test]
fn check_map_pauli_x_x() {
    test_expression(
        {
            "{
                Std.Diagnostics.CheckOperationsAreEqual(1,
                    q => Std.BlochSphere.MapPauliAxis(PauliX, PauliX, q[0]),
                    q => I(q[0])
                )
            }"
        },
        &Value::Bool(true),
    );
}

#[test]
fn check_map_pauli_y_y() {
    test_expression(
        {
            "{
                Std.Diagnostics.CheckOperationsAreEqual(1,
                    q => Std.BlochSphere.MapPauliAxis(PauliY, PauliY, q[0]),
                    q => I(q[0])
                )
            }"
        },
        &Value::Bool(true),
    );
}

#[test]
fn check_map_pauli_z_z() {
    test_expression(
        {
            "{
                Std.Diagnostics.CheckOperationsAreEqual(1,
                    q => Std.BlochSphere.MapPauliAxis(PauliZ, PauliZ, q[0]),
                    q => I(q[0])
                )
            }"
        },
        &Value::Bool(true),
    );
}

#[test]
fn check_map_pauli_x_y() {
    test_expression(
        {
            "{
                Std.Diagnostics.CheckOperationsAreEqual(1,
                    q => { within { Std.BlochSphere.MapPauliAxis(PauliX, PauliY, q[0]) } apply { Rx(0.1, q[0]) } },
                    q => { within { Adjoint S(q[0]) } apply { Rx(0.1, q[0]) } }
                )
            }"
        },
        &Value::Bool(true),
    );
}

#[test]
fn check_map_pauli_y_x() {
    test_expression(
        {
            "{
                Std.Diagnostics.CheckOperationsAreEqual(1,
                    q => { within { Std.BlochSphere.MapPauliAxis(PauliY, PauliX, q[0]) } apply { Ry(0.1, q[0]) } },
                    q => { within { S(q[0]) } apply { Ry(0.1, q[0]) } }
                )
            }"
        },
        &Value::Bool(true),
    );
}

#[test]
fn check_map_pauli_x_z() {
    test_expression(
        {
            "{
                Std.Diagnostics.CheckOperationsAreEqual(1,
                    q => { within { Std.BlochSphere.MapPauliAxis(PauliX, PauliZ, q[0]) } apply { Rx(0.1, q[0]) } },
                    q => { within { H(q[0]) } apply { Rx(0.1, q[0]) } }
                )
            }"
        },
        &Value::Bool(true),
    );
}

#[test]
fn check_map_pauli_z_x() {
    test_expression(
        {
            "{
                Std.Diagnostics.CheckOperationsAreEqual(1,
                    q => { within { Std.BlochSphere.MapPauliAxis(PauliZ, PauliX, q[0]) } apply { Rz(0.1, q[0]) } },
                    q => { within { H(q[0]) } apply { Rz(0.1, q[0]) } }
                )
            }"
        },
        &Value::Bool(true),
    );
}

#[test]
fn check_map_pauli_y_z() {
    test_expression(
        {
            "{
                Std.Diagnostics.CheckOperationsAreEqual(1,
                    q => { within { Std.BlochSphere.MapPauliAxis(PauliY, PauliZ, q[0]) } apply { Ry(0.1, q[0]) } },
                    q => { within { H(q[0]); Adjoint S(q[0]); H(q[0]) } apply { Ry(0.1, q[0]) } }
                )
            }"
        },
        &Value::Bool(true),
    );
}

#[test]
fn check_map_pauli_z_y() {
    test_expression(
        {
            "{
                Std.Diagnostics.CheckOperationsAreEqual(1,
                    q => { within { Std.BlochSphere.MapPauliAxis(PauliZ, PauliY, q[0]) } apply { Rz(0.1, q[0]) } },
                    q => { within { H(q[0]); S(q[0]); H(q[0]) } apply { Rz(0.1, q[0]) } }
                )
            }"
        },
        &Value::Bool(true),
    );
}
