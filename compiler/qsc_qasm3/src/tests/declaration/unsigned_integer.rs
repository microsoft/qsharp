// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_stmt_to_qsharp;

use expect_test::expect;
use miette::Report;

#[test]
fn implicit_bitness_int_default_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        uint x;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 0;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_int_default_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const uint x;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 0;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_int_lit_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const uint x = 42;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 42;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "oq3 parser bug, capital X is not recognized as hex"]
fn implicit_bitness_int_hex_cap_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        uint x = 0XFa_1F;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 64031;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_int_hex_low_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const uint x = 0xFa_1F;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 64031;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "oq3 parser bug, capital X is not recognized as hex"]
fn const_implicit_bitness_int_hex_cap_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const uint y = 0XFa_1F;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 64031;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn implicit_bitness_int_octal_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        uint x = 0o42;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 34;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_int_octal_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const uint x = 0o42;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 34;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn implicit_bitness_int_binary_low_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        uint x = 0b1001_1001;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 153;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "oq3 parser bug, capital B is not recognized as binary"]
fn implicit_bitness_int_binary_cap_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        uint x = 0B1010;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 10;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_int_binary_low_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const uint x = 0b1001_1001;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 153;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
#[ignore = "oq3 parser bug, capital B is not recognized as binary"]
fn const_implicit_bitness_int_binary_cap_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const uint x = 0B1010;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 10;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn implicit_bitness_int_formatted_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        uint x = 2_0_00;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 2000;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_int_formatted_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const uint x = 2_0_00;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 2000;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn explicit_bitness_int_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        uint[10] x;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = 0;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_explicit_bitness_int_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const uint[10] x;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = 0;
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
