// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::parser::tests::check;

use crate::parser::stmt::parse;

#[test]
fn pragma_decl() {
    check(
        parse,
        "pragma a.b.d 23",
        &expect![[r#"
            Stmt [0-15]:
                annotations: <empty>
                kind: Pragma [0-15]:
                    identifier: "a.b.d"
                    value: "23"
                    value_span: [13-15]"#]],
    );
}

#[test]
fn pragma_decl_complex_value_stops_at_newline() {
    check(
        parse,
        "pragma a.b.d 23 or \"value\" or 'other' or // comment\n 42",
        &expect![[r#"
            Stmt [0-51]:
                annotations: <empty>
                kind: Pragma [0-51]:
                    identifier: "a.b.d"
                    value: "23 or "value" or 'other' or // comment"
                    value_span: [13-51]"#]],
    );
}

#[test]
fn pragma_decl_ident_only() {
    check(
        parse,
        "pragma a.b.d",
        &expect![[r#"
            Stmt [0-12]:
                annotations: <empty>
                kind: Pragma [0-12]:
                    identifier: "a.b.d"
                    value: <none>
                    value_span: <none>"#]],
    );
}

#[test]
fn pragma_decl_missing_ident() {
    check(
        parse,
        "pragma ",
        &expect![[r#"
            Stmt [0-7]:
                annotations: <empty>
                kind: Pragma [0-7]:
                    identifier: "Err"
                    value: <none>
                    value_span: <none>

            [
                Error(
                    Rule(
                        "pragma missing identifier",
                        Pragma,
                        Span {
                            lo: 0,
                            hi: 7,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn legacy_pragma_decl() {
    check(
        parse,
        "#pragma a.b.d 23",
        &expect![[r#"
            Stmt [0-16]:
                annotations: <empty>
                kind: Pragma [0-16]:
                    identifier: "a.b.d"
                    value: "23"
                    value_span: [14-16]"#]],
    );
}

#[test]
fn legacy_pragma_decl_ident_only() {
    check(
        parse,
        "#pragma a.b.d",
        &expect![[r#"
            Stmt [0-13]:
                annotations: <empty>
                kind: Pragma [0-13]:
                    identifier: "a.b.d"
                    value: <none>
                    value_span: <none>"#]],
    );
}

#[test]
fn legacy_pragma_ws_after_hash() {
    check(
        parse,
        "# pragma a.b.d",
        &expect![[r#"
            Stmt [2-14]:
                annotations: <empty>
                kind: Pragma [2-14]:
                    identifier: "a.b.d"
                    value: <none>
                    value_span: <none>

            [
                Error(
                    Lex(
                        Incomplete(
                            Ident,
                            Identifier,
                            Whitespace,
                            Span {
                                lo: 1,
                                hi: 2,
                            },
                        ),
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn legacy_pragma_decl_missing_ident() {
    check(
        parse,
        "#pragma ",
        &expect![[r#"
            Stmt [0-8]:
                annotations: <empty>
                kind: Pragma [0-8]:
                    identifier: "Err"
                    value: <none>
                    value_span: <none>

            [
                Error(
                    Rule(
                        "pragma missing identifier",
                        Pragma,
                        Span {
                            lo: 0,
                            hi: 8,
                        },
                    ),
                ),
            ]"#]],
    );
}
