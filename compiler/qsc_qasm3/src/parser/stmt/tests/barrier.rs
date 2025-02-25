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
        Stmt [0-20]
            StmtKind: Barrier [0-20]: [
            GateOperand IndexedIdent [8-9]: Ident [8-9] "r"[]
            GateOperand IndexedIdent [11-15]: Ident [11-12] "q"[
            IndexElement:
                IndexSetItem Expr [13-14]: Lit: Int(0)]
            GateOperand HardwareQubit [17-19]: 2]"#]],
    );
}

#[test]
fn barrier_no_args() {
    check(
        parse,
        "barrier;",
        &expect![[r#"
        Stmt [0-8]
            StmtKind: Barrier [0-8]: []"#]],
    );
}
