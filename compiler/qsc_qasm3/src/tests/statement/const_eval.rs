// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The tests in this file need to check that const exprs are
//! evaluatable at lowering time. To do that we use them in
//! contexts where they need to be const-evaluated, like array
//! sizes or type widths.

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;
use std::fmt::Write;

#[test]
fn const_exprs_work_in_bitarray_size_position() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const int b = 2 + a;
        const int c = a + 3;
        bit[b] r1;
        bit[c] r2;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2 + a;
        let c = a + 3;
        mutable r1 = [Zero, Zero, Zero];
        mutable r2 = [Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn const_exprs_implicit_cast_work_in_bitarray_size_position() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const float b = 2.0 + a;
        const float c = a + 3.0;
        bit[b] r1;
        bit[c] r2;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2. + Microsoft.Quantum.Convert.IntAsDouble(a);
        let c = Microsoft.Quantum.Convert.IntAsDouble(a) + 3.;
        mutable r1 = [Zero, Zero, Zero];
        mutable r2 = [Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn non_const_exprs_fail_in_bitarray_size_position() {
    let source = r#"
        const int a = 1;
        int b = 2 + a;
        int c = a + 3;
        bit[b] r1;
        bit[c] r2;
    "#;

    let Err(errs) = compile_qasm_to_qsharp(source) else {
        panic!("non const array size should have generated an error");
    };

    let mut errs_string = String::new();

    for err in errs {
        writeln!(&mut errs_string, "{err:?}").expect("");
    }

    expect![[r#"
        Qsc.Qasm3.Compile.ExprMustBeConst

          x designator must be a const expression
           ,-[Test.qasm:5:13]
         4 |         int c = a + 3;
         5 |         bit[b] r1;
           :             ^
         6 |         bit[c] r2;
           `----

        Qsc.Qasm3.Compile.ExprMustBeConst

          x designator must be a const expression
           ,-[Test.qasm:6:13]
         5 |         bit[b] r1;
         6 |         bit[c] r2;
           :             ^
         7 |     
           `----

    "#]]
    .assert_eq(&errs_string);
}

#[test]
fn can_assign_const_expr_to_non_const_decl() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const int b = 2;
        int c = a + b;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2;
        mutable c = a + b;
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
