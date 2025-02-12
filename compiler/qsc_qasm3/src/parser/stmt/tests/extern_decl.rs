// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::parser::tests::check;

use crate::parser::stmt::parse;

#[test]
fn missing_semicolon_err() {
    check(
        parse,
        "extern x()",
        &expect![[r#"
            Stmt [0-10]
                StmtKind: ExternDecl [0-10]: Ident [7-8] "x"

            [
                Error(
                    Token(
                        Semicolon,
                        Eof,
                        Span {
                            lo: 10,
                            hi: 10,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn bit_param_bit_ret_decl() {
    check(
        parse,
        "extern x(bit) -> bit;",
        &expect![[r#"
            Stmt [0-21]
                StmtKind: ExternDecl [0-21]: Ident [7-8] "x"
                [9-12]: ClassicalType [9-12]: BitType
                ClassicalType [17-20]: BitType"#]],
    );
}

#[test]
fn sized_bit_param_bit_ret_decl() {
    check(
        parse,
        "extern x(bit[n]) -> bit;",
        &expect![[r#"
            Stmt [0-24]
                StmtKind: ExternDecl [0-24]: Ident [7-8] "x"
                [9-15]: ClassicalType [9-15]: BitType [9-15]: ExprStmt [12-15]: Expr [13-14]: Ident [13-14] "n"
                ClassicalType [20-23]: BitType"#]],
    );
}

#[test]
fn sized_creg_param_bit_ret_decl() {
    check(
        parse,
        "extern x(creg[n]) -> bit;",
        &expect![[r#"
            Stmt [0-25]
                StmtKind: ExternDecl [0-25]: Ident [7-8] "x"
                [9-16]: ClassicalType [9-16]: BitType [9-16]: ExprStmt [13-16]: Expr [14-15]: Ident [14-15] "n"
                ClassicalType [21-24]: BitType"#]],
    );
}

#[test]
fn creg_param_bit_ret_decl() {
    check(
        parse,
        "extern x(creg) -> bit;",
        &expect![[r#"
            Stmt [0-22]
                StmtKind: ExternDecl [0-22]: Ident [7-8] "x"
                [9-13]: ClassicalType [9-13]: BitType
                ClassicalType [18-21]: BitType"#]],
    );
}

#[test]
fn readonly_array_arg_with_int_dims() {
    check(
        parse,
        "extern x(readonly array[int[8], 2, 10]);",
        &expect![[r#"
            Stmt [0-40]
                StmtKind: ExternDecl [0-40]: Ident [7-8] "x"
                [9-38]: ArrayReferenceType [9-38]: ArrayBaseTypeKind IntType[ExprStmt [27-30]: Expr [28-29]: Lit: Int(8)]: [24-30]
                Expr [32-33]: Lit: Int(2)
                Expr [35-37]: Lit: Int(10)"#]],
    );
}

#[test]
fn readonly_array_arg_with_dim() {
    check(
        parse,
        "extern x(readonly array[int[8], #dim = 1]);",
        &expect![[r#"
            Stmt [0-43]
                StmtKind: ExternDecl [0-43]: Ident [7-8] "x"
                [9-41]: ArrayReferenceType [9-41]: ArrayBaseTypeKind IntType[ExprStmt [27-30]: Expr [28-29]: Lit: Int(8)]: [24-30]
                Expr [39-40]: Lit: Int(1)"#]],
    );
}

#[test]
fn mutable_array_arg() {
    check(
        parse,
        "extern x(mutable array[int[8], #dim = 1]);",
        &expect![[r#"
            Stmt [0-42]
                StmtKind: ExternDecl [0-42]: Ident [7-8] "x"
                [9-40]: ArrayReferenceType [9-40]: ArrayBaseTypeKind IntType[ExprStmt [26-29]: Expr [27-28]: Lit: Int(8)]: [23-29]
                Expr [38-39]: Lit: Int(1)"#]],
    );
}

#[test]
fn unexpected_ident_in_params() {
    check(
        parse,
        "extern x(creg c) -> bit;",
        &expect![[r#"
            Error(
                Token(
                    Close(
                        Paren,
                    ),
                    Identifier,
                    Span {
                        lo: 14,
                        hi: 15,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn annotation() {
    check(
        parse,
        r#"@test.annotation
        extern x(creg) -> bit;"#,
        &expect![[r#"
            Stmt [0-47]
                Annotation [0-16]: (test.annotation)
                StmtKind: ExternDecl [25-47]: Ident [32-33] "x"
                [34-38]: ClassicalType [34-38]: BitType
                ClassicalType [43-46]: BitType"#]],
    );
}

#[test]
fn missing_ty_error() {
    check(
        parse,
        "extern x() -> ;",
        &expect![[r#"
            Error(
                Rule(
                    "scalar type",
                    Semicolon,
                    Span {
                        lo: 14,
                        hi: 15,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn missing_args_with_delim_error() {
    check(
        parse,
        "extern x(,);",
        &expect![[r#"
            Stmt [0-12]
                StmtKind: ExternDecl [0-12]: Ident [7-8] "x"
                [9-9]: ClassicalType [0-0]: Err

            [
                Error(
                    MissingSeqEntry(
                        Span {
                            lo: 9,
                            hi: 9,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn args_with_extra_delim_err_ty() {
    check(
        parse,
        "extern x(int,,int);",
        &expect![[r#"
            Stmt [0-19]
                StmtKind: ExternDecl [0-19]: Ident [7-8] "x"
                [9-12]: ClassicalType [9-12]: IntType [9-12]
                [13-13]: ClassicalType [0-0]: Err
                [14-17]: ClassicalType [14-17]: IntType [14-17]

            [
                Error(
                    MissingSeqEntry(
                        Span {
                            lo: 13,
                            hi: 13,
                        },
                    ),
                ),
            ]"#]],
    );
}
