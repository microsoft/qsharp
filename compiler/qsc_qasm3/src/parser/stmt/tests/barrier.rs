// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::stmt::parse;
use crate::parser::tests::check;
use expect_test::expect;

#[test]
fn barrier() {
    check(
        parse,
        "barrier r, q[0], $2;",
        &expect![[r#"
            Stmt [0-20]:
                annotations: <empty>
                kind: Barrier [0-20]:
                    operands: 
                        GateOperand IndexedIdent [8-9]:
                            name: Ident [8-9] "r"
                            indices: <empty>
                        GateOperand IndexedIdent [11-15]:
                            name: Ident [11-12] "q"
                            indices: 
                                IndexElement: 
                                    IndexSetItem Expr [13-14]: Lit: Int(0)
                        GateOperand HardwareQubit [17-19]: 2"#]],
    );
}

#[test]
fn barrier_no_args() {
    check(
        parse,
        "barrier;",
        &expect![[r#"
            Stmt [0-8]:
                annotations: <empty>
                kind: Barrier [0-8]:
                    operands: <empty>"#]],
    );
}
