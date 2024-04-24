// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn single_qubit_can_be_measured_into_single_bit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        bit c;
        qubit q;
        c = measure q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
            mutable c = Zero;
            let q = QIR.Runtime.__quantum__rt__qubit_allocate();
            set c = M(q);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "Arrow syntax is not supported yet in the parser"]
fn single_qubit_can_be_arrow_measured_into_single_bit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        bit c;
        qubit q;
        measure q -> c;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
            mutable c = Zero;
            let q = QIR.Runtime.__quantum__rt__qubit_allocate();
            set c = M(q);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn indexed_single_qubit_can_be_measured_into_indexed_bit_register(
) -> miette::Result<(), Vec<Report>> {
    let source = r#"
        bit[1] c;
        qubit[1] q;
        c[0] = measure q[0];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
            mutable c = [Zero];
            let q = QIR.Runtime.AllocateQubitArray(1);
            set c w/= 0 <- M(q[0]);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn indexed_single_qubit_can_be_measured_into_single_bit_register() -> miette::Result<(), Vec<Report>>
{
    let source = r#"
        bit c;
        qubit[1] q;
        c = measure q[0];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
            mutable c = Zero;
            let q = QIR.Runtime.AllocateQubitArray(1);
            set c = M(q[0]);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn measuring_hardware_qubits_generates_an_error() {
    let source = r#"
        bit c;
        c = measure $2;
    "#;

    let Err(err) = compile_qasm_to_qsharp(source) else {
        panic!("Measuring HW qubit should have generated an error");
    };
    assert!(
        err.len() == 1,
        "Expected a single error when measuring a HW qubit, got: {err:#?}"
    );

    assert!(err[0]
        .to_string()
        .contains("Hardware qubit operands are not supported"));
}

#[test]
fn value_from_measurement_can_be_dropped() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        qubit q;
        measure q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
            let q = QIR.Runtime.__quantum__rt__qubit_allocate();
            M(q);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
