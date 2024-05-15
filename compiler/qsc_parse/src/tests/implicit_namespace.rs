// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;
use qsc_data_structures::language_features::LanguageFeatures;

#[test]
fn explicit_namespace_overrides_implicit() {
    let result = format!(
        "{:#?}",
        crate::namespaces(
            "namespace Explicit {}",
            Some("code/src/Implicit.qs"),
            LanguageFeatures::default()
        )
    );
    expect![[r#"
        (
            [
                Namespace {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 21,
                    },
                    doc: "",
                    name: Idents(
                        [
                            Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 10,
                                    hi: 18,
                                },
                                name: "Explicit",
                            },
                        ],
                    ),
                    items: [],
                },
            ],
            [],
        )"#]]
    .assert_eq(&result);
}

#[test]
fn reject_bad_namespace_name_1() {
    let result = format!(
        "{:#?}",
        crate::namespaces(
            "operation Main() : Unit {}",
            Some("code/src/Foo-Bar.qs"),
            LanguageFeatures::default()
        )
    );
    expect![[r"
        (
            [],
            [
                Error(
                    InvalidFileName(
                        Span {
                            lo: 0,
                            hi: 26,
                        },
                    ),
                ),
            ],
        )"]]
    .assert_eq(&result);
}

#[test]
fn reject_bad_namespace_name_2() {
    let result = format!(
        "{:#?}",
        crate::namespaces(
            "operation Main() : Unit {}",
            Some("code/src/123Bar.qs"),
            LanguageFeatures::default()
        )
    );
    expect![[r"
        (
            [],
            [
                Error(
                    InvalidFileName(
                        Span {
                            lo: 0,
                            hi: 26,
                        },
                    ),
                ),
            ],
        )"]]
    .assert_eq(&result);
}
