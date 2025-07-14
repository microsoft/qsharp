// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::stmt::parse_annotation;
use crate::parser::tests::check;
use expect_test::expect;

#[test]
fn annotation() {
    check(
        parse_annotation,
        "@a.b.d 23",
        &expect![[r#"
            Annotation [0-9]:
                identifier: "a.b.d"
                value: "23""#]],
    );
}

#[test]
fn annotation_ident_only() {
    check(
        parse_annotation,
        "@a.b.d",
        &expect![[r#"
            Annotation [0-6]:
                identifier: "a.b.d"
                value: <none>"#]],
    );
}

#[test]
fn annotation_missing_ident() {
    check(
        parse_annotation,
        "@",
        &expect![[r#"
            Annotation [0-1]:
                identifier: ""
                value: <none>

            [
                Error(
                    Rule(
                        "annotation missing identifier",
                        Annotation,
                        Span {
                            lo: 0,
                            hi: 1,
                        },
                    ),
                ),
            ]"#]],
    );
}
