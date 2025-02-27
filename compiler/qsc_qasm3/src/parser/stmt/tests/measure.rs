// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::stmt::parse;
use crate::parser::tests::check;
use expect_test::expect;

#[test]
fn measure_identifier() {
    check(
        parse,
        "measure q;",
        &expect![[r#"
        Stmt [0-10]
            StmtKind: MeasureStmt [0-10]: MeasureExpr [0-7]: GateOperand IndexedIdent [8-9]: Ident [8-9] "q"[]"#]],
    );
}

#[test]
fn measure_indented_ident() {
    check(
        parse,
        "measure q[2];",
        &expect![[r#"
        Stmt [0-13]
            StmtKind: MeasureStmt [0-13]: MeasureExpr [0-7]: GateOperand IndexedIdent [8-12]: Ident [8-9] "q"[
            IndexElement:
                IndexSetItem Expr [10-11]: Lit: Int(2)]"#]],
    );
}

#[test]
fn measure_hardware_qubit() {
    check(
        parse,
        "measure $42;",
        &expect![[r#"
        Stmt [0-12]
            StmtKind: MeasureStmt [0-12]: MeasureExpr [0-7]: GateOperand HardwareQubit [8-11]: 42"#]],
    );
}

#[test]
fn measure_arrow_into_ident() {
    check(
        parse,
        "measure q -> a;",
        &expect![[r#"
            Stmt [0-15]
                StmtKind: MeasureStmt [0-15]: MeasureExpr [0-7]: GateOperand IndexedIdent [8-9]: Ident [8-9] "q"[], IndexedIdent [13-14]: Ident [13-14] "a"[]"#]],
    );
}

#[test]
fn measure_arrow_into_indented_ident() {
    check(
        parse,
        "measure q -> a[1];",
        &expect![[r#"
            Stmt [0-18]
                StmtKind: MeasureStmt [0-18]: MeasureExpr [0-7]: GateOperand IndexedIdent [8-9]: Ident [8-9] "q"[], IndexedIdent [13-17]: Ident [13-14] "a"[
                IndexElement:
                    IndexSetItem Expr [15-16]: Lit: Int(1)]"#]],
    );
}
