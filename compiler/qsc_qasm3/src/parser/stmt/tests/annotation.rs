// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::parser::tests::check;

use crate::parser::stmt::parse_annotation;

#[test]
fn annotation() {
    check(
        parse_annotation,
        "@a.b.d 23",
        &expect!["Annotation [0-9]: (a.b.d, 23)"],
    );
}

#[test]
fn annotation_ident_only() {
    check(
        parse_annotation,
        "@a.b.d",
        &expect!["Annotation [0-6]: (a.b.d)"],
    );
}

#[test]
fn annotation_missing_ident() {
    check(
        parse_annotation,
        "@",
        &expect![[r#"
            Annotation [0-1]: ()

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
