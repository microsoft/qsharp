// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use expect_test::expect;
use miette::Report;

use crate::tests::compile_qasm_to_qsharp;

#[test]
fn to_bit_implicitly() {
    let source = "
        float x = 42.;
        bit y = x;
    ";

    let Err(error) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error")
    };

    expect![r#"Cannot cast expression of type Float(None, False) to type Bit(False)"#]
        .assert_eq(&error[0].to_string());
}

#[test]
fn explicit_width_to_bit_implicitly() {
    let source = "
        float[64] x = 42.;
        bit y = x;
    ";

    let Err(error) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error")
    };

    expect![r#"Cannot cast expression of type Float(Some(64), False) to type Bit(False)"#]
        .assert_eq(&error[0].to_string());
}

#[test]
fn to_bool_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 42.;
        bool y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.;
        mutable y = if Microsoft.Quantum.Math.Truncate(x) == 0 {
            false
        } else {
            true
        };
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_implicit_int_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 42.;
        int y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.;
        mutable y = Microsoft.Quantum.Math.Truncate(x);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_explicit_int_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 42.;
        int[32] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.;
        mutable y = Microsoft.Quantum.Math.Truncate(x);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_implicit_uint_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 42.;
        uint y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.;
        mutable y = Microsoft.Quantum.Math.Truncate(x);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_explicit_uint_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 42.;
        uint[32] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.;
        mutable y = Microsoft.Quantum.Math.Truncate(x);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_explicit_bigint_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 42.;
        int[65] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.;
        mutable y = Microsoft.Quantum.Convert.IntAsBigInt(Microsoft.Quantum.Math.Truncate(x));
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_implicit_float_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 42.;
        float y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.;
        mutable y = x;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_explicit_float_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 42.;
        float[32] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.;
        mutable y = x;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_implicit_complex_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 42.;
        complex[float] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.;
        mutable y = Microsoft.Quantum.Math.Complex(x, 0.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_explicit_complex_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 42.;
        complex[float[32]] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.;
        mutable y = Microsoft.Quantum.Math.Complex(x, 0.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
