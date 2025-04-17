// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_stmt_kind;

#[test]
fn with_no_init_expr() {
    check_stmt_kind(
        "qreg a;",
        &expect![[r#"
            QubitDeclaration [0-7]:
                symbol_id: 8"#]],
    );
}

#[test]
fn array_with_no_init_expr() {
    check_stmt_kind(
        "qreg a[3];",
        &expect![[r#"
            QubitArrayDeclaration [0-10]:
                symbol_id: 8
                size: 3
                size_span: [7-8]"#]],
    );
}
