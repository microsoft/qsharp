// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::tests::{compile_qasm_stmt_to_qsharp, compile_qasm_to_qsharp};

use expect_test::expect;
use miette::Report;

#[test]
fn unresolved_idenfiers_raise_symbol_error() {
    let source = "
        float x = t;
    ";

    let Err(errors) = compile_qasm_stmt_to_qsharp(source) else {
        panic!("Expected an error");
    };
    assert_eq!(1, errors.len(), "Expected one error");
    expect![r#"Undefined symbol: t."#].assert_eq(&errors[0].to_string());
}

// this test verifies QASM behavior that would normally be allowed
// by the Q# compiler
#[test]
fn redefining_symbols_in_same_scope_raise_symbol_error() {
    let source = "
        float x = 0;
        float x = 5;
    ";

    let Err(errors) = compile_qasm_stmt_to_qsharp(source) else {
        panic!("Expected an error");
    };
    assert_eq!(1, errors.len(), "Expected one error");
    expect![r#"Redefined symbol: x."#].assert_eq(&errors[0].to_string());
}

#[test]
fn resolved_idenfiers_are_compiled_as_refs() -> miette::Result<(), Vec<Report>> {
    let source = "
        float p = pi;
        float x = p;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        mutable p = Microsoft.Quantum.Math.PI();
        mutable x = p;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn euler_latin_is_resolved() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = euler;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.E();
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn euler_unicode_is_resolved() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = ℇ;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.E();
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn pi_latin_is_resolved() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = pi;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.PI();
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn pi_unicode_is_resolved() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = π;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.PI();
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn tau_latin_is_resolved() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = tau;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 2. * Microsoft.Quantum.Math.PI();
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn tau_unicode_is_resolved() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = τ;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 2. * Microsoft.Quantum.Math.PI();
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
