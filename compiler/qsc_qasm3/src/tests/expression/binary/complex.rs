// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::tests::compile_qasm_stmt_to_qsharp;

use expect_test::expect;
use miette::Report;

#[test]
fn subtraction() -> miette::Result<(), Vec<Report>> {
    let source = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a - b);
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = (Microsoft.Quantum.Math.MinusC(a, b));
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn addition() -> miette::Result<(), Vec<Report>> {
    let source = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a + b);
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = (Microsoft.Quantum.Math.PlusC(a, b));
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn multiplication() -> miette::Result<(), Vec<Report>> {
    let source = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a * b);
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = (Microsoft.Quantum.Math.TimesC(a, b));
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn division() -> miette::Result<(), Vec<Report>> {
    let source = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a / b);
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = (Microsoft.Quantum.Math.DividedByC(a, b));
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "QASM3 parser bug"]
fn power() -> miette::Result<(), Vec<Report>> {
    let source = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a ** b);
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = (Microsoft.Quantum.Math.PowC(a, b));
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
