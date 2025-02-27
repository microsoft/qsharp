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
            Stmt [0-5]
                StmtKind: GateCall [0-5]: Ident [0-1] "H"
                GateOperand IndexedIdent [2-4]: Ident [2-4] "q0"[]"#]],
    );
}

#[test]
fn gate_call_qubit_register() {
    check(
        parse,
        "H q[2];",
        &expect![[r#"
            Stmt [0-7]
                StmtKind: GateCall [0-7]: Ident [0-1] "H"
                GateOperand IndexedIdent [2-6]: Ident [2-3] "q"[
                IndexElement:
                    IndexSetItem Expr [4-5]: Lit: Int(2)]"#]],
    );
}

#[test]
fn gate_multiple_qubits() {
    check(
        parse,
        "CNOT q0, q[4];",
        &expect![[r#"
            Stmt [0-14]
                StmtKind: GateCall [0-14]: Ident [0-4] "CNOT"
                GateOperand IndexedIdent [5-7]: Ident [5-7] "q0"[]
                GateOperand IndexedIdent [9-13]: Ident [9-10] "q"[
                IndexElement:
                    IndexSetItem Expr [11-12]: Lit: Int(4)]"#]],
    );
}

#[test]
fn gate_no_qubits() {
    check(
        parse,
        "inv @ H;",
        &expect![[r#"
            Stmt [0-8]
                StmtKind: GateCall [0-8]: Ident [6-7] "H"

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
            Stmt [0-14]
                StmtKind: GateCall [0-14]: Ident [0-2] "Rx"
                Expr [3-9]: BinOp (Div):
                    Expr [3-5]: Ident [3-5] "pi"
                    Expr [8-9]: Lit: Int(2)
                GateOperand IndexedIdent [11-13]: Ident [11-13] "q0"[]"#]],
    );
}

#[test]
fn gate_call_inv_modifier() {
    check(
        parse,
        "inv @ H q0;",
        &expect![[r#"
            Stmt [0-11]
                StmtKind: GateCall [0-11]: Ident [6-7] "H"
                GateOperand IndexedIdent [8-10]: Ident [8-10] "q0"[]"#]],
    );
}

#[test]
fn gate_call_ctrl_inv_modifiers() {
    check(
        parse,
        "ctrl(2) @ inv @ Rx(pi / 2) c1, c2, q0;",
        &expect![[r#"
            Stmt [0-38]
                StmtKind: GateCall [0-38]: Ident [16-18] "Rx"
                Expr [19-25]: BinOp (Div):
                    Expr [19-21]: Ident [19-21] "pi"
                    Expr [24-25]: Lit: Int(2)
                GateOperand IndexedIdent [27-29]: Ident [27-29] "c1"[]
                GateOperand IndexedIdent [31-33]: Ident [31-33] "c2"[]
                GateOperand IndexedIdent [35-37]: Ident [35-37] "q0"[]"#]],
    );
}
