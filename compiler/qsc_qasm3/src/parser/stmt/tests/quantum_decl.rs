// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::parser::tests::check;

use crate::parser::stmt::parse;

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
fn annotated_quantum_decl() {
    check(
        parse,
        r#"
        @a.b.c 123
        qubit q;"#,
        &expect![[r#"
            Stmt [9-36]
                Annotation [9-19]: (a.b.c, 123)
                StmtKind: QubitDeclaration [28-36]: Ident [34-35] "q""#]],
    );
}

#[test]
fn multi_annotated_quantum_decl() {
    check(
        parse,
        r#"
        @g.h dolor sit amet, consectetur adipiscing elit
        @d.e.f
        @a.b.c 123
        qubit q;"#,
        &expect![[r#"
            Stmt [9-108]
                Annotation [9-57]: (g.h, dolor sit amet, consectetur adipiscing elit)
                Annotation [66-72]: (d.e.f)
                Annotation [81-91]: (a.b.c, 123)
                StmtKind: QubitDeclaration [100-108]: Ident [106-107] "q""#]],
    );
}

#[test]
fn quantum_decl_missing_name() {
    check(
        parse,
        "qubit;",
        &expect![[r#"
            Error(
                Rule(
                    "identifier",
                    Semicolon,
                    Span {
                        lo: 5,
                        hi: 6,
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
                StmtKind: QubitDeclaration [0-16]: Ident [9-15] "qubits", Expr [6-7]: Lit: Int(5)"#]],
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
                        lo: 8,
                        hi: 8,
                    },
                ),
            )
        "#]],
    );
}
