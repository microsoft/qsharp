// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn sdg_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        sdg q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        Adjoint s(q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn tdg_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        tdg q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        Adjoint t(q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn crx_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[2] q;
        crx(0.5) q[1], q[0];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.AllocateQubitArray(2);
        Controlled rx([q[1]], (__DoubleAsAngle__(0.5, 53), q[0]));
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn cry_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[2] q;
        cry(0.5) q[1], q[0];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.AllocateQubitArray(2);
        Controlled ry([q[1]], (__DoubleAsAngle__(0.5, 53), q[0]));
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn crz_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[2] q;
        crz(0.5) q[1], q[0];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.AllocateQubitArray(2);
        Controlled rz([q[1]], (__DoubleAsAngle__(0.5, 53), q[0]));
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn ch_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[2] q;
        ch q[1], q[0];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.AllocateQubitArray(2);
        Controlled h([q[1]], q[0]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
