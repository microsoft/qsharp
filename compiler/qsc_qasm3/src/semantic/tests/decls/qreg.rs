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
                symbol_id: 6"#]],
    );
}
