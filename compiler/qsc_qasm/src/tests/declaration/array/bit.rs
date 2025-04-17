// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_stmt_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn bitarray_with_bitstring() -> miette::Result<(), Vec<Report>> {
    let source = r#"
            bit[4] b = "0100";
        "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
            mutable b = [Zero, One, Zero, Zero];
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bitarray_with_formatted_bitstring() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        bit[8] b = "1001_0100";
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
            mutable b = [One, Zero, Zero, One, Zero, One, Zero, Zero];
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bitarray_with_no_initializer() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        bit[8] b;
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
            mutable b = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bitarray_with_int_initializer() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        bit[3] b = 7;
    "#;

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
            mutable b = [One, One, One];
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
