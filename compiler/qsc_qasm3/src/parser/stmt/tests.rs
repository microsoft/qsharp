use expect_test::expect;

use crate::parser::tests::check;

use super::parse;

#[test]
fn quantum_decl() {
    check(
        parse,
        "qubit q;",
        &expect![[r#"
            Stmt [0-8]
            StmtKind: QubitDeclaration [0-8]: Ident [6-7] "q""#]],
    );
}

#[test]
fn quantum_decl_missing_name() {
    check(
        parse,
        "qubit",
        &expect![[r#"
            Error(
                Rule(
                    "identifier",
                    Semicolon,
                    Span {
                        lo: 6,
                        hi: 7,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn quantum_decl_with_designator() {
    check(
        parse,
        "qubit[5] qubits;",
        &expect![[r#"
            Stmt [0-16]
            StmtKind: QubitDeclaration [0-16]: Ident [9-15] "qubits", ExprStmt [5-8]: Expr [6-7]: Literal Lit [6-7]: Integer: 5"#]],
    );
}

#[test]
fn quantum_decl_with_designator_missing_name() {
    check(
        parse,
        "qubit[5]",
        &expect![[r#"
            Error(
                Rule(
                    "identifier",
                    Eof,
                    Span {
                        lo: 9,
                        hi: 9,
                    },
                ),
            )
        "#]],
    );
}
