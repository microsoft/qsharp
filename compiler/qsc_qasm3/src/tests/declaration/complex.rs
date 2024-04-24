// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::tests::{compile_qasm_stmt_to_qsharp, gen_qsharp, qasm_to_program_fragments};
use crate::{tests::fail_on_compilation_errors, tests::parse};

use expect_test::expect;
use miette::Report;

#[test]
// TODO: break this into multiple tests for binops
fn complex() -> miette::Result<(), Vec<Report>> {
    let source = "
        complex[float] a;
        complex[float] b = 4 - 5.5im;
        complex[float[64]] c = a + 3 im;
        complex[float[32]] d = a * b;
        complex[float] e = 1im;
        complex[float] f = 1	im;
        complex g = c * d;
        complex h = g / 2.2im;
        complex z;
    ";

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program_fragments(res.source, res.source_map);
    fail_on_compilation_errors(&unit);
    let Some(package) = &unit.package else {
        panic!("no package found");
    };
    let qsharp = gen_qsharp(package);
    println!("{qsharp}");
    Ok(())
}

#[test]
fn implicit_bitness_default_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        complex[float] x;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.Complex(0., 0.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_default_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const complex[float] x;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = Microsoft.Quantum.Math.Complex(0., 0.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn explicit_bitness_default_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        complex[float[42]] x;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.Complex(0., 0.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_explicit_bitness_default_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const complex[float[42]] x;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = Microsoft.Quantum.Math.Complex(0., 0.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_double_img_only_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const complex[float] x = 1.01im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = Microsoft.Quantum.Math.Complex(0., 1.01);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_int_img_only_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const complex[float] x = 1im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = Microsoft.Quantum.Math.Complex(0., 1.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_explicit_bitness_double_img_only_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const complex[float[42]] x = 1.01im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = Microsoft.Quantum.Math.Complex(0., 1.01);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_explicit_bitness_int_img_only_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const complex[float[42]] x = 1im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = Microsoft.Quantum.Math.Complex(0., 1.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn implicit_bitness_double_img_only_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        complex[float] x = 1.01im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.Complex(0., 1.01);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn implicit_bitness_int_img_only_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        complex[float] x = 1im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.Complex(0., 1.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

// This test is based on spec decls which show this exact case
#[test]
fn implicit_bitness_int_img_only_tab_between_suffix_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        complex[float] x = 1	im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.Complex(0., 1.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_double_real_only_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const complex[float] x = 1.01;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = Microsoft.Quantum.Math.Complex(1.01, 0.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_int_real_only_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const complex[float] x = 1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = Microsoft.Quantum.Math.Complex(1., 0.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn implicit_bitness_double_real_only_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        complex[float] x = 1.01;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.Complex(1.01, 0.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn implicit_bitness_int_real_only_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        complex[float] x = 1;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.Complex(1., 0.);
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn implicit_bitness_simple_double_pos_im_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        complex[float] x = 1.1 + 2.2im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.PlusC(Microsoft.Quantum.Math.Complex(1.1, 0.), Microsoft.Quantum.Math.Complex(0., 2.2));
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_simple_double_pos_im_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const complex[float] x = 1.1 + 2.2im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = Microsoft.Quantum.Math.PlusC(Microsoft.Quantum.Math.Complex(1.1, 0.), Microsoft.Quantum.Math.Complex(0., 2.2));
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn implicit_bitness_simple_double_neg_im_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        complex[float] x = 1.1 - 2.2im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.MinusC(Microsoft.Quantum.Math.Complex(1.1, 0.), Microsoft.Quantum.Math.Complex(0., 2.2));
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_simple_double_neg_im_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const complex[float] x = 1.1 - 2.2im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = Microsoft.Quantum.Math.MinusC(Microsoft.Quantum.Math.Complex(1.1, 0.), Microsoft.Quantum.Math.Complex(0., 2.2));
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn implicit_bitness_simple_double_neg_real_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        complex[float] x = -1.1 + 2.2im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        mutable x = Microsoft.Quantum.Math.PlusC(Microsoft.Quantum.Math.Complex(-1.1, 0.), Microsoft.Quantum.Math.Complex(0., 2.2));
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_implicit_bitness_simple_double_neg_real_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        const complex[float] x = -1.1 + 2.2im;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let x = Microsoft.Quantum.Math.PlusC(Microsoft.Quantum.Math.Complex(-1.1, 0.), Microsoft.Quantum.Math.Complex(0., 2.2));
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
