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
            Stmt [5-50]:
                annotations: <empty>
                kind: BoxStmt [5-50]:
                    duration: <none>
                    body:
                        Stmt [19-24]:
                            annotations: <empty>
                            kind: GateCall [19-24]:
                                modifiers: <empty>
                                name: Ident [19-20] "H"
                                args: <empty>
                                duration: <none>
                                qubits:
                                    GateOperand IndexedIdent [21-23]:
                                        name: Ident [21-23] "q0"
                                        indices: <empty>
                        Stmt [33-44]:
                            annotations: <empty>
                            kind: GateCall [33-44]:
                                modifiers: <empty>
                                name: Ident [33-35] "Rx"
                                args:
                                    Expr [36-39]: Lit: Float(2.4)
                                duration: <none>
                                qubits:
                                    GateOperand IndexedIdent [41-43]:
                                        name: Ident [41-43] "q1"
                                        indices: <empty>"#]],
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
            Stmt [5-55]:
                annotations: <empty>
                kind: BoxStmt [5-55]:
                    duration: Expr [9-12]: Lit: Duration(4.0, Us)
                    body:
                        Stmt [24-29]:
                            annotations: <empty>
                            kind: GateCall [24-29]:
                                modifiers: <empty>
                                name: Ident [24-25] "H"
                                args: <empty>
                                duration: <none>
                                qubits:
                                    GateOperand IndexedIdent [26-28]:
                                        name: Ident [26-28] "q0"
                                        indices: <empty>
                        Stmt [38-49]:
                            annotations: <empty>
                            kind: GateCall [38-49]:
                                modifiers: <empty>
                                name: Ident [38-40] "Rx"
                                args:
                                    Expr [41-44]: Lit: Float(2.4)
                                duration: <none>
                                qubits:
                                    GateOperand IndexedIdent [46-48]:
                                        name: Ident [46-48] "q1"
                                        indices: <empty>"#]],
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
            Stmt [0-54]:
                annotations: <empty>
                kind: BoxStmt [0-54]:
                    duration: <none>
                    body:
                        Stmt [14-19]:
                            annotations: <empty>
                            kind: GateCall [14-19]:
                                modifiers: <empty>
                                name: Ident [14-15] "H"
                                args: <empty>
                                duration: <none>
                                qubits:
                                    GateOperand IndexedIdent [16-18]:
                                        name: Ident [16-18] "q0"
                                        indices: <empty>
                        Stmt [28-34]:
                            annotations: <empty>
                            kind: Err
                        Stmt [43-48]:
                            annotations: <empty>
                            kind: GateCall [43-48]:
                                modifiers: <empty>
                                name: Ident [43-44] "X"
                                args: <empty>
                                duration: <none>
                                qubits:
                                    GateOperand IndexedIdent [45-47]:
                                        name: Ident [45-47] "q1"
                                        indices: <empty>

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
