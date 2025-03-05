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
                    target: IndexedIdent [13-14]:
                        name: Ident [13-14] "a"
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
                    target: IndexedIdent [13-17]:
                        name: Ident [13-14] "a"
                        indices: 
                            IndexElement: 
                                IndexSetItem Expr [15-16]: Lit: Int(1)"#]],
    );
}
