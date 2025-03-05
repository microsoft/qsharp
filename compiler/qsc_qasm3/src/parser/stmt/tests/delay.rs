// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::stmt::parse;
use crate::parser::tests::check;
use expect_test::expect;

#[test]
fn delay() {
    check(
        parse,
        "delay[a] q[0], q[1];",
        &expect![[r#"
            Stmt [0-20]:
                annotations: <empty>
                kind: DelayStmt [0-20]:
                    duration: Expr [6-7]: Ident [6-7] "a"
                    qubits:
                        IndexedIdent [9-13]:
                            name: Ident [9-10] "q"
                            indices:
                                IndexSet [11-12]:
                                    values:
                                        Expr [11-12]: Lit: Int(0)
                        IndexedIdent [15-19]:
                            name: Ident [15-16] "q"
                            indices:
                                IndexSet [17-18]:
                                    values:
                                        Expr [17-18]: Lit: Int(1)"#]],
    );
}
