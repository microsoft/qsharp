// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::tests::compile_qasm_stmt_to_qsharp;

use expect_test::expect;
use miette::Report;

#[test]
fn implicit_bitness_default_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 0.;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_default_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const float x;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 0.;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn lit_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 42.1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_lit_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const float x = 42.1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn lit_explicit_width_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        float[64] x = 42.1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_explicit_width_lit_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const float[64] x = 42.1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "oq3 parser bug, can't read float with leading dot"]
fn lit_decl_leading_dot() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = .421;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 0.421;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "oq3 parser bug, can't read float with leading dot"]
fn const_lit_decl_leading_dot() -> miette::Result<(), Vec<Report>> {
    let source = "
        const float x = .421;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 0.421;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "oq3 parser bug, can't read float with leading dot"]
fn const_lit_decl_leading_dot_scientific() -> miette::Result<(), Vec<Report>> {
    let source = "
        const float x = .421e2;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn lit_decl_trailing_dot() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 421.;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 421.;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_lit_decl_trailing_dot() -> miette::Result<(), Vec<Report>> {
    let source = "
        const float x = 421.;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 421.;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn lit_decl_scientific() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 4.21e1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_lit_decl_scientific() -> miette::Result<(), Vec<Report>> {
    let source = "
        const float x = 4.21e1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn lit_decl_scientific_signed_pos() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 4.21e+1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_lit_decl_scientific_signed_pos() -> miette::Result<(), Vec<Report>> {
    let source = "
        const float x = 4.21e+1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn lit_decl_scientific_cap_e() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 4.21E1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_lit_decl_scientific_cap_e() -> miette::Result<(), Vec<Report>> {
    let source = "
        const float x = 4.21E1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn lit_decl_scientific_signed_neg() -> miette::Result<(), Vec<Report>> {
    let source = "
        float x = 421.0e-1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_lit_decl_scientific_signed_neg() -> miette::Result<(), Vec<Report>> {
    let source = "
        const float x = 421.0e-1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 42.1;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_lit_decl_signed_float_lit_cast_neg() -> miette::Result<(), Vec<Report>> {
    let source = "
        const float x = -7.;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = -7.;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_lit_decl_signed_int_lit_cast_neg() -> miette::Result<(), Vec<Report>> {
    let source = "
        const float x = -7;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = -7.;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
