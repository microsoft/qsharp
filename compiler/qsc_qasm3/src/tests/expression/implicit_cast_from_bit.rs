// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;
use miette::Report;

use crate::tests::compile_qasm_to_qsharp;

#[test]
fn to_bool_and_back_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        input bit a;
        bool _bool0;
        bool _bool1;
        _bool0 = true;
        _bool1 = a;
        _bool0 = _bool1;
        _bool0 = _bool1;
        a = _bool1;
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
        mutable _bool0 = false;
        mutable _bool1 = false;
        set _bool0 = true;
        set _bool1 = __ResultAsBool__(a);
        set _bool0 = _bool1;
        set _bool0 = _bool1;
        set a = __BoolAsResult__(_bool1);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_bool_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit x = 1;
        bool y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        function __ResultAsBool__(input : Result) : Bool {
            Microsoft.Quantum.Convert.ResultAsBool(input)
        }
        mutable x = One;
        mutable y = __ResultAsBool__(x);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_implicit_int_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit x = 1;
        int y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        function __ResultAsInt__(input : Result) : Int {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1
            } else {
                0
            }
        }
        mutable x = One;
        mutable y = __ResultAsInt__(x);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_explicit_int_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit x = 1;
        int[32] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        function __ResultAsInt__(input : Result) : Int {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1
            } else {
                0
            }
        }
        mutable x = One;
        mutable y = __ResultAsInt__(x);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_implicit_uint_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit x = 1;
        uint y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        function __ResultAsInt__(input : Result) : Int {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1
            } else {
                0
            }
        }
        mutable x = One;
        mutable y = __ResultAsInt__(x);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_explicit_uint_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit x = 1;
        uint[32] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        function __ResultAsInt__(input : Result) : Int {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1
            } else {
                0
            }
        }
        mutable x = One;
        mutable y = __ResultAsInt__(x);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_explicit_bigint_implicitly() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit x = 1;
        int[65] y = x;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        function __ResultAsBigInt__(input : Result) : BigInt {
            if Microsoft.Quantum.Convert.ResultAsBool(input) {
                1L
            } else {
                0L
            }
        }
        mutable x = One;
        mutable y = __ResultAsBigInt__(x);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn to_implicit_float_implicitly_fails() {
    let source = "
        bit x = 1;
        float y = x;
    ";

    let Err(error) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error")
    };

    expect![r#"Cannot cast expression of type Bit(False) to type Float(None, False)"#]
        .assert_eq(&error[0].to_string());
}
