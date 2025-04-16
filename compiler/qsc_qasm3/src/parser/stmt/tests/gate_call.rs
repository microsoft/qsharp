// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::stmt::parse;
use crate::parser::tests::check;
use expect_test::expect;

#[test]
fn gate_call() {
    check(
        parse,
        "H q0;",
        &expect![[r#"
            Stmt [0-5]:
                annotations: <empty>
                kind: GateCall [0-5]:
                    modifiers: <empty>
                    name: Ident [0-1] "H"
                    args: <empty>
                    duration: <none>
                    qubits:
                        GateOperand [2-4]:
                            kind: IndexedIdent [2-4]:
                                name: Ident [2-4] "q0"
                                index_span: [0-0]
                                indices: <empty>"#]],
    );
}

#[test]
fn gate_call_qubit_register() {
    check(
        parse,
        "H q[2];",
        &expect![[r#"
            Stmt [0-7]:
                annotations: <empty>
                kind: GateCall [0-7]:
                    modifiers: <empty>
                    name: Ident [0-1] "H"
                    args: <empty>
                    duration: <none>
                    qubits:
                        GateOperand [2-6]:
                            kind: IndexedIdent [2-6]:
                                name: Ident [2-3] "q"
                                index_span: [3-6]
                                indices:
                                    IndexSet [4-5]:
                                        values:
                                            Expr [4-5]: Lit: Int(2)"#]],
    );
}

#[test]
fn gate_multiple_qubits() {
    check(
        parse,
        "CNOT q0, q[4];",
        &expect![[r#"
            Stmt [0-14]:
                annotations: <empty>
                kind: GateCall [0-14]:
                    modifiers: <empty>
                    name: Ident [0-4] "CNOT"
                    args: <empty>
                    duration: <none>
                    qubits:
                        GateOperand [5-7]:
                            kind: IndexedIdent [5-7]:
                                name: Ident [5-7] "q0"
                                index_span: [0-0]
                                indices: <empty>
                        GateOperand [9-13]:
                            kind: IndexedIdent [9-13]:
                                name: Ident [9-10] "q"
                                index_span: [10-13]
                                indices:
                                    IndexSet [11-12]:
                                        values:
                                            Expr [11-12]: Lit: Int(4)"#]],
    );
}

#[test]
fn gate_with_no_qubits() {
    check(
        parse,
        "inv @ H;",
        &expect![[r#"
            Stmt [0-8]:
                annotations: <empty>
                kind: GateCall [0-8]:
                    modifiers:
                        QuantumGateModifier [0-5]:
                            modifier_keyword_span: [0-3]
                            kind: Inv
                    name: Ident [6-7] "H"
                    args: <empty>
                    duration: <none>
                    qubits: <empty>

            [
                Error(
                    MissingGateCallOperands(
                        Span {
                            lo: 0,
                            hi: 8,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn gate_call_with_parameters() {
    check(
        parse,
        "Rx(pi / 2) q0;",
        &expect![[r#"
            Stmt [0-14]:
                annotations: <empty>
                kind: GateCall [0-14]:
                    modifiers: <empty>
                    name: Ident [0-2] "Rx"
                    args:
                        Expr [3-9]: BinaryOpExpr:
                            op: Div
                            lhs: Expr [3-5]: Ident [3-5] "pi"
                            rhs: Expr [8-9]: Lit: Int(2)
                    duration: <none>
                    qubits:
                        GateOperand [11-13]:
                            kind: IndexedIdent [11-13]:
                                name: Ident [11-13] "q0"
                                index_span: [0-0]
                                indices: <empty>"#]],
    );
}

#[test]
fn gate_call_inv_modifier() {
    check(
        parse,
        "inv @ H q0;",
        &expect![[r#"
            Stmt [0-11]:
                annotations: <empty>
                kind: GateCall [0-11]:
                    modifiers:
                        QuantumGateModifier [0-5]:
                            modifier_keyword_span: [0-3]
                            kind: Inv
                    name: Ident [6-7] "H"
                    args: <empty>
                    duration: <none>
                    qubits:
                        GateOperand [8-10]:
                            kind: IndexedIdent [8-10]:
                                name: Ident [8-10] "q0"
                                index_span: [0-0]
                                indices: <empty>"#]],
    );
}

#[test]
fn gate_call_ctrl_inv_modifiers() {
    check(
        parse,
        "ctrl(2) @ inv @ Rx(pi / 2) c1, c2, q0;",
        &expect![[r#"
            Stmt [0-38]:
                annotations: <empty>
                kind: GateCall [0-38]:
                    modifiers:
                        QuantumGateModifier [0-9]:
                            modifier_keyword_span: [0-4]
                            kind: Ctrl Some(Expr { span: Span { lo: 5, hi: 6 }, kind: Lit(Lit { span: Span { lo: 5, hi: 6 }, kind: Int(2) }) })
                        QuantumGateModifier [10-15]:
                            modifier_keyword_span: [10-13]
                            kind: Inv
                    name: Ident [16-18] "Rx"
                    args:
                        Expr [19-25]: BinaryOpExpr:
                            op: Div
                            lhs: Expr [19-21]: Ident [19-21] "pi"
                            rhs: Expr [24-25]: Lit: Int(2)
                    duration: <none>
                    qubits:
                        GateOperand [27-29]:
                            kind: IndexedIdent [27-29]:
                                name: Ident [27-29] "c1"
                                index_span: [0-0]
                                indices: <empty>
                        GateOperand [31-33]:
                            kind: IndexedIdent [31-33]:
                                name: Ident [31-33] "c2"
                                index_span: [0-0]
                                indices: <empty>
                        GateOperand [35-37]:
                            kind: IndexedIdent [35-37]:
                                name: Ident [35-37] "q0"
                                index_span: [0-0]
                                indices: <empty>"#]],
    );
}

#[test]
fn binary_expr_qubit() {
    check(
        parse,
        "Name(2, 3) + a q;",
        &expect![[r#"
            Error(
                ExpectedItem(
                    Identifier,
                    Span {
                        lo: 0,
                        hi: 14,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn parametrized_gate_call() {
    check(
        parse,
        "Name(2, 3) q;",
        &expect![[r#"
            Stmt [0-13]:
                annotations: <empty>
                kind: GateCall [0-13]:
                    modifiers: <empty>
                    name: Ident [0-4] "Name"
                    args:
                        Expr [5-6]: Lit: Int(2)
                        Expr [8-9]: Lit: Int(3)
                    duration: <none>
                    qubits:
                        GateOperand [11-12]:
                            kind: IndexedIdent [11-12]:
                                name: Ident [11-12] "q"
                                index_span: [0-0]
                                indices: <empty>"#]],
    );
}

#[test]
fn parametrized_gate_call_with_designator() {
    check(
        parse,
        "Name(2, 3)[1] q;",
        &expect![[r#"
            Stmt [0-16]:
                annotations: <empty>
                kind: GateCall [0-16]:
                    modifiers: <empty>
                    name: Ident [0-4] "Name"
                    args:
                        Expr [5-6]: Lit: Int(2)
                        Expr [8-9]: Lit: Int(3)
                    duration: Expr [11-12]: Lit: Int(1)
                    qubits:
                        GateOperand [14-15]:
                            kind: IndexedIdent [14-15]:
                                name: Ident [14-15] "q"
                                index_span: [0-0]
                                indices: <empty>"#]],
    );
}

#[test]
fn multi_indexed_gate_call() {
    check(
        parse,
        "Name(2, 3)[1, 0] q;",
        &expect![[r#"
            Error(
                InvalidGateCallDesignator(
                    Span {
                        lo: 0,
                        hi: 16,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn gate_call_with_designator() {
    check(
        parse,
        "H[2us] q;",
        &expect![[r#"
            Stmt [0-9]:
                annotations: <empty>
                kind: GateCall [0-9]:
                    modifiers: <empty>
                    name: Ident [0-1] "H"
                    args: <empty>
                    duration: Expr [2-5]: Lit: Duration(2.0, Us)
                    qubits:
                        GateOperand [7-8]:
                            kind: IndexedIdent [7-8]:
                                name: Ident [7-8] "q"
                                index_span: [0-0]
                                indices: <empty>"#]],
    );
}

#[test]
fn gate_call_with_invalid_designator() {
    check(
        parse,
        "H[2us][3] q;",
        &expect![[r#"
            Stmt [0-12]:
                annotations: <empty>
                kind: GateCall [0-12]:
                    modifiers: <empty>
                    name: Ident [0-1] "H"
                    args: <empty>
                    duration: Expr [2-5]: Lit: Duration(2.0, Us)
                    qubits:
                        GateOperand [10-11]:
                            kind: IndexedIdent [10-11]:
                                name: Ident [10-11] "q"
                                index_span: [0-0]
                                indices: <empty>

            [
                Error(
                    MultipleIndexOperators(
                        Span {
                            lo: 0,
                            hi: 9,
                        },
                    ),
                ),
            ]"#]],
    );
}
