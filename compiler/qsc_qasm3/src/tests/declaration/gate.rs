// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::tests::compile_qasm_stmt_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn single_qubit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_h q {
            h q;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let my_h : (Qubit) => Unit = (q) => {
            H(q);
        };
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn two_qubits() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_h q, q2 {
            h q2;
            h q;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let my_h : (Qubit, Qubit) => Unit = (q, q2) => {
            H(q2);
            H(q);
        };
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn single_angle_single_qubit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_h(θ) q {
            rx(θ) q;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let my_h : (Double, Qubit) => Unit = (θ, q) => {
            Rx(θ, q);
        };
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn two_angles_two_qubits() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_h(θ, φ) q, q2 {
            rx(θ) q2;
            ry(φ) q;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let my_h : (Double, Double, Qubit, Qubit) => Unit = (θ, φ, q, q2) => {
            Rx(θ, q2);
            Ry(φ, q);
        };
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
