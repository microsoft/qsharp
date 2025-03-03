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
        Stmt [0-20]
            StmtKind: DelayInstruction [0-20]: Expr [6-7]: Ident [6-7] "a"
            GateOperand IndexedIdent [9-13]: Ident [9-10] "q"[
            IndexElement:
                IndexSetItem Expr [11-12]: Lit: Int(0)]
            GateOperand IndexedIdent [15-19]: Ident [15-16] "q"[
            IndexElement:
                IndexSetItem Expr [17-18]: Lit: Int(1)]"#]],
    );
}
