// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::parser::tests::check;

use crate::parser::stmt::parse;

#[test]
fn no_qubits_no_classical() {
    check(
        parse,
        "gate c0q0 {}",
        &expect![[r#"
            Stmt [0-12]
                StmtKind: Gate [0-12]: Ident [5-9] "c0q0"(<no params>) 
        "#]],
    );
}

#[test]
fn no_qubits_no_classical_with_parens() {
    check(
        parse,
        "gate c0q0() {}",
        &expect![[r#"
            Stmt [0-14]
                StmtKind: Gate [0-14]: Ident [5-9] "c0q0"(<no params>) 
        "#]],
    );
}

#[test]
fn one_qubit_no_classical() {
    check(
        parse,
        "gate c0q1 a {}",
        &expect![[r#"
            Stmt [0-14]
                StmtKind: Gate [0-14]: Ident [5-9] "c0q1"(<no params>) Ident [10-11] "a"
        "#]],
    );
}

#[test]
fn two_qubits_no_classical() {
    check(
        parse,
        "gate c0q2 a, b {}",
        &expect![[r#"
            Stmt [0-17]
                StmtKind: Gate [0-17]: Ident [5-9] "c0q2"(<no params>) Ident [10-11] "a", Ident [13-14] "b"
        "#]],
    );
}

#[test]
fn three_qubits_trailing_comma_no_classical() {
    check(
        parse,
        "gate c0q3 a, b, c, {}",
        &expect![[r#"
            Stmt [0-21]
                StmtKind: Gate [0-21]: Ident [5-9] "c0q3"(<no params>) Ident [10-11] "a", Ident [13-14] "b", Ident [16-17] "c"
        "#]],
    );
}

#[test]
fn no_qubits_one_classical() {
    check(
        parse,
        "gate c1q0(a) {}",
        &expect![[r#"
            Stmt [0-15]
                StmtKind: Gate [0-15]: Ident [5-9] "c1q0"(Ident [10-11] "a") 
        "#]],
    );
}

#[test]
fn no_qubits_two_classical() {
    check(
        parse,
        "gate c2q0(a, b) {}",
        &expect![[r#"
            Stmt [0-18]
                StmtKind: Gate [0-18]: Ident [5-9] "c2q0"(Ident [10-11] "a", Ident [13-14] "b") 
        "#]],
    );
}

#[test]
fn no_qubits_three_classical() {
    check(
        parse,
        "gate c3q0(a, b, c) {}",
        &expect![[r#"
            Stmt [0-21]
                StmtKind: Gate [0-21]: Ident [5-9] "c3q0"(Ident [10-11] "a", Ident [13-14] "b", Ident [16-17] "c") 
        "#]],
    );
}

#[test]
fn one_qubit_one_classical() {
    check(
        parse,
        "gate c1q1(a) b {}",
        &expect![[r#"
            Stmt [0-17]
                StmtKind: Gate [0-17]: Ident [5-9] "c1q1"(Ident [10-11] "a") Ident [13-14] "b"
        "#]],
    );
}

#[test]
fn two_qubits_two_classical() {
    check(
        parse,
        "gate c2q2(a, b) c, d {}",
        &expect![[r#"
            Stmt [0-23]
                StmtKind: Gate [0-23]: Ident [5-9] "c2q2"(Ident [10-11] "a", Ident [13-14] "b") Ident [16-17] "c", Ident [19-20] "d"
        "#]],
    );
}

#[test]
fn two_qubits_two_classical_with_body() {
    check(
        parse,
        "gate c2q2(a, b) c, d { float[32] x = a - b; }",
        &expect![[r#"
            Stmt [0-45]
                StmtKind: Gate [0-45]: Ident [5-9] "c2q2"(Ident [10-11] "a", Ident [13-14] "b") Ident [16-17] "c", Ident [19-20] "d"

                Stmt [23-43]
                    StmtKind: ClassicalDeclarationStmt [23-43]: ClassicalType [23-32]: FloatType[ExprStmt [28-32]: Expr [29-31]: Lit: Int(32)]: [23-32], Ident [33-34] "x", ValueExpression ExprStmt [37-42]: Expr [37-42]: BinOp (Sub):
                        Expr [37-38]: Ident [37-38] "a"
                        Expr [41-42]: Ident [41-42] "b""#]],
    );
}
