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
        function __BoolAsResult__(input : Bool) : Result {
            Microsoft.Quantum.Convert.BoolAsResult(input)
        }
        function __ResultAsBool__(input : Result) : Bool {
            Microsoft.Quantum.Convert.ResultAsBool(input)
        }
        mutable _bit0 = Zero;
        mutable _bit1 = Zero;
        set _bit0 = One;
        set _bit1 = __BoolAsResult__(a);
        set _bit0 = _bit1;
        set _bit0 = _bit1;
        set a = __ResultAsBool__(_bit1);
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
        function __BoolAsResult__(input : Bool) : Result {
            Microsoft.Quantum.Convert.BoolAsResult(input)
        }
        mutable x = true;
        mutable y = __BoolAsResult__(x);
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
        function __BoolAsInt__(value : Bool) : Int {
            if value {
                1
            } else {
                0
            }
        }
        mutable x = true;
        mutable y = __BoolAsInt__(x);
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
        function __BoolAsInt__(value : Bool) : Int {
            if value {
                1
            } else {
                0
            }
        }
        mutable x = true;
        mutable y = __BoolAsInt__(x);
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
        function __BoolAsInt__(value : Bool) : Int {
            if value {
                1
            } else {
                0
            }
        }
        mutable x = true;
        mutable y = __BoolAsInt__(x);
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
        function __BoolAsInt__(value : Bool) : Int {
            if value {
                1
            } else {
                0
            }
        }
        mutable x = true;
        mutable y = __BoolAsInt__(x);
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
        function __BoolAsBigInt__(value : Bool) : BigInt {
            if value {
                1L
            } else {
                0L
            }
        }
        mutable x = true;
        mutable y = __BoolAsBigInt__(x);
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
        function __BoolAsDouble__(value : Bool) : Double {
            if value {
                1.
            } else {
                0.
            }
        }
        mutable x = true;
        mutable y = __BoolAsDouble__(x);
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
        function __BoolAsDouble__(value : Bool) : Double {
            if value {
                1.
            } else {
                0.
            }
        }
        mutable x = true;
        mutable y = __BoolAsDouble__(x);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
