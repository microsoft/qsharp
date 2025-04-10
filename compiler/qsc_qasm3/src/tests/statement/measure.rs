// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;
use std::fmt::Write;

#[test]
fn single_qubit_can_be_measured_into_single_bit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        bit c;
        qubit q;
        c = measure q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable c = Zero;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        set c = QIR.Intrinsic.__quantum__qis__m__body(q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn single_qubit_can_be_arrow_measured_into_single_bit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        bit c;
        qubit q;
        measure q -> c;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable c = Zero;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        set c = QIR.Intrinsic.__quantum__qis__m__body(q);
    "#]]
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
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable c = [Zero];
        let q = QIR.Runtime.AllocateQubitArray(1);
        set c w/= 0 <- QIR.Intrinsic.__quantum__qis__m__body(q[0]);
    "#]]
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
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable c = Zero;
        let q = QIR.Runtime.AllocateQubitArray(1);
        set c = QIR.Intrinsic.__quantum__qis__m__body(q[0]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn measuring_hardware_qubits_generates_an_error() {
    let source = r#"
        bit c;
        c = measure $2;
    "#;

    let Err(errs) = compile_qasm_to_qsharp(source) else {
        panic!("Measuring HW qubit should have generated an error");
    };

    let mut errs_string = String::new();

    for err in errs {
        writeln!(&mut errs_string, "{err:?}").expect("");
    }

    expect![[r#"
        Qsc.Qasm3.Compiler.NotSupported

          x Hardware qubit operands are not supported.
           ,-[Test.qasm:3:21]
         2 |         bit c;
         3 |         c = measure $2;
           :                     ^^
         4 |     
           `----

    "#]]
    .assert_eq(&errs_string);
}

#[test]
fn value_from_measurement_can_be_dropped() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        qubit q;
        measure q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        QIR.Intrinsic.__quantum__qis__m__body(q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
