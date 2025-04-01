// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn can_access_const_decls_from_global_scope() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        const int i = 7;
        gate my_h q {
            if (i == 0) {
                h q;
            }
        }
        qubit q;
        my_h q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let i = 7;
        let my_h : (Qubit) => Unit = (q) => {
            if i == 0 {
                h(q);
            };
        };
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        my_h(q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn cannot_access_mutable_decls_from_global_scope() {
    let source = r#"
        include "stdgates.inc";
        int i;
        gate my_h q {
            if (i == 0) {
                h q;
            }
        }
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected an error");
    };
    expect![r#"Undefined symbol: i."#].assert_eq(&errors[0].to_string());
}

#[test]
fn gates_can_call_previously_declared_gates() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_h q {
            h q;
        }
        gate my_hx q {
            my_h q;
            x q;
        }
        qubit q;
        my_hx q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let my_h : (Qubit) => Unit = (q) => {
            h(q);
        };
        let my_hx : (Qubit) => Unit = (q) => {
            my_h(q);
            x(q);
        };
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        my_hx(q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn def_can_call_previously_declared_def() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        def apply_h(qubit q) {
            h q;
        }
        def apply_hx(qubit q) {
            apply_h(q);
            x q;
        }
        qubit q;
        apply_hx(q);
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let apply_h : (Qubit) => Unit = (q) => {
            h(q);
        };
        let apply_hx : (Qubit) => Unit = (q) => {
            apply_h(q);
            x(q);
        };
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        apply_hx(q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn gate_can_call_previously_declared_def() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        def apply_h(qubit q) {
            h q;
        }
        gate my_hx q {
            apply_h(q);
            x q;
        }
        qubit q;
        my_hx q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let apply_h : (Qubit) => Unit = (q) => {
            h(q);
        };
        let my_hx : (Qubit) => Unit = (q) => {
            apply_h(q);
            x(q);
        };
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        my_hx(q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn def_can_call_previously_declared_gate() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_h q {
            h q;
        }
        def apply_hx(qubit q) {
            my_h q;
            x q;
        }
        qubit q;
        apply_hx(q);
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let my_h : (Qubit) => Unit = (q) => {
            h(q);
        };
        let apply_hx : (Qubit) => Unit = (q) => {
            my_h(q);
            x(q);
        };
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        apply_hx(q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn def_can_call_itself_recursively() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        def apply_hx(int limit, qubit q) {
            if (limit > 0) {
                apply_hx(limit - 1, q);
                x q;
            }
            h q;
        }
        qubit q;
        apply_hx(2, q);
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let apply_hx : (Int, Qubit) => Unit = (limit, q) => {
            if limit > 0 {
                apply_hx(limit - 1, q);
                x(q);
            };
            h(q);
        };
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        apply_hx(2, q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
