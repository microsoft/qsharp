// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn adj_x_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        inv @ x q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        Adjoint X(q);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn adj_adj_x_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        inv @ inv @ x q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        Adjoint Adjoint X(q);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn multiple_controls_on_x_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[3] q;
        qubit f;
        ctrl(3) @ x q[1], q[0], q[2], f;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(3);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        Controlled X([q[1], q[0], q[2]], f);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn repeated_multi_controls_on_x_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[2] q;
        qubit[3] r;
        qubit f;
        ctrl(2) @ ctrl(3) @ x q[1], r[0], q[0], f, r[1], r[2];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(2);
        let r = QIR.Runtime.AllocateQubitArray(3);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        Controlled Controlled X([q[1], r[0]], ([q[0], f, r[1]], r[2]));
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn repeated_multi_controls_on_x_gate_can_be_mixed_with_inv() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[2] q;
        qubit[3] r;
        qubit f;
        ctrl(2) @ inv @ ctrl(3) @ inv @ x q[1], r[0], q[0], f, r[1], r[2];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(2);
        let r = QIR.Runtime.AllocateQubitArray(3);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        Controlled Adjoint Controlled Adjoint X([q[1], r[0]], ([q[0], f, r[1]], r[2]));
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn multiple_controls_on_cx_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[4] q;
        qubit f;
        ctrl(3) @ cx q[1], q[0], q[2], f, q[3];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(4);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        Controlled CNOT([q[1], q[0], q[2]], (f, q[3]));
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn multiple_controls_on_crx_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[4] q;
        qubit f;
        ctrl(3) @ inv @ crx(0.5) q[1], q[0], q[2], f, q[3];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(4);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        Controlled Adjoint Controlled Rx([q[1], q[0], q[2]], ([f], (0.5, q[3])));
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
