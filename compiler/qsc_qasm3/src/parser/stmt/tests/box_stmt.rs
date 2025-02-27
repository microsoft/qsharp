// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::stmt::parse;
use crate::parser::tests::check;
use expect_test::expect;

#[test]
fn box_stmt() {
    check(
        parse,
        "
    box {
        H q0;
        Rx(2.4) q1;
    }",
        &expect![[r#"
            Stmt [5-50]
                StmtKind: BoxStmt [5-50]: <no duration>
                Stmt [19-24]
                    StmtKind: GateCall [19-24]: Ident [19-20] "H"
                    GateOperand IndexedIdent [21-23]: Ident [21-23] "q0"[]
                Stmt [33-44]
                    StmtKind: GateCall [33-44]: Ident [33-35] "Rx"
                    Expr [36-39]: Lit: Float(2.4)
                    GateOperand IndexedIdent [41-43]: Ident [41-43] "q1"[]"#]],
    );
}

#[test]
fn box_stmt_with_designator() {
    check(
        parse,
        "
    box[4us] {
        H q0;
        Rx(2.4) q1;
    }",
        &expect![[r#"
            Stmt [5-55]
                StmtKind: BoxStmt [5-55]: Expr [9-12]: Lit: Duration(4.0, Us)
                Stmt [24-29]
                    StmtKind: GateCall [24-29]: Ident [24-25] "H"
                    GateOperand IndexedIdent [26-28]: Ident [26-28] "q0"[]
                Stmt [38-49]
                    StmtKind: GateCall [38-49]: Ident [38-40] "Rx"
                    Expr [41-44]: Lit: Float(2.4)
                    GateOperand IndexedIdent [46-48]: Ident [46-48] "q1"[]"#]],
    );
}

#[test]
fn box_stmt_with_invalid_instruction() {
    check(
        parse,
        "box {
        H q0;
        2 + 4;
        X q1;
    }",
        &expect![[r#"
            Stmt [0-54]
                StmtKind: BoxStmt [0-54]: <no duration>
                Stmt [14-19]
                    StmtKind: GateCall [14-19]: Ident [14-15] "H"
                    GateOperand IndexedIdent [16-18]: Ident [16-18] "q0"[]
                Stmt [28-34]
                    StmtKind: Err
                Stmt [43-48]
                    StmtKind: GateCall [43-48]: Ident [43-44] "X"
                    GateOperand IndexedIdent [45-47]: Ident [45-47] "q1"[]

            [
                Error(
                    ClassicalStmtInBox(
                        Span {
                            lo: 28,
                            hi: 34,
                        },
                    ),
                ),
            ]"#]],
    );
}
