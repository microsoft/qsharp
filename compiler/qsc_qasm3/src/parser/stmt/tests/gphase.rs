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
            Stmt [0-15]
                StmtKind: GPhase [0-15]:
                Expr [7-13]: BinOp (Div):
                    Expr [7-9]: Ident [7-9] "pi"
                    Expr [12-13]: Lit: Int(2)"#]],
    );
}

#[test]
fn gphase_qubit_ident() {
    check(
        parse,
        "gphase(a) q0;",
        &expect![[r#"
            Stmt [0-13]
                StmtKind: GPhase [0-13]:
                Expr [7-8]: Ident [7-8] "a"
                GateOperand IndexedIdent [10-12]: Ident [10-12] "q0"[]"#]],
    );
}

#[test]
fn gphase_qubit_register() {
    check(
        parse,
        "gphase(a) q[2];",
        &expect![[r#"
            Stmt [0-15]
                StmtKind: GPhase [0-15]:
                Expr [7-8]: Ident [7-8] "a"
                GateOperand IndexedIdent [10-14]: Ident [10-11] "q"[
                IndexElement:
                    IndexSetItem Expr [12-13]: Lit: Int(2)]"#]],
    );
}

#[test]
fn gphase_multiple_qubits() {
    check(
        parse,
        "gphase(a) q0, q[4];",
        &expect![[r#"
            Stmt [0-19]
                StmtKind: GPhase [0-19]:
                Expr [7-8]: Ident [7-8] "a"
                GateOperand IndexedIdent [10-12]: Ident [10-12] "q0"[]
                GateOperand IndexedIdent [14-18]: Ident [14-15] "q"[
                IndexElement:
                    IndexSetItem Expr [16-17]: Lit: Int(4)]"#]],
    );
}

#[test]
fn gphase_no_arguments() {
    check(
        parse,
        "gphase;",
        &expect![[r#"
            Stmt [0-7]
                StmtKind: GPhase [0-7]:

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
            Stmt [0-15]
                StmtKind: GPhase [0-15]:
                Expr [7-13]: BinOp (Div):
                    Expr [7-9]: Ident [7-9] "pi"
                    Expr [12-13]: Lit: Int(2)"#]],
    );
}

#[test]
fn gphase_inv_modifier() {
    check(
        parse,
        "inv @ gphase(a);",
        &expect![[r#"
            Stmt [0-16]
                StmtKind: GPhase [0-16]:
                Expr [13-14]: Ident [13-14] "a""#]],
    );
}

#[test]
fn gphase_ctrl_inv_modifiers() {
    check(
        parse,
        "ctrl @ inv @ gphase(pi / 2) q0;",
        &expect![[r#"
            Stmt [0-31]
                StmtKind: GPhase [0-31]:
                Expr [20-26]: BinOp (Div):
                    Expr [20-22]: Ident [20-22] "pi"
                    Expr [25-26]: Lit: Int(2)
                GateOperand IndexedIdent [28-30]: Ident [28-30] "q0"[]"#]],
    );
}
