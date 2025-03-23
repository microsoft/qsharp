// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The tests in this file need to check that const exprs are
//! evaluatable at lowering time. To do that we use them in
//! contexts where they need to be const-evaluated, like array
//! sizes or type widths.

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;

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
        panic!("should have generated an error");
    };
    let errs: Vec<_> = errs.iter().map(|e| format!("{e:?}")).collect();
    let errs_string = errs.join("\n");
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

#[test]
fn ident_const() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 1;
        bit[a] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn indexed_ident() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint[2] a = {1, 2};
        bit[a[1]] r;
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

// UnaryOp Float

#[test]
fn unary_op_float_neg() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = -1.0;
        const float b = -a;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = -1.;
        let b = -a;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn unary_op_float_notb_fails() {
    let source = r#"
        const float a = -1.0;
        const float b = ~a;
        bit[b] r;
    "#;

    let Err(errs) = compile_qasm_to_qsharp(source) else {
        panic!("should have generated an error");
    };
    let errs: Vec<_> = errs.iter().map(|e| format!("{e:?}")).collect();
    let errs_string = errs.join("\n");
    expect![[r#"
        Qsc.Qasm3.Compile.TypeDoesNotSupportedUnaryNegation

          x Unary negation is not allowed for instances of Float(None, true).
           ,-[Test.qasm:3:26]
         2 |         const float a = -1.0;
         3 |         const float b = ~a;
           :                          ^
         4 |         bit[b] r;
           `----
    "#]]
    .assert_eq(&errs_string);
}

#[test]
fn unary_op_float_notl_fails() {
    let source = r#"
        const float a = -1.0;
        const float b = !a;
        bit[b] r;
    "#;

    let Err(errs) = compile_qasm_to_qsharp(source) else {
        panic!("should have generated an error");
    };
    let errs: Vec<_> = errs.iter().map(|e| format!("{e:?}")).collect();
    let errs_string = errs.join("\n");
    expect![[r#"
        Qsc.Qasm3.Compile.ExprMustBeConst

          x designator must be a const expression
           ,-[Test.qasm:4:13]
         3 |         const float b = !a;
         4 |         bit[b] r;
           :             ^
         5 |
           `----
    "#]]
    .assert_eq(&errs_string);
}

// UnaryOp Int

#[test]
fn unary_op_int_neg() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = -1;
        const int b = -a;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = -1;
        let b = -a;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn unary_op_int_notb() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const int b = ~a;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn unary_op_int_notl() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const int b = !a;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// UnaryOp UInt

#[test]
fn unary_op_uint_neg_not_allowed() {
    let source = r#"
        const uint a = -1;
        const uint b = -a;
        bit[b] r;
    "#;

    let Err(errs) = compile_qasm_to_qsharp(source) else {
        panic!("should have generated an error");
    };
    let errs: Vec<_> = errs.iter().map(|e| format!("{e:?}")).collect();
    let errs_string = errs.join("\n");
    expect![[r#"
        Qsc.Qasm3.Compile.TypeDoesNotSupportedUnaryNegation

          x Unary negation is not allowed for instances of UInt(None, true).
           ,-[Test.qasm:3:25]
         2 |         const uint a = -1;
         3 |         const uint b = -a;
           :                         ^
         4 |         bit[b] r;
           `----
    "#]]
    .assert_eq(&errs_string);
}

#[test]
fn unary_op_uint_notb() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 1;
        const uint b = ~a;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = Microsoft.Quantum.Math.Truncate(-1.);
        let b = -a;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn unary_op_uint_notl() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 1;
        const uint b = !a;
        bit[b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// BinaryOp

#[test]
fn lhs_ty_equals_rhs_ty_assumption_holds() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const float b = 2.0;
        const uint c = a + b;
        bit[c] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2.;
        let c = Microsoft.Quantum.Math.Truncate(Microsoft.Quantum.Convert.IntAsDouble(a) + b);
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// BinaryOp Float

#[test]
fn binary_op_float_add() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 1.0;
        const float b = 2.0;
        bit[a + b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1.;
        let b = 2.;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_float_sub() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 3.0;
        const float b = 2.0;
        bit[a - b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3.;
        let b = 2.;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_float_mul() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 3.0;
        const float b = 2.0;
        bit[a * b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3.;
        let b = 2.;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_float_div() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 6.0;
        const float b = 2.0;
        bit[a / b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 6.;
        let b = 2.;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_float_pow() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const float a = 2.0;
        const float b = 3.0;
        bit[a ** b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 2.;
        let b = 3.;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// BinaryOp Int

#[test]
fn binary_op_int_add() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const int b = 2;
        bit[a + b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_int_sub() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 3;
        const int b = 2;
        bit[a - b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 2;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_int_mul() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 3;
        const int b = 2;
        bit[a * b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 2;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_int_div() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 6;
        const int b = 2;
        bit[a / b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 6;
        let b = 2;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_int_pow() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 2;
        const int b = 3;
        bit[a ** b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 2;
        let b = 3;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_int_shl() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 1;
        const int b = 2;
        bit[a << b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2;
        mutable r = [Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_int_shr() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 8;
        const int b = 2;
        bit[a >> b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 8;
        let b = 2;
        mutable r = [Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_int_andb() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 3;
        const int b = 6;
        bit[a & b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 6;
        mutable r = [Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_int_orb() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 3;
        const int b = 6;
        bit[a | b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 6;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_int_xorb() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const int a = 3;
        const int b = 6;
        bit[a ^ b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 6;
        mutable r = [Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// BinaryOp UInt

#[test]
fn binary_op_uint_add() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 1;
        const uint b = 2;
        bit[a + b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_uint_sub() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 3;
        const uint b = 2;
        bit[a - b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 2;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_uint_mul() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 3;
        const uint b = 2;
        bit[a * b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 2;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_uint_div() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 6;
        const uint b = 2;
        bit[a / b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 6;
        let b = 2;
        mutable r = [Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_uint_pow() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 2;
        const uint b = 3;
        bit[a ** b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 2;
        let b = 3;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_uint_shl() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 1;
        const uint b = 2;
        bit[a << b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 1;
        let b = 2;
        mutable r = [Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_uint_shr() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 8;
        const uint b = 2;
        bit[a >> b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 8;
        let b = 2;
        mutable r = [Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_uint_andb() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 3;
        const uint b = 6;
        bit[a & b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 6;
        mutable r = [Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_uint_orb() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 3;
        const uint b = 6;
        bit[a | b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 6;
        mutable r = [Zero, Zero, Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_uint_xorb() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const uint a = 3;
        const uint b = 6;
        bit[a ^ b] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 6;
        mutable r = [Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// Binary Bool

#[test]
fn binary_op_bool_andb() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = true;
        const bool b = true;
        bit[a & b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = true;
        let b = true;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bool_andl() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = true;
        const bool b = true;
        bit[a && b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = true;
        let b = true;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bool_orb() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = false;
        const bool b = true;
        bit[a | b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = false;
        let b = true;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bool_orl() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = false;
        const bool b = true;
        bit[a || b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = false;
        let b = true;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bool_xor() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = false;
        const bool b = true;
        bit[a ^ b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = false;
        let b = true;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bool_eq() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = true;
        const bool b = true;
        bit[a == b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = true;
        let b = true;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bool_neq() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = false;
        const bool b = true;
        bit[a != b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = false;
        let b = true;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bool_gt() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = false;
        const bool b = true;
        bit[b > a] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = false;
        let b = true;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bool_lt() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = false;
        const bool b = true;
        bit[a < b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = false;
        let b = true;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// BinaryOp Bit

#[test]
fn binary_op_bit_andb() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 1;
        const bit b = 1;
        bit[a & b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let f = 1;
        let t = 1;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bit_andl() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 1;
        const bit b = 1;
        bit[a && b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = One;
        let b = One;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bit_orb() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 0;
        const bit b = 1;
        bit[a | b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let f = false;
        let t = 1;
        mutable r1 = [Zero];
        mutable r2 = [Zero];
        mutable r3 = [Zero];
        mutable r4 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bit_orl() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 0;
        const bit b = 1;
        bit[a || b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = Zero;
        let b = One;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bit_xor() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 0;
        const bit b = 1;
        bit[a ^ b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let f = false;
        let t = 1;
        mutable r1 = [Zero];
        mutable r2 = [Zero];
        mutable r3 = [Zero];
        mutable r4 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bit_eq() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 1;
        const bit b = 1;
        bit[a == b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = One;
        let b = One;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bit_neq() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 0;
        const bit b = 1;
        bit[a != b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = Zero;
        let b = One;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bit_gt() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 0;
        const bit b = 1;
        bit[b > a] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = Zero;
        let b = One;
        mutable r1 = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn binary_op_bit_lt() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit a = 0;
        const bit b = 1;
        bit[a > b] r1;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = 3;
        let b = 6;
        mutable r = [Zero, Zero, Zero, Zero, Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

// Casting

#[test]
fn casting_bool_to_uint() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bool a = true;
        bit[a] r;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        let a = true;
        mutable r = [Zero];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
