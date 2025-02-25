// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::stmt::parse;
use crate::parser::tests::check;
use expect_test::expect;

#[test]
fn reset_ident() {
    check(
        parse,
        "reset a;",
        &expect![[r#"
        Stmt [0-8]
            StmtKind: ResetStmt [0-8]: GateOperand IndexedIdent [6-7]: Ident [6-7] "a"[]"#]],
    );
}

#[test]
fn reset_indexed_ident() {
    check(
        parse,
        "reset a[1];",
        &expect![[r#"
        Stmt [0-11]
            StmtKind: ResetStmt [0-11]: GateOperand IndexedIdent [6-10]: Ident [6-7] "a"[
            IndexElement:
                IndexSetItem Expr [8-9]: Lit: Int(1)]"#]],
    );
}

#[test]
fn reset_hardware_qubit() {
    check(
        parse,
        "reset $42;",
        &expect![[r#"
        Stmt [0-10]
            StmtKind: ResetStmt [0-10]: GateOperand HardwareQubit [6-9]: 42"#]],
    );
}
