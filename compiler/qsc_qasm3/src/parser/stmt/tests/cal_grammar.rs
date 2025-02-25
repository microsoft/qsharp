// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::stmt::parse;
use crate::parser::tests::check;
use expect_test::expect;

#[test]
fn defcalgrammar() {
    check(
        parse,
        r#"defcalgrammar "openpulse";"#,
        &expect![[r#"
        Stmt [0-26]
            StmtKind: CalibrationGrammarStmt [0-26]: openpulse"#]],
    );
}
