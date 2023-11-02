// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::get_references;
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};
use expect_test::{expect, Expect};
use indoc::indoc;

fn check(source_with_markers: &str, expect: &Expect) {
    let (source, cursor_offsets, _) = get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_references(&compilation, "<source>", cursor_offsets[0]);
    expect.assert_debug_eq(&actual);
}

#[test]
fn std_callable() {
    check(
        indoc! {r#"
        namespace Test {
            open FakeStdLib;
            operation Foo() : Unit {
                Fa↘ke();
                let x = 3;
                Fake();
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "qsharp-library-source:<std>",
                    offset: 49,
                },
                Location {
                    source: "qsharp-library-source:<std>",
                    offset: 571,
                },
                Location {
                    source: "<source>",
                    offset: 75,
                },
                Location {
                    source: "<source>",
                    offset: 110,
                },
            ]
        "#]],
    );
}
