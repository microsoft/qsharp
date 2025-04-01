// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

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
    expect![[r#"
        operation my_h(q : Qubit) : Unit is Adj + Ctl {
            h(q);
        }
    "#]]
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
    expect![[r#"
        operation my_h(q : Qubit, q2 : Qubit) : Unit is Adj + Ctl {
            h(q2);
            h(q);
        }
    "#]]
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
    expect![[r#"
        operation my_h(θ : __Angle__, q : Qubit) : Unit is Adj + Ctl {
            rx(θ, q);
        }
    "#]]
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
    expect![[r#"
        operation my_h(θ : __Angle__, φ : __Angle__, q : Qubit, q2 : Qubit) : Unit is Adj + Ctl {
            rx(θ, q2);
            ry(φ, q);
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
