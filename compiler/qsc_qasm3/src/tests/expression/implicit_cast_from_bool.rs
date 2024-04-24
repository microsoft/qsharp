// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use expect_test::expect;
use miette::Report;

use crate::tests::compile_qasm_to_qsharp;

#[test]
fn to_bit_and_back_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        input bool a;
        bit _bit0;
        bit _bit1;
        _bit0 = true;
        _bit1 = a;
        _bit0 = _bit1;
        _bit0 = _bit1;
        a = _bit1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable _bit0 = Zero;
        mutable _bit1 = Zero;
        set _bit0 = One;
        set _bit1 = if a {
            One
        } else {
            Zero
        };
        set _bit0 = _bit1;
        set _bit0 = _bit1;
        set a = if Microsoft.Quantum.Convert.ResultAsBool(_bit1) {
            true
        } else {
            false
        };
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_bit_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bool x = true;
        bit y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = true;
        mutable y = if x {
            One
        } else {
            Zero
        };
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_implicit_int_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bool x = true;
        int y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = true;
        mutable y = if x {
            1
        } else {
            0
        };
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_explicit_int_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bool x = true;
        int[32] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = true;
        mutable y = if x {
            1
        } else {
            0
        };
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_implicit_uint_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bool x = true;
        uint y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = true;
        mutable y = if x {
            1
        } else {
            0
        };
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_explicit_uint_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bool x = true;
        uint[32] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = true;
        mutable y = if x {
            1
        } else {
            0
        };
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_explicit_bigint_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bool x = true;
        int[65] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = true;
        mutable y = if x {
            1L
        } else {
            0L
        };
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_implicit_float_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bool x = true;
        float y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = true;
        mutable y = if x {
            1.
        } else {
            0.
        };
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_explicit_float_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bool x = true;
        float[32] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = true;
        mutable y = if x {
            1.
        } else {
            0.
        };
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
