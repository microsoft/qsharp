// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::get_code_lenses;
use crate::{
    test_utils::{
        compile_notebook_with_fake_stdlib, compile_with_fake_stdlib_and_markers_no_cursor,
    },
    Encoding,
};
use expect_test::{expect, Expect};

fn check(source_with_markers: &str, expect: &Expect) {
    let (compilation, expected_code_lens_ranges) =
        compile_with_fake_stdlib_and_markers_no_cursor(source_with_markers, true);
    let mut actual_code_lenses = get_code_lenses(&compilation, "<source>", Encoding::Utf8);

    for expected_range in &expected_code_lens_ranges {
        assert!(
            actual_code_lenses
                .iter()
                .any(|cl| cl.range == *expected_range),
            "expected range not found in actual code lenses: {expected_range:?}"
        );
    }

    for actual_range in actual_code_lenses.iter().map(|cl| cl.range) {
        assert!(
            expected_code_lens_ranges.iter().any(|r| r == &actual_range),
            "got code lens for unexpected range: {actual_range:?}"
        );
    }

    let actual = expected_code_lens_ranges
        .into_iter()
        .enumerate()
        .map(move |(i, r)| {
            actual_code_lenses.sort_by_key(|cl| cl.range == r);
            let partition_point = actual_code_lenses.partition_point(|cl| cl.range != r);
            let for_this_range = actual_code_lenses.drain(partition_point..);
            (i, for_this_range.map(|cl| cl.command).collect::<Vec<_>>())
        })
        .collect::<Vec<_>>();
    expect.assert_debug_eq(&actual);
}

#[test]
fn one_entrypoint() {
    check(
        r#"
        namespace Test {
            @EntryPoint()
            ◉operation Test() : Unit{
            }◉
        }"#,
        &expect![[r#"
            [
                (
                    0,
                    [
                        Run,
                        Histogram,
                        Estimate,
                        Debug,
                        Circuit(
                            None,
                        ),
                    ],
                ),
            ]
        "#]],
    );
}

#[test]
fn two_entrypoints() {
    check(
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit{
            }

            @EntryPoint()
            operation Foo() : Unit{
            }
        }"#,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn main_function() {
    check(
        r#"
        namespace Test {
            ◉operation Main() : Unit {
            }◉

            operation Foo() : Unit{
            }
        }"#,
        &expect![[r#"
            [
                (
                    0,
                    [
                        Run,
                        Histogram,
                        Estimate,
                        Debug,
                        Circuit(
                            None,
                        ),
                    ],
                ),
            ]
        "#]],
    );
}

#[test]
fn no_entrypoint_code_lens_in_notebook() {
    let compilation = compile_notebook_with_fake_stdlib(
        [(
            "cell1",
            "@EntryPoint()
            operation Main() : Unit {}",
        )]
        .into_iter(),
    );

    let lenses = get_code_lenses(&compilation, "cell1", Encoding::Utf8);
    assert!(
        lenses.is_empty(),
        "entrypoint code lenses should not be present in notebooks"
    );
}

#[test]
fn qubit_operation_circuit() {
    check(
        r#"
        namespace Test {
            ◉operation Foo(q: Qubit) : Unit {
            }◉
        }"#,
        &expect![[r#"
            [
                (
                    0,
                    [
                        Circuit(
                            Some(
                                OperationInfo {
                                    operation: "Test.Foo",
                                    total_num_qubits: 1,
                                },
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
    );
}

#[test]
fn qubit_arrays_operation_circuit() {
    check(
        r#"
        namespace Test {
            ◉operation Foo(q: Qubit, q1: Qubit[], q2: Qubit[][]) : Unit {
            }◉
        }"#,
        &expect![[r#"
            [
                (
                    0,
                    [
                        Circuit(
                            Some(
                                OperationInfo {
                                    operation: "Test.Foo",
                                    total_num_qubits: 7,
                                },
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
    );
}
