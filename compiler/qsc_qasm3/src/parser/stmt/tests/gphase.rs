// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::stmt::parse;
use crate::parser::tests::check;
use expect_test::expect;

#[test]
fn gphase() {
    check(
        parse,
        "gphase(pi / 2);",
        &expect![[r#"
            Stmt [0-15]:
                annotations: <empty>
                kind: GPhase [0-15]:
                    modifiers: <empty>
                    args:
                        Expr [7-13]: BinaryOpExpr:
                            op: Div
                            lhs: Expr [7-9]: Ident [7-9] "pi"
                            rhs: Expr [12-13]: Lit: Int(2)
                    duration: <none>
                    qubits: <empty>"#]],
    );
}

#[test]
fn gphase_qubit_ident() {
    check(
        parse,
        "gphase(a) q0;",
        &expect![[r#"
            Stmt [0-13]:
                annotations: <empty>
                kind: GPhase [0-13]:
                    modifiers: <empty>
                    args:
                        Expr [7-8]: Ident [7-8] "a"
                    duration: <none>
                    qubits:
                        GateOperand [10-12]:
                            kind: IndexedIdent [10-12]:
                                name: Ident [10-12] "q0"
                                index_span: [0-0]
                                indices: <empty>"#]],
    );
}

#[test]
fn gphase_qubit_register() {
    check(
        parse,
        "gphase(a) q[2];",
        &expect![[r#"
            Stmt [0-15]:
                annotations: <empty>
                kind: GPhase [0-15]:
                    modifiers: <empty>
                    args:
                        Expr [7-8]: Ident [7-8] "a"
                    duration: <none>
                    qubits:
                        GateOperand [10-14]:
                            kind: IndexedIdent [10-14]:
                                name: Ident [10-11] "q"
                                index_span: [11-14]
                                indices:
                                    IndexSet [12-13]:
                                        values:
                                            Expr [12-13]: Lit: Int(2)"#]],
    );
}

#[test]
fn gphase_multiple_qubits() {
    check(
        parse,
        "gphase(a) q0, q[4];",
        &expect![[r#"
            Stmt [0-19]:
                annotations: <empty>
                kind: GPhase [0-19]:
                    modifiers: <empty>
                    args:
                        Expr [7-8]: Ident [7-8] "a"
                    duration: <none>
                    qubits:
                        GateOperand [10-12]:
                            kind: IndexedIdent [10-12]:
                                name: Ident [10-12] "q0"
                                index_span: [0-0]
                                indices: <empty>
                        GateOperand [14-18]:
                            kind: IndexedIdent [14-18]:
                                name: Ident [14-15] "q"
                                index_span: [15-18]
                                indices:
                                    IndexSet [16-17]:
                                        values:
                                            Expr [16-17]: Lit: Int(4)"#]],
    );
}

#[test]
fn gphase_no_arguments() {
    check(
        parse,
        "gphase;",
        &expect![[r#"
            Stmt [0-7]:
                annotations: <empty>
                kind: GPhase [0-7]:
                    modifiers: <empty>
                    args: <empty>
                    duration: <none>
                    qubits: <empty>

            [
                Error(
                    GPhaseInvalidArguments(
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
fn gphase_with_parameters() {
    check(
        parse,
        "gphase(pi / 2);",
        &expect![[r#"
            Stmt [0-15]:
                annotations: <empty>
                kind: GPhase [0-15]:
                    modifiers: <empty>
                    args:
                        Expr [7-13]: BinaryOpExpr:
                            op: Div
                            lhs: Expr [7-9]: Ident [7-9] "pi"
                            rhs: Expr [12-13]: Lit: Int(2)
                    duration: <none>
                    qubits: <empty>"#]],
    );
}

#[test]
fn gphase_inv_modifier() {
    check(
        parse,
        "inv @ gphase(a);",
        &expect![[r#"
            Stmt [0-16]:
                annotations: <empty>
                kind: GPhase [0-16]:
                    modifiers:
                        QuantumGateModifier [0-5]: Inv
                    args:
                        Expr [13-14]: Ident [13-14] "a"
                    duration: <none>
                    qubits: <empty>"#]],
    );
}

#[test]
fn gphase_ctrl_inv_modifiers() {
    check(
        parse,
        "ctrl @ inv @ gphase(pi / 2) q0;",
        &expect![[r#"
            Stmt [0-31]:
                annotations: <empty>
                kind: GPhase [0-31]:
                    modifiers:
                        QuantumGateModifier [0-6]: Ctrl None
                        QuantumGateModifier [7-12]: Inv
                    args:
                        Expr [20-26]: BinaryOpExpr:
                            op: Div
                            lhs: Expr [20-22]: Ident [20-22] "pi"
                            rhs: Expr [25-26]: Lit: Int(2)
                    duration: <none>
                    qubits:
                        GateOperand [28-30]:
                            kind: IndexedIdent [28-30]:
                                name: Ident [28-30] "q0"
                                index_span: [0-0]
                                indices: <empty>"#]],
    );
}
