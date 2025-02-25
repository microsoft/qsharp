// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::parser::tests::check;

use crate::parser::stmt::parse;

#[test]
fn minimal() {
    check(
        parse,
        "def x() { }",
        &expect![[r#"
            Stmt [0-11]
                StmtKind: DefStmt [0-11]: Ident [4-5] "x"(<no params>) "#]],
    );
}

#[test]
fn missing_ty_error() {
    check(
        parse,
        "def x() -> { }",
        &expect![[r#"
            Error(
                Rule(
                    "scalar type",
                    Open(
                        Brace,
                    ),
                    Span {
                        lo: 11,
                        hi: 12,
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
        "def x(,) { }",
        &expect![[r#"
            Stmt [0-12]
                StmtKind: DefStmt [0-12]: Ident [4-5] "x"([6-6] Ident [0-0] "": ClassicalType [0-0]: Err)

            [
                Error(
                    MissingSeqEntry(
                        Span {
                            lo: 6,
                            hi: 6,
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
        "def x(int a,,int b) { }",
        &expect![[r#"
            Stmt [0-23]
                StmtKind: DefStmt [0-23]: Ident [4-5] "x"([6-11] Ident [10-11] "a": ClassicalType [6-9]: IntType [6-9], [12-12] Ident [0-0] "": ClassicalType [0-0]: Err, [13-18] Ident [17-18] "b": ClassicalType [13-16]: IntType [13-16])

            [
                Error(
                    MissingSeqEntry(
                        Span {
                            lo: 12,
                            hi: 12,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn classical_subroutine() {
    check(
        parse,
        "def square(int[32] x) -> int { return x ** 2; }",
        &expect![[r#"
            Stmt [0-47]
                StmtKind: DefStmt [0-47]: Ident [4-10] "square"([11-20] Ident [19-20] "x": ClassicalType [11-18]: IntType[Expr [15-17]: Lit: Int(32)]: [11-18])
                Stmt [31-45]
                    StmtKind: ReturnStmt [31-45]: ValueExpression Expr [38-44]: BinOp (Exp):
                        Expr [38-39]: Ident [38-39] "x"
                        Expr [43-44]: Lit: Int(2)
                ClassicalType [25-28]: IntType [25-28]"#]],
    );
}

#[test]
fn quantum_args() {
    check(
        parse,
        "def x(qubit q, qubit[n] qubits) { }",
        &expect![[r#"
            Stmt [0-35]
                StmtKind: DefStmt [0-35]: Ident [4-5] "x"([6-13] Ident [12-13] "q": qubit, [15-30] Ident [24-30] "qubits": qubit[Expr [21-22]: Ident [21-22] "n"]) "#]],
    );
}

#[test]
fn old_style_args() {
    check(
        parse,
        "def test(creg c, qreg q, creg c2[2], qreg q4[4]) -> int { return x ** 2; }",
        &expect![[r#"
            Stmt [0-74]
                StmtKind: DefStmt [0-74]: Ident [4-8] "test"([9-15] Ident [14-15] "c": ClassicalType [9-15]: BitType, [17-23] Ident [22-23] "q": qubit, [25-35] Ident [30-32] "c2": ClassicalType [25-35]: BitType [25-35]: Expr [33-34]: Lit: Int(2), [37-47] Ident [42-44] "q4": qubit[Expr [45-46]: Lit: Int(4)])
                Stmt [58-72]
                    StmtKind: ReturnStmt [58-72]: ValueExpression Expr [65-71]: BinOp (Exp):
                        Expr [65-66]: Ident [65-66] "x"
                        Expr [70-71]: Lit: Int(2)
                ClassicalType [52-55]: IntType [52-55]"#]],
    );
}

#[test]
fn readonly_array_arg_with_int_dims() {
    check(
        parse,
        "def specified_sub(readonly array[int[8], 2, 10] arr_arg) {}",
        &expect![[r#"
            Stmt [0-59]
                StmtKind: DefStmt [0-59]: Ident [4-17] "specified_sub"([18-55] Ident [48-55] "arr_arg": ArrayReferenceType [18-47]: ArrayBaseTypeKind IntType[Expr [37-38]: Lit: Int(8)]: [33-39]
                Expr [41-42]: Lit: Int(2)
                Expr [44-46]: Lit: Int(10)) "#]],
    );
}

#[test]
fn readonly_array_arg_with_dim() {
    check(
        parse,
        "def arr_subroutine(readonly array[int[8], #dim = 1] arr_arg) {}",
        &expect![[r#"
            Stmt [0-63]
                StmtKind: DefStmt [0-63]: Ident [4-18] "arr_subroutine"([19-59] Ident [52-59] "arr_arg": ArrayReferenceType [19-51]: ArrayBaseTypeKind IntType[Expr [38-39]: Lit: Int(8)]: [34-40]
                Expr [49-50]: Lit: Int(1)) "#]],
    );
}

#[test]
fn mutable_array_arg() {
    check(
        parse,
        "def mut_subroutine(mutable array[int[8], #dim = 1] arr_arg) {}",
        &expect![[r#"
            Stmt [0-62]
                StmtKind: DefStmt [0-62]: Ident [4-18] "mut_subroutine"([19-58] Ident [51-58] "arr_arg": ArrayReferenceType [19-50]: ArrayBaseTypeKind IntType[Expr [37-38]: Lit: Int(8)]: [33-39]
                Expr [48-49]: Lit: Int(1)) "#]],
    );
}
