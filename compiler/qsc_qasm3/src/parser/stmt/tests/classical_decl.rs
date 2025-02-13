// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::parser::tests::check;

use crate::parser::stmt::parse;

#[test]
fn bit_decl() {
    check(
        parse,
        "bit b;",
        &expect![[r#"
            Stmt [0-6]
                StmtKind: ClassicalDeclarationStmt [0-6]: ClassicalType [0-3]: BitType, Ident [4-5] "b""#]],
    );
}

#[test]
fn bit_decl_bit_lit() {
    check(
        parse,
        "bit b = 1;",
        &expect![[r#"
            Stmt [0-10]
                StmtKind: ClassicalDeclarationStmt [0-10]: ClassicalType [0-3]: BitType, Ident [4-5] "b", ValueExpression ExprStmt [8-9]: Expr [8-9]: Lit: Int(1)"#]],
    );
}

#[test]
fn const_bit_decl_bit_lit() {
    check(
        parse,
        "const bit b = 1;",
        &expect![[r#"
            Stmt [0-16]
                StmtKind: ConstantDeclaration [0-16]: ClassicalType [6-9]: BitType, Ident [10-11] "b", ExprStmt [14-15]: Expr [14-15]: Lit: Int(1)"#]],
    );
}

#[test]
fn bit_array_decl() {
    check(
        parse,
        "bit[2] b;",
        &expect![[r#"
            Stmt [0-9]
                StmtKind: ClassicalDeclarationStmt [0-9]: ClassicalType [0-6]: BitType [0-6]: ExprStmt [3-6]: Expr [4-5]: Lit: Int(2), Ident [7-8] "b""#]],
    );
}

#[test]
fn bit_array_decl_bit_lit() {
    check(
        parse,
        "bit[2] b = \"11\";",
        &expect![[r#"
            Stmt [0-16]
                StmtKind: ClassicalDeclarationStmt [0-16]: ClassicalType [0-6]: BitType [0-6]: ExprStmt [3-6]: Expr [4-5]: Lit: Int(2), Ident [7-8] "b", ValueExpression ExprStmt [11-15]: Expr [11-15]: Lit: Bitstring("11")"#]],
    );
}

#[test]
fn const_bit_array_decl_bit_lit() {
    check(
        parse,
        "const bit[2] b = \"11\";",
        &expect![[r#"
            Stmt [0-22]
                StmtKind: ConstantDeclaration [0-22]: ClassicalType [6-12]: BitType [6-12]: ExprStmt [9-12]: Expr [10-11]: Lit: Int(2), Ident [13-14] "b", ExprStmt [17-21]: Expr [17-21]: Lit: Bitstring("11")"#]],
    );
}

#[test]
fn bool_decl() {
    check(
        parse,
        "bool b;",
        &expect![[r#"
            Stmt [0-7]
                StmtKind: ClassicalDeclarationStmt [0-7]: ClassicalType [0-4]: BoolType, Ident [5-6] "b""#]],
    );
}

#[test]
fn bool_decl_int_lit() {
    check(
        parse,
        "bool b = 1;",
        &expect![[r#"
            Stmt [0-11]
                StmtKind: ClassicalDeclarationStmt [0-11]: ClassicalType [0-4]: BoolType, Ident [5-6] "b", ValueExpression ExprStmt [9-10]: Expr [9-10]: Lit: Int(1)"#]],
    );
}

#[test]
fn const_bool_decl_bool_lit() {
    check(
        parse,
        "const bool b = true;",
        &expect![[r#"
            Stmt [0-20]
                StmtKind: ConstantDeclaration [0-20]: ClassicalType [6-10]: BoolType, Ident [11-12] "b", ExprStmt [15-19]: Expr [15-19]: Lit: Bool(true)"#]],
    );
}

#[test]
fn complex_decl() {
    check(
        parse,
        "complex c;",
        &expect![[r#"
            Stmt [0-10]
                StmtKind: ClassicalDeclarationStmt [0-10]: ClassicalType [0-7]: ComplexType [0-7], Ident [8-9] "c""#]],
    );
}

#[test]
fn complex_decl_complex_lit() {
    check(
        parse,
        "complex c = 1im;",
        &expect![[r#"
            Stmt [0-16]
                StmtKind: ClassicalDeclarationStmt [0-16]: ClassicalType [0-7]: ComplexType [0-7], Ident [8-9] "c", ValueExpression ExprStmt [12-15]: Expr [12-15]: Lit: Imaginary(1.0)"#]],
    );
}

#[test]
fn const_complex_decl_complex_lit() {
    check(
        parse,
        "const complex c = 1im;",
        &expect![[r#"
            Stmt [0-22]
                StmtKind: ConstantDeclaration [0-22]: ClassicalType [6-13]: ComplexType [6-13], Ident [14-15] "c", ExprStmt [18-21]: Expr [18-21]: Lit: Imaginary(1.0)"#]],
    );
}

#[test]
#[ignore = "need binary operator support for const complex number exprs"]
fn const_complex_decl_complex_binary_lit() {
    check(
        parse,
        "const complex c = 23.5 + 1.7im;",
        &expect![[r#"
            Stmt [0-22]
                StmtKind: ConstantDeclaration [0-22]: ClassicalType [6-13]: ComplexType [6-13], Ident [14-15] "c", ExprStmt [18-22]: Expr [18-22]: Lit: Float(23.5)

            [
                Error(
                    Token(
                        Semicolon,
                        ClosedBinOp(
                            Plus,
                        ),
                        Span {
                            lo: 23,
                            hi: 24,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn complex_sized_decl() {
    check(
        parse,
        "complex[float[32]] c;",
        &expect![[r#"
            Stmt [0-21]
                StmtKind: ClassicalDeclarationStmt [0-21]: ClassicalType [0-18]: ComplexType[float[FloatType[ExprStmt [13-17]: Expr [14-16]: Lit: Int(32)]: [8-17]]]: [0-18], Ident [19-20] "c""#]],
    );
}

#[test]
fn complex_sized_non_float_subty_decl() {
    check(
        parse,
        "complex[int[32]] c;",
        &expect![[r#"
            Error(
                Rule(
                    "scalar or array type",
                    Type(
                        Int,
                    ),
                    Span {
                        lo: 8,
                        hi: 11,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn complex_sized_decl_complex_lit() {
    check(
        parse,
        "complex[float[32]] c = 1im;",
        &expect![[r#"
            Stmt [0-27]
                StmtKind: ClassicalDeclarationStmt [0-27]: ClassicalType [0-18]: ComplexType[float[FloatType[ExprStmt [13-17]: Expr [14-16]: Lit: Int(32)]: [8-17]]]: [0-18], Ident [19-20] "c", ValueExpression ExprStmt [23-26]: Expr [23-26]: Lit: Imaginary(1.0)"#]],
    );
}

#[test]
fn const_complex_sized_decl_complex_lit() {
    check(
        parse,
        "const complex[float[32]] c = 1im;",
        &expect![[r#"
            Stmt [0-33]
                StmtKind: ConstantDeclaration [0-33]: ClassicalType [6-24]: ComplexType[float[FloatType[ExprStmt [19-23]: Expr [20-22]: Lit: Int(32)]: [14-23]]]: [6-24], Ident [25-26] "c", ExprStmt [29-32]: Expr [29-32]: Lit: Imaginary(1.0)"#]],
    );
}

#[test]
fn int_decl() {
    check(
        parse,
        "int i;",
        &expect![[r#"
            Stmt [0-6]
                StmtKind: ClassicalDeclarationStmt [0-6]: ClassicalType [0-3]: IntType [0-3], Ident [4-5] "i""#]],
    );
}

#[test]
fn int_decl_int_lit() {
    check(
        parse,
        "int i = 1;",
        &expect![[r#"
            Stmt [0-10]
                StmtKind: ClassicalDeclarationStmt [0-10]: ClassicalType [0-3]: IntType [0-3], Ident [4-5] "i", ValueExpression ExprStmt [8-9]: Expr [8-9]: Lit: Int(1)"#]],
    );
}

#[test]
fn const_int_decl_int_lit() {
    check(
        parse,
        "const int i = 1;",
        &expect![[r#"
            Stmt [0-16]
                StmtKind: ConstantDeclaration [0-16]: ClassicalType [6-9]: IntType [6-9], Ident [10-11] "i", ExprStmt [14-15]: Expr [14-15]: Lit: Int(1)"#]],
    );
}

#[test]
fn int_sized_decl() {
    check(
        parse,
        "int[32] i;",
        &expect![[r#"
            Stmt [0-10]
                StmtKind: ClassicalDeclarationStmt [0-10]: ClassicalType [0-7]: IntType[ExprStmt [3-7]: Expr [4-6]: Lit: Int(32)]: [0-7], Ident [8-9] "i""#]],
    );
}

#[test]
fn int_sized_decl_int_lit() {
    check(
        parse,
        "int[32] i = 1;",
        &expect![[r#"
            Stmt [0-14]
                StmtKind: ClassicalDeclarationStmt [0-14]: ClassicalType [0-7]: IntType[ExprStmt [3-7]: Expr [4-6]: Lit: Int(32)]: [0-7], Ident [8-9] "i", ValueExpression ExprStmt [12-13]: Expr [12-13]: Lit: Int(1)"#]],
    );
}

#[test]
fn const_int_sized_decl_int_lit() {
    check(
        parse,
        "const int[32] i = 1;",
        &expect![[r#"
            Stmt [0-20]
                StmtKind: ConstantDeclaration [0-20]: ClassicalType [6-13]: IntType[ExprStmt [9-13]: Expr [10-12]: Lit: Int(32)]: [6-13], Ident [14-15] "i", ExprStmt [18-19]: Expr [18-19]: Lit: Int(1)"#]],
    );
}

#[test]
fn uint_decl() {
    check(
        parse,
        "uint i;",
        &expect![[r#"
            Stmt [0-7]
                StmtKind: ClassicalDeclarationStmt [0-7]: ClassicalType [0-4]: UIntType [0-4], Ident [5-6] "i""#]],
    );
}

#[test]
fn uint_decl_uint_lit() {
    check(
        parse,
        "uint i = 1;",
        &expect![[r#"
            Stmt [0-11]
                StmtKind: ClassicalDeclarationStmt [0-11]: ClassicalType [0-4]: UIntType [0-4], Ident [5-6] "i", ValueExpression ExprStmt [9-10]: Expr [9-10]: Lit: Int(1)"#]],
    );
}

#[test]
fn const_uint_decl_uint_lit() {
    check(
        parse,
        "const uint i = 1;",
        &expect![[r#"
            Stmt [0-17]
                StmtKind: ConstantDeclaration [0-17]: ClassicalType [6-10]: UIntType [6-10], Ident [11-12] "i", ExprStmt [15-16]: Expr [15-16]: Lit: Int(1)"#]],
    );
}

#[test]
fn uint_sized_decl() {
    check(
        parse,
        "uint[32] i;",
        &expect![[r#"
            Stmt [0-11]
                StmtKind: ClassicalDeclarationStmt [0-11]: ClassicalType [0-8]: UIntType[ExprStmt [4-8]: Expr [5-7]: Lit: Int(32)]: [0-8], Ident [9-10] "i""#]],
    );
}

#[test]
fn uint_sized_decl_uint_lit() {
    check(
        parse,
        "uint[32] i = 1;",
        &expect![[r#"
            Stmt [0-15]
                StmtKind: ClassicalDeclarationStmt [0-15]: ClassicalType [0-8]: UIntType[ExprStmt [4-8]: Expr [5-7]: Lit: Int(32)]: [0-8], Ident [9-10] "i", ValueExpression ExprStmt [13-14]: Expr [13-14]: Lit: Int(1)"#]],
    );
}

#[test]
fn const_uint_sized_decl_uint_lit() {
    check(
        parse,
        "const uint[32] i = 1;",
        &expect![[r#"
            Stmt [0-21]
                StmtKind: ConstantDeclaration [0-21]: ClassicalType [6-14]: UIntType[ExprStmt [10-14]: Expr [11-13]: Lit: Int(32)]: [6-14], Ident [15-16] "i", ExprStmt [19-20]: Expr [19-20]: Lit: Int(1)"#]],
    );
}

#[test]
fn float_decl() {
    check(
        parse,
        "float f;",
        &expect![[r#"
            Stmt [0-8]
                StmtKind: ClassicalDeclarationStmt [0-8]: ClassicalType [0-5]: FloatType [0-5], Ident [6-7] "f""#]],
    );
}

#[test]
fn float_decl_float_lit() {
    check(
        parse,
        "float f = 1;",
        &expect![[r#"
            Stmt [0-12]
                StmtKind: ClassicalDeclarationStmt [0-12]: ClassicalType [0-5]: FloatType [0-5], Ident [6-7] "f", ValueExpression ExprStmt [10-11]: Expr [10-11]: Lit: Int(1)"#]],
    );
}

#[test]
fn const_float_decl_float_lit() {
    check(
        parse,
        "const float f = 1.0;",
        &expect![[r#"
            Stmt [0-20]
                StmtKind: ConstantDeclaration [0-20]: ClassicalType [6-11]: FloatType [6-11], Ident [12-13] "f", ExprStmt [16-19]: Expr [16-19]: Lit: Float(1.0)"#]],
    );
}

#[test]
fn float_sized_decl() {
    check(
        parse,
        "float[32] f;",
        &expect![[r#"
            Stmt [0-12]
                StmtKind: ClassicalDeclarationStmt [0-12]: ClassicalType [0-9]: FloatType[ExprStmt [5-9]: Expr [6-8]: Lit: Int(32)]: [0-9], Ident [10-11] "f""#]],
    );
}

#[test]
fn float_sized_decl_float_lit() {
    check(
        parse,
        "float[32] f = 1.0;",
        &expect![[r#"
            Stmt [0-18]
                StmtKind: ClassicalDeclarationStmt [0-18]: ClassicalType [0-9]: FloatType[ExprStmt [5-9]: Expr [6-8]: Lit: Int(32)]: [0-9], Ident [10-11] "f", ValueExpression ExprStmt [14-17]: Expr [14-17]: Lit: Float(1.0)"#]],
    );
}

#[test]
fn const_float_sized_decl_float_lit() {
    check(
        parse,
        "const float[32] f = 1;",
        &expect![[r#"
            Stmt [0-22]
                StmtKind: ConstantDeclaration [0-22]: ClassicalType [6-15]: FloatType[ExprStmt [11-15]: Expr [12-14]: Lit: Int(32)]: [6-15], Ident [16-17] "f", ExprStmt [20-21]: Expr [20-21]: Lit: Int(1)"#]],
    );
}

#[test]
fn angle_decl() {
    check(
        parse,
        "angle a;",
        &expect![[r#"
            Stmt [0-8]
                StmtKind: ClassicalDeclarationStmt [0-8]: ClassicalType [0-5]: AngleType [0-5], Ident [6-7] "a""#]],
    );
}

#[test]
fn angle_decl_angle_lit() {
    check(
        parse,
        "angle a = 1.0;",
        &expect![[r#"
            Stmt [0-14]
                StmtKind: ClassicalDeclarationStmt [0-14]: ClassicalType [0-5]: AngleType [0-5], Ident [6-7] "a", ValueExpression ExprStmt [10-13]: Expr [10-13]: Lit: Float(1.0)"#]],
    );
}

#[test]
fn const_angle_decl_no_init() {
    check(
        parse,
        "const angle a;",
        &expect![[r#"
            Error(
                Token(
                    Eq,
                    Semicolon,
                    Span {
                        lo: 13,
                        hi: 14,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn const_angle_decl_angle_lit() {
    check(
        parse,
        "const angle a = 1.0;",
        &expect![[r#"
            Stmt [0-20]
                StmtKind: ConstantDeclaration [0-20]: ClassicalType [6-11]: AngleType [6-11], Ident [12-13] "a", ExprStmt [16-19]: Expr [16-19]: Lit: Float(1.0)"#]],
    );
}

#[test]
fn angle_sized_decl() {
    check(
        parse,
        "angle[32] a;",
        &expect![[r#"
            Stmt [0-12]
                StmtKind: ClassicalDeclarationStmt [0-12]: ClassicalType [0-9]: AngleType [0-9]: ExprStmt [5-9]: Expr [6-8]: Lit: Int(32), Ident [10-11] "a""#]],
    );
}

#[test]
fn angle_sized_decl_angle_lit() {
    check(
        parse,
        "angle[32] a = 1.0;",
        &expect![[r#"
            Stmt [0-18]
                StmtKind: ClassicalDeclarationStmt [0-18]: ClassicalType [0-9]: AngleType [0-9]: ExprStmt [5-9]: Expr [6-8]: Lit: Int(32), Ident [10-11] "a", ValueExpression ExprStmt [14-17]: Expr [14-17]: Lit: Float(1.0)"#]],
    );
}

#[test]
fn const_angle_sized_decl_angle_lit() {
    check(
        parse,
        "const angle[32] a = 1.0;",
        &expect![[r#"
            Stmt [0-24]
                StmtKind: ConstantDeclaration [0-24]: ClassicalType [6-15]: AngleType [6-15]: ExprStmt [11-15]: Expr [12-14]: Lit: Int(32), Ident [16-17] "a", ExprStmt [20-23]: Expr [20-23]: Lit: Float(1.0)"#]],
    );
}

#[test]
fn duration_decl() {
    check(
        parse,
        "duration d;",
        &expect![[r#"
            Stmt [0-11]
                StmtKind: ClassicalDeclarationStmt [0-11]: ClassicalType [0-8]: Duration, Ident [9-10] "d""#]],
    );
}

#[test]
#[ignore = "unimplemented: timing literal"]
fn duration_decl_ns_lit() {
    check(
        parse,
        "duration d = 1000ns;",
        &expect![[r#"
            Error(
                Lit(
                    "unimplemented: timing literal",
                    Span {
                        lo: 13,
                        hi: 19,
                    },
                ),
            )
        "#]],
    );
}

#[test]
#[ignore = "unimplemented: timing literal"]
fn duration_decl_us_lit() {
    check(
        parse,
        "duration d = 1000us;",
        &expect![[r#"
            Error(
                Lit(
                    "unimplemented: timing literal",
                    Span {
                        lo: 13,
                        hi: 19,
                    },
                ),
            )
        "#]],
    );
}

#[test]
#[ignore = "unimplemented: timing literal"]
fn duration_decl_uus_lit() {
    // uus is for µ, disabling the lint must be done at the
    // crate level, so using uus here in the test name.
    check(
        parse,
        "duration d = 1000µs;",
        &expect![[r#"
            Error(
                Lit(
                    "unimplemented: timing literal",
                    Span {
                        lo: 13,
                        hi: 20,
                    },
                ),
            )
        "#]],
    );
}

#[test]
#[ignore = "unimplemented: timing literal"]
fn duration_decl_ms_lit() {
    check(
        parse,
        "duration d = 1000ms;",
        &expect![[r#"
            Error(
                Lit(
                    "unimplemented: timing literal",
                    Span {
                        lo: 13,
                        hi: 19,
                    },
                ),
            )
        "#]],
    );
}

#[test]
#[ignore = "unimplemented: timing literal"]
fn duration_decl_s_lit() {
    check(
        parse,
        "duration d = 1000s;",
        &expect![[r#"
            Error(
                Lit(
                    "unimplemented: timing literal",
                    Span {
                        lo: 13,
                        hi: 18,
                    },
                ),
            )
        "#]],
    );
}

#[test]
#[ignore = "unimplemented: timing literal"]
fn duration_decl_dt_lit() {
    check(
        parse,
        "duration d = 1000dt;",
        &expect![[r#"
            Error(
                Lit(
                    "unimplemented: timing literal",
                    Span {
                        lo: 13,
                        hi: 19,
                    },
                ),
            )
        "#]],
    );
}

#[test]
#[ignore = "unimplemented: timing literal"]
fn const_duration_decl_dt_lit() {
    check(
        parse,
        "const duration d = 10dt;",
        &expect![[r#"
            Error(
                Lit(
                    "unimplemented: timing literal",
                    Span {
                        lo: 19,
                        hi: 23,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn stretch_decl() {
    check(
        parse,
        "stretch s;",
        &expect![[r#"
            Stmt [0-10]
                StmtKind: ClassicalDeclarationStmt [0-10]: ClassicalType [0-7]: Stretch, Ident [8-9] "s""#]],
    );
}

#[test]
fn empty_array_decl() {
    check(
        parse,
        "array[int, 0] arr = {};",
        &expect![[r#"
            Stmt [0-23]
                StmtKind: ClassicalDeclarationStmt [0-23]: ArrayType [0-13]: ArrayBaseTypeKind IntType [6-9]
                Expr [11-12]: Lit: Int(0), Ident [14-17] "arr", ValueExpression ExprStmt [20-22]: Expr [20-22]: Lit: Array:"#]],
    );
}

#[test]
fn simple_array_decl() {
    check(
        parse,
        "array[int[32], 3] arr = {1, 2, 3};",
        &expect![[r#"
        Stmt [0-34]
            StmtKind: ClassicalDeclarationStmt [0-34]: ArrayType [0-17]: ArrayBaseTypeKind IntType[ExprStmt [9-13]: Expr [10-12]: Lit: Int(32)]: [6-13]
            Expr [15-16]: Lit: Int(3), Ident [18-21] "arr", ValueExpression ExprStmt [24-33]: Expr [24-33]: Lit: Array:
                Expr { span: Span { lo: 25, hi: 26 }, kind: Lit(Lit { span: Span { lo: 25, hi: 26 }, kind: Int(1) }) }
                Expr { span: Span { lo: 28, hi: 29 }, kind: Lit(Lit { span: Span { lo: 28, hi: 29 }, kind: Int(2) }) }
                Expr { span: Span { lo: 31, hi: 32 }, kind: Lit(Lit { span: Span { lo: 31, hi: 32 }, kind: Int(3) }) }"#]],
    );
}

#[test]
fn nested_array_decl() {
    check(
        parse,
        "array[int[32], 3, 2] arr = {{1, 2}, {3, 4}, {5, 6}};",
        &expect![[r#"
            Stmt [0-52]
                StmtKind: ClassicalDeclarationStmt [0-52]: ArrayType [0-20]: ArrayBaseTypeKind IntType[ExprStmt [9-13]: Expr [10-12]: Lit: Int(32)]: [6-13]
                Expr [15-16]: Lit: Int(3)
                Expr [18-19]: Lit: Int(2), Ident [21-24] "arr", ValueExpression ExprStmt [27-51]: Expr [27-51]: Lit: Array:
                    Expr { span: Span { lo: 28, hi: 34 }, kind: Lit(Lit { span: Span { lo: 28, hi: 34 }, kind: Array([Expr { span: Span { lo: 29, hi: 30 }, kind: Lit(Lit { span: Span { lo: 29, hi: 30 }, kind: Int(1) }) }, Expr { span: Span { lo: 32, hi: 33 }, kind: Lit(Lit { span: Span { lo: 32, hi: 33 }, kind: Int(2) }) }]) }) }
                    Expr { span: Span { lo: 36, hi: 42 }, kind: Lit(Lit { span: Span { lo: 36, hi: 42 }, kind: Array([Expr { span: Span { lo: 37, hi: 38 }, kind: Lit(Lit { span: Span { lo: 37, hi: 38 }, kind: Int(3) }) }, Expr { span: Span { lo: 40, hi: 41 }, kind: Lit(Lit { span: Span { lo: 40, hi: 41 }, kind: Int(4) }) }]) }) }
                    Expr { span: Span { lo: 44, hi: 50 }, kind: Lit(Lit { span: Span { lo: 44, hi: 50 }, kind: Array([Expr { span: Span { lo: 45, hi: 46 }, kind: Lit(Lit { span: Span { lo: 45, hi: 46 }, kind: Int(5) }) }, Expr { span: Span { lo: 48, hi: 49 }, kind: Lit(Lit { span: Span { lo: 48, hi: 49 }, kind: Int(6) }) }]) }) }"#]],
    );
}

#[test]
fn measure_hardware_qubit_decl() {
    check(
        parse,
        "bit res = measure $12;",
        &expect![[r#"
            Stmt [0-22]
                StmtKind: ClassicalDeclarationStmt [0-22]: ClassicalType [0-3]: BitType, Ident [4-7] "res", ValueExpression MeasureExpr [10-17]: GateOperand HardwareQubit [18-21]: 12"#]],
    );
}

#[test]
fn measure_register_decl() {
    check(
        parse,
        "bit res = measure qubits[2][3];",
        &expect![[r#"
            Stmt [0-31]
                StmtKind: ClassicalDeclarationStmt [0-31]: ClassicalType [0-3]: BitType, Ident [4-7] "res", ValueExpression MeasureExpr [10-17]: GateOperand IndexedIdent [18-30]: Ident [18-24] "qubits"[
                IndexElement:
                    IndexSetItem Expr [25-26]: Lit: Int(2)
                IndexElement:
                    IndexSetItem Expr [28-29]: Lit: Int(3)]"#]],
    );
}
