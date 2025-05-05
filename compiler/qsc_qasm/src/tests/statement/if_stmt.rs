// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn can_use_cond_with_implicit_cast_to_bool() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;

        h q;
        bit result = measure q;
        if (result) {
            reset q;
        }
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        h(q);
        mutable result = QIR.Intrinsic.__quantum__qis__m__body(q);
        if QasmStd.Convert.ResultAsBool(result) {
            Reset(q);
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn can_use_negated_cond_with_implicit_cast_to_bool() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;

        h q;
        bit result = measure q;
        if (!result) {
            reset q;
        }
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        h(q);
        mutable result = QIR.Intrinsic.__quantum__qis__m__body(q);
        if not QasmStd.Convert.ResultAsBool(result) {
            Reset(q);
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

/// <https://openqasm.com/language/classical.html#if-else-statements>
/// Both true-body and false-body can be a single statement terminated
/// by a semicolon, or a program block of several statements { stmt1; stmt2; }.
/// The stmts can also be on the next line.

#[test]
fn then_branch_can_be_stmt() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        if (0 == 1) z q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        if 0 == 1 {
            z(q);
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn else_branch_can_be_stmt() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        if (0 == 1) {z q;}
        else y q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        if 0 == 1 {
            z(q);
        } else {
            y(q);
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn then_and_else_branch_can_be_stmt() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        if (0 == 1) z q;
        else y q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        if 0 == 1 {
            z(q);
        } else {
            y(q);
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn using_cond_that_cannot_implicit_cast_to_bool_fail() {
    let source = r#"
        qubit q;
        if (q) {
            reset q;
        }
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect!["cannot cast expression of type Qubit to type Bool(false)"]
        .assert_eq(&errors[0].to_string());
}
