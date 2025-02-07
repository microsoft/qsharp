// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::parser::tests::check;

use crate::parser::stmt::parse;

#[test]
fn input_bit_decl() {
    check(
        parse,
        "input bit b;",
        &expect![[r#"
            Stmt [0-12]
                StmtKind: IODeclaration [0-12]: input, ClassicalType [6-9]: BitType, Ident [10-11] "b""#]],
    );
}

#[test]
fn output_bit_decl() {
    check(
        parse,
        "output bit b;",
        &expect![[r#"
            Stmt [0-13]
                StmtKind: IODeclaration [0-13]: output, ClassicalType [7-10]: BitType, Ident [11-12] "b""#]],
    );
}

#[test]
fn input_bit_array_decl() {
    check(
        parse,
        "input bit[2] b;",
        &expect![[r#"
            Stmt [0-15]
                StmtKind: IODeclaration [0-15]: input, ClassicalType [6-12]: BitType [6-12]: ExprStmt [9-12]: Expr [10-11]: Lit: Int(2), Ident [13-14] "b""#]],
    );
}

#[test]
fn output_bit_array_decl() {
    check(
        parse,
        "output bit[2] b;",
        &expect![[r#"
            Stmt [0-16]
                StmtKind: IODeclaration [0-16]: output, ClassicalType [7-13]: BitType [7-13]: ExprStmt [10-13]: Expr [11-12]: Lit: Int(2), Ident [14-15] "b""#]],
    );
}

#[test]
fn intput_bool_decl() {
    check(
        parse,
        "input bool b;",
        &expect![[r#"
            Stmt [0-13]
                StmtKind: IODeclaration [0-13]: input, ClassicalType [6-10]: BoolType, Ident [11-12] "b""#]],
    );
}

#[test]
fn output_bool_decl() {
    check(
        parse,
        "output bool b;",
        &expect![[r#"
            Stmt [0-14]
                StmtKind: IODeclaration [0-14]: output, ClassicalType [7-11]: BoolType, Ident [12-13] "b""#]],
    );
}

#[test]
fn input_complex_decl() {
    check(
        parse,
        "input complex c;",
        &expect![[r#"
            Stmt [0-16]
                StmtKind: IODeclaration [0-16]: input, ClassicalType [6-13]: ComplexType [6-13], Ident [14-15] "c""#]],
    );
}

#[test]
fn output_complex_decl() {
    check(
        parse,
        "output complex c;",
        &expect![[r#"
            Stmt [0-17]
                StmtKind: IODeclaration [0-17]: output, ClassicalType [7-14]: ComplexType [7-14], Ident [15-16] "c""#]],
    );
}

#[test]
fn input_complex_sized_decl() {
    check(
        parse,
        "input complex[float[32]] c;",
        &expect![[r#"
            Stmt [0-27]
                StmtKind: IODeclaration [0-27]: input, ClassicalType [6-24]: ComplexType[float[FloatType[ExprStmt [19-23]: Expr [20-22]: Lit: Int(32)]: [14-23]]]: [6-24], Ident [25-26] "c""#]],
    );
}

#[test]
fn output_complex_sized_decl() {
    check(
        parse,
        "output complex[float[32]] c;",
        &expect![[r#"
            Stmt [0-28]
                StmtKind: IODeclaration [0-28]: output, ClassicalType [7-25]: ComplexType[float[FloatType[ExprStmt [20-24]: Expr [21-23]: Lit: Int(32)]: [15-24]]]: [7-25], Ident [26-27] "c""#]],
    );
}

#[test]
fn input_int_decl() {
    check(
        parse,
        "input int i;",
        &expect![[r#"
            Stmt [0-12]
                StmtKind: IODeclaration [0-12]: input, ClassicalType [6-9]: IntType [6-9], Ident [10-11] "i""#]],
    );
}

#[test]
fn output_int_decl() {
    check(
        parse,
        "output int i;",
        &expect![[r#"
            Stmt [0-13]
                StmtKind: IODeclaration [0-13]: output, ClassicalType [7-10]: IntType [7-10], Ident [11-12] "i""#]],
    );
}

#[test]
fn input_int_sized_decl() {
    check(
        parse,
        "input int[32] i;",
        &expect![[r#"
            Stmt [0-16]
                StmtKind: IODeclaration [0-16]: input, ClassicalType [6-13]: IntType[ExprStmt [9-13]: Expr [10-12]: Lit: Int(32)]: [6-13], Ident [14-15] "i""#]],
    );
}

#[test]
fn output_int_sized_decl() {
    check(
        parse,
        "output int[32] i;",
        &expect![[r#"
            Stmt [0-17]
                StmtKind: IODeclaration [0-17]: output, ClassicalType [7-14]: IntType[ExprStmt [10-14]: Expr [11-13]: Lit: Int(32)]: [7-14], Ident [15-16] "i""#]],
    );
}

#[test]
fn input_uint_decl() {
    check(
        parse,
        "input uint i;",
        &expect![[r#"
            Stmt [0-13]
                StmtKind: IODeclaration [0-13]: input, ClassicalType [6-10]: UIntType [6-10], Ident [11-12] "i""#]],
    );
}

#[test]
fn output_uint_decl() {
    check(
        parse,
        "output uint i;",
        &expect![[r#"
            Stmt [0-14]
                StmtKind: IODeclaration [0-14]: output, ClassicalType [7-11]: UIntType [7-11], Ident [12-13] "i""#]],
    );
}

#[test]
fn input_uint_sized_decl() {
    check(
        parse,
        "input uint[32] i;",
        &expect![[r#"
            Stmt [0-17]
                StmtKind: IODeclaration [0-17]: input, ClassicalType [6-14]: UIntType[ExprStmt [10-14]: Expr [11-13]: Lit: Int(32)]: [6-14], Ident [15-16] "i""#]],
    );
}

#[test]
fn output_uint_sized_decl() {
    check(
        parse,
        "output uint[32] i;",
        &expect![[r#"
            Stmt [0-18]
                StmtKind: IODeclaration [0-18]: output, ClassicalType [7-15]: UIntType[ExprStmt [11-15]: Expr [12-14]: Lit: Int(32)]: [7-15], Ident [16-17] "i""#]],
    );
}

#[test]
fn input_float_decl() {
    check(
        parse,
        "input float f;",
        &expect![[r#"
            Stmt [0-14]
                StmtKind: IODeclaration [0-14]: input, ClassicalType [6-11]: FloatType [6-11], Ident [12-13] "f""#]],
    );
}

#[test]
fn output_float_decl() {
    check(
        parse,
        "output float f;",
        &expect![[r#"
            Stmt [0-15]
                StmtKind: IODeclaration [0-15]: output, ClassicalType [7-12]: FloatType [7-12], Ident [13-14] "f""#]],
    );
}

#[test]
fn input_float_sized_decl() {
    check(
        parse,
        "input float[32] f;",
        &expect![[r#"
            Stmt [0-18]
                StmtKind: IODeclaration [0-18]: input, ClassicalType [6-15]: FloatType[ExprStmt [11-15]: Expr [12-14]: Lit: Int(32)]: [6-15], Ident [16-17] "f""#]],
    );
}

#[test]
fn output_float_sized_decl() {
    check(
        parse,
        "output float[32] f;",
        &expect![[r#"
            Stmt [0-19]
                StmtKind: IODeclaration [0-19]: output, ClassicalType [7-16]: FloatType[ExprStmt [12-16]: Expr [13-15]: Lit: Int(32)]: [7-16], Ident [17-18] "f""#]],
    );
}

#[test]
fn input_angle_decl() {
    check(
        parse,
        "input angle a;",
        &expect![[r#"
            Stmt [0-14]
                StmtKind: IODeclaration [0-14]: input, ClassicalType [6-11]: AngleType [6-11], Ident [12-13] "a""#]],
    );
}

#[test]
fn output_angle_decl() {
    check(
        parse,
        "output angle a;",
        &expect![[r#"
            Stmt [0-15]
                StmtKind: IODeclaration [0-15]: output, ClassicalType [7-12]: AngleType [7-12], Ident [13-14] "a""#]],
    );
}

#[test]
fn input_angle_sized_decl() {
    check(
        parse,
        "input angle[32] a;",
        &expect![[r#"
            Stmt [0-18]
                StmtKind: IODeclaration [0-18]: input, ClassicalType [6-15]: AngleType [6-15]: ExprStmt [11-15]: Expr [12-14]: Lit: Int(32), Ident [16-17] "a""#]],
    );
}

#[test]
fn output_angle_sized_decl() {
    check(
        parse,
        "output angle[32] a;",
        &expect![[r#"
            Stmt [0-19]
                StmtKind: IODeclaration [0-19]: output, ClassicalType [7-16]: AngleType [7-16]: ExprStmt [12-16]: Expr [13-15]: Lit: Int(32), Ident [17-18] "a""#]],
    );
}

#[test]
fn input_duration_decl() {
    check(
        parse,
        "input duration d;",
        &expect![[r#"
            Stmt [0-17]
                StmtKind: IODeclaration [0-17]: input, ClassicalType [6-14]: Duration, Ident [15-16] "d""#]],
    );
}

#[test]
fn output_duration_decl() {
    check(
        parse,
        "output duration d;",
        &expect![[r#"
            Stmt [0-18]
                StmtKind: IODeclaration [0-18]: output, ClassicalType [7-15]: Duration, Ident [16-17] "d""#]],
    );
}

#[test]
fn input_stretch_decl() {
    check(
        parse,
        "input stretch s;",
        &expect![[r#"
            Stmt [0-16]
                StmtKind: IODeclaration [0-16]: input, ClassicalType [6-13]: Stretch, Ident [14-15] "s""#]],
    );
}

#[test]
fn output_stretch_decl() {
    check(
        parse,
        "output stretch s;",
        &expect![[r#"
            Stmt [0-17]
                StmtKind: IODeclaration [0-17]: output, ClassicalType [7-14]: Stretch, Ident [15-16] "s""#]],
    );
}
