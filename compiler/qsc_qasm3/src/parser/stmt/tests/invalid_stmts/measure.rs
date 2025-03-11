// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::{stmt::parse, tests::check};
use expect_test::expect;

#[test]
fn measure_multiple_qubits() {
    check(parse, "measure $0, $1;", &expect![[r#"
        Stmt [0-10]:
            annotations: <empty>
            kind: MeasureStmt [0-10]:
                measurement: MeasureExpr [0-7]:
                    operand: HardwareQubit [8-10]: 0
                target: <none>

        [
            Error(
                Token(
                    Semicolon,
                    Comma,
                    Span {
                        lo: 10,
                        hi: 11,
                    },
                ),
            ),
        ]"#]]);
}

#[test]
fn assign_measure_multiple_qubits() {
    check(parse, "a[0:1] = measure $0, $1;", &expect![[r#"
        Error(
            Rule(
                "expression",
                Measure,
                Span {
                    lo: 9,
                    hi: 16,
                },
            ),
        )
    "#]]);
}

#[test]
fn assign_arrow() {
    check(parse, "a = measure $0 -> b;", &expect![[r#"
        Error(
            Rule(
                "expression",
                Measure,
                Span {
                    lo: 4,
                    hi: 11,
                },
            ),
        )
    "#]]);
}

#[test]
fn initialized_creg() {
    check(parse, "creg a[1] = measure $0;", &expect![[r#"
        Stmt [0-9]:
            annotations: <empty>
            kind: ClassicalDeclarationStmt [0-9]:
                type: ScalarType [0-9]: BitType [0-9]:
                    size: Expr [7-8]: Lit: Int(1)
                ident: Ident [5-6] "a"
                init_expr: <none>

        [
            Error(
                Token(
                    Semicolon,
                    Eq,
                    Span {
                        lo: 10,
                        hi: 11,
                    },
                ),
            ),
        ]"#]]);
}

#[test]
fn invalid_arrow_target() {
    check(parse, "measure $0 -> creg a[1];", &expect![[r#"
        Error(
            Rule(
                "identifier",
                Keyword(
                    CReg,
                ),
                Span {
                    lo: 14,
                    hi: 18,
                },
            ),
        )
    "#]]);
    check(parse, "measure $0 -> bit[1] a;", &expect![[r#"
        Error(
            Rule(
                "identifier",
                Type(
                    Bit,
                ),
                Span {
                    lo: 14,
                    hi: 17,
                },
            ),
        )
    "#]]);
}

#[test]
fn measure_cant_be_used_in_sub_expressions() {
    check(parse, "a = 2 * measure $0;", &expect![[r#"
        Error(
            Rule(
                "expression",
                Measure,
                Span {
                    lo: 8,
                    hi: 15,
                },
            ),
        )
    "#]]);
    check(parse, "a = (measure $0) + (measure $1);", &expect![[r#"
        Error(
            Token(
                Close(
                    Paren,
                ),
                Measure,
                Span {
                    lo: 5,
                    hi: 12,
                },
            ),
        )
    "#]]);
}
