// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_stmt_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn no_parameters_no_return() -> miette::Result<(), Vec<Report>> {
    let source = "def empty() {}";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        let empty : () -> Unit = () -> {};
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn single_parameter() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def square(int x) -> int {
            return x * x;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        let square : (Int) -> Int = (x) -> {
            return x * x;
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn qubit_parameter() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def square(qubit q) -> uint {
            return 1;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        let square : (Qubit) => Int = (q) => {
            return 1;
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn qubit_array_parameter() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        def square(qubit[3] qs) -> uint {
            return 1;
        }
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![[r#"
        let square : (Qubit[]) => Int = (qs) => {
            return 1;
        };
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
