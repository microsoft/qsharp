// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::{compile_qasm_stmt_to_qsharp, compile_qasm_to_qsharp};
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

#[test]
fn bitarray_indexing() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit[5] a = "10101";
        const bit b = a[2];

        def f() {
            bit c = b;
        }
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        let a = [One, Zero, One, Zero, One];
        let b = a[2];
        function f() : Unit {
            mutable c = One;
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bitarray_slicing() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit[5] a = "10101";
        const bit[3] b = a[1:3];

        def f() {
            bit[3] c = b;
        }
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        let a = [One, Zero, One, Zero, One];
        let b = a[1..3];
        function f() : Unit {
            mutable c = [Zero, One, Zero];
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bitarray_const_evaluation() -> miette::Result<(), Vec<Report>> {
    let source = "
        const bit[5] a = 10;

        def f() {
            bit b = a[1];
        }
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        let a = [Zero, One, Zero, One, Zero];
        function f() : Unit {
            mutable b = [Zero, One, Zero, One, Zero][1];
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
