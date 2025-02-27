// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::stmt::parse;
use crate::parser::tests::check;
use expect_test::expect;

#[test]
fn gphase() {
    check(
        parse,
        "gphase q0;",
        &expect![[r#"
            Stmt [0-10]
                StmtKind: GPhase [0-10]:
                GateOperand IndexedIdent [7-9]: Ident [7-9] "q0"[]"#]],
    );
}

#[test]
fn gphase_qubit_register() {
    check(
        parse,
        "gphase q[2];",
        &expect![[r#"
            Stmt [0-12]
                StmtKind: GPhase [0-12]:
                GateOperand IndexedIdent [7-11]: Ident [7-8] "q"[
                IndexElement:
                    IndexSetItem Expr [9-10]: Lit: Int(2)]"#]],
    );
}

#[test]
fn gphase_multiple_qubits() {
    check(
        parse,
        "gphase q0, q[4];",
        &expect![[r#"
            Stmt [0-16]
                StmtKind: GPhase [0-16]:
                GateOperand IndexedIdent [7-9]: Ident [7-9] "q0"[]
                GateOperand IndexedIdent [11-15]: Ident [11-12] "q"[
                IndexElement:
                    IndexSetItem Expr [13-14]: Lit: Int(4)]"#]],
    );
}

#[test]
fn gphase_no_qubits() {
    check(
        parse,
        "inv @ gphase;",
        &expect![[r#"
            Stmt [0-13]
                StmtKind: GPhase [0-13]:"#]],
    );
}

#[test]
fn gphase_with_parameters() {
    check(
        parse,
        "gphase(pi / 2) q0;",
        &expect![[r#"
            Stmt [0-18]
                StmtKind: GPhase [0-18]:
                Expr [7-13]: BinOp (Div):
                    Expr [7-9]: Ident [7-9] "pi"
                    Expr [12-13]: Lit: Int(2)
                GateOperand IndexedIdent [15-17]: Ident [15-17] "q0"[]"#]],
    );
}

#[test]
fn gphase_inv_modifier() {
    check(
        parse,
        "inv @ gphase q0;",
        &expect![[r#"
            Stmt [0-16]
                StmtKind: GPhase [0-16]:
                GateOperand IndexedIdent [13-15]: Ident [13-15] "q0"[]"#]],
    );
}

#[test]
fn gphase_ctrl_inv_modifiers() {
    check(
        parse,
        "ctrl(2) @ inv @ gphase(pi / 2) c1, c2, q0;",
        &expect![[r#"
            Stmt [0-42]
                StmtKind: GPhase [0-42]:
                Expr [23-29]: BinOp (Div):
                    Expr [23-25]: Ident [23-25] "pi"
                    Expr [28-29]: Lit: Int(2)
                GateOperand IndexedIdent [31-33]: Ident [31-33] "c1"[]
                GateOperand IndexedIdent [35-37]: Ident [35-37] "c2"[]
                GateOperand IndexedIdent [39-41]: Ident [39-41] "q0"[]"#]],
    );
}
