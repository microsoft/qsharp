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
            Stmt [0-10]:
                annotations: <empty>
                kind: MeasureStmt [0-10]:
                    measurement: MeasureExpr [0-7]:
                        operand: IndexedIdent [8-9]:
                            name: Ident [8-9] "q"
                            index_span: [0-0]
                            indices: <empty>
                    target: <none>"#]],
    );
}

#[test]
fn measure_indented_ident() {
    check(
        parse,
        "measure q[2];",
        &expect![[r#"
            Stmt [0-13]:
                annotations: <empty>
                kind: MeasureStmt [0-13]:
                    measurement: MeasureExpr [0-7]:
                        operand: IndexedIdent [8-12]:
                            name: Ident [8-9] "q"
                            index_span: [9-12]
                            indices:
                                IndexSet [10-11]:
                                    values:
                                        Expr [10-11]: Lit: Int(2)
                    target: <none>"#]],
    );
}

#[test]
fn measure_hardware_qubit() {
    check(
        parse,
        "measure $42;",
        &expect![[r#"
            Stmt [0-12]:
                annotations: <empty>
                kind: MeasureStmt [0-12]:
                    measurement: MeasureExpr [0-7]:
                        operand: HardwareQubit [8-11]: 42
                    target: <none>"#]],
    );
}

#[test]
fn measure_arrow_into_ident() {
    check(
        parse,
        "measure q -> a;",
        &expect![[r#"
            Stmt [0-15]:
                annotations: <empty>
                kind: MeasureStmt [0-15]:
                    measurement: MeasureExpr [0-7]:
                        operand: IndexedIdent [8-9]:
                            name: Ident [8-9] "q"
                            index_span: [0-0]
                            indices: <empty>
                    target: IndexedIdent [13-14]:
                        name: Ident [13-14] "a"
                        index_span: [0-0]
                        indices: <empty>"#]],
    );
}

#[test]
fn measure_arrow_into_indented_ident() {
    check(
        parse,
        "measure q -> a[1];",
        &expect![[r#"
            Stmt [0-18]:
                annotations: <empty>
                kind: MeasureStmt [0-18]:
                    measurement: MeasureExpr [0-7]:
                        operand: IndexedIdent [8-9]:
                            name: Ident [8-9] "q"
                            index_span: [0-0]
                            indices: <empty>
                    target: IndexedIdent [13-17]:
                        name: Ident [13-14] "a"
                        index_span: [14-17]
                        indices:
                            IndexSet [15-16]:
                                values:
                                    Expr [15-16]: Lit: Int(1)"#]],
    );
}
