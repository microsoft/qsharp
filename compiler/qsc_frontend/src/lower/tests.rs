// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compile::{self, compile, PackageStore, SourceMap};
use expect_test::{expect, Expect};
use indoc::indoc;

fn check(input: &str, expect: &Expect) {
    let sources = SourceMap::new([("test".into(), input.into())], None);
    let unit = compile(&PackageStore::new(compile::core()), &[], sources);

    let lower_errors: Vec<_> = unit
        .errors
        .into_iter()
        .filter_map(try_into_lower_error)
        .collect();

    expect.assert_debug_eq(&lower_errors);
}

fn try_into_lower_error(error: compile::Error) -> Option<super::Error> {
    if let compile::ErrorKind::Lower(error) = error.0 {
        Some(error)
    } else {
        None
    }
}

#[test]
fn test_entrypoint_attr_allowed() {
    check(
        indoc! {"
            namespace input {
                @EntryPoint()
                operation Foo() : Unit {
                    body ... {}
                }
            }
        "},
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn test_entrypoint_attr_wrong_args() {
    check(
        indoc! {r#"
            namespace input {
                @EntryPoint("Bar")
                operation Foo() : Unit {
                    body ... {}
                }
            }
        "#},
        &expect![[r#"
            [
                InvalidAttrArgs(
                    "()",
                    Span {
                        lo: 33,
                        hi: 40,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_unknown_attr() {
    check(
        indoc! {"
            namespace input {
                @Bar()
                operation Foo() : Unit {
                    body ... {}
                }
            }
        "},
        &expect![[r#"
            [
                UnknownAttr(
                    "Bar",
                    Span {
                        lo: 23,
                        hi: 26,
                    },
                ),
            ]
        "#]],
    );
}
