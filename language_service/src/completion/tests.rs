// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};

use super::{get_completions, CompletionItem};
use crate::test_utils::{
    compile_notebook_with_fake_stdlib, compile_with_fake_stdlib, get_source_and_marker_offsets,
};

fn expect_completions(source_with_cursor: &str, completions_to_check: &[&str], expect: &Expect) {
    let (source, cursor_offset, _) = get_source_and_marker_offsets(source_with_cursor);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual_completions = get_completions(&compilation, "<source>", cursor_offset[0]);
    let checked_completions: Vec<Option<&CompletionItem>> = completions_to_check
        .iter()
        .map(|comp| {
            actual_completions
                .items
                .iter()
                .find(|item| item.label == **comp)
        })
        .collect();

    expect.assert_debug_eq(&checked_completions);
}

fn expect_notebook_completions(
    cells: &[(&str, &str)],
    completions_to_check: &[&str],
    expect: &Expect,
) {
    let (mut cell_uri, mut offset) = (None, None);
    let cells = cells
        .iter()
        .map(|c| {
            let (source, cursor_offsets, _) = get_source_and_marker_offsets(c.1);
            if !cursor_offsets.is_empty() {
                assert!(
                    cell_uri.replace(c.0).is_none(),
                    "only one cell can have a cursor marker"
                );
                assert!(
                    offset.replace(cursor_offsets[0]).is_none(),
                    "only one cell can have a cursor marker"
                );
            }
            (c.0, source)
        })
        .collect::<Vec<_>>();
    let compilation = compile_notebook_with_fake_stdlib(cells.iter().map(|c| (c.0, c.1.as_str())));
    let actual_completions = get_completions(
        &compilation,
        cell_uri.expect("input should have a cursor marker"),
        offset.expect("input string should have a cursor marker"),
    );
    let checked_completions: Vec<Option<&CompletionItem>> = completions_to_check
        .iter()
        .map(|comp| {
            actual_completions
                .items
                .iter()
                .find(|item| item.label == **comp)
        })
        .collect();

    expect.assert_debug_eq(&checked_completions);
}

#[test]
fn in_block_contains_std_functions() {
    expect_completions(
        r#"
    namespace Test {
        operation Test() : Unit {
            ↘
        }
    }"#,
        &["Fake", "FakeWithParam", "FakeCtlAdj"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0600Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                (
                                    Span {
                                        start: 30,
                                        end: 30,
                                    },
                                    "open FakeStdLib;\n    ",
                                ),
                            ],
                        ),
                    },
                ),
                Some(
                    CompletionItem {
                        label: "FakeWithParam",
                        kind: Function,
                        sort_text: Some(
                            "0600FakeWithParam",
                        ),
                        detail: Some(
                            "operation FakeWithParam(x : Int) : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                (
                                    Span {
                                        start: 30,
                                        end: 30,
                                    },
                                    "open FakeStdLib;\n    ",
                                ),
                            ],
                        ),
                    },
                ),
                Some(
                    CompletionItem {
                        label: "FakeCtlAdj",
                        kind: Function,
                        sort_text: Some(
                            "0600FakeCtlAdj",
                        ),
                        detail: Some(
                            "operation FakeCtlAdj() : Unit is Adj + Ctl",
                        ),
                        additional_text_edits: Some(
                            [
                                (
                                    Span {
                                        start: 30,
                                        end: 30,
                                    },
                                    "open FakeStdLib;\n    ",
                                ),
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn in_block_no_auto_open() {
    expect_completions(
        r#"
    namespace Test {
        open FakeStdLib;
        operation Test() : Unit {
            ↘
        }
    }"#,
        &["Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0600Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn in_block_with_alias() {
    expect_completions(
        r#"
    namespace Test {
        open FakeStdLib as Alias;
        operation Test() : Unit {
            ↘
        }
    }"#,
        &["Alias.Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Alias.Fake",
                        kind: Function,
                        sort_text: Some(
                            "0600Alias.Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn in_block_from_other_namespace() {
    expect_completions(
        r#"
    namespace Test {
        operation Test() : Unit {
            ↘
        }
    }
    namespace Other {
        operation Foo() : Unit {}
    }"#,
        &["Foo"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Foo",
                        kind: Function,
                        sort_text: Some(
                            "0500Foo",
                        ),
                        detail: Some(
                            "operation Foo() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                (
                                    Span {
                                        start: 30,
                                        end: 30,
                                    },
                                    "open Other;\n    ",
                                ),
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[ignore = "nested callables are not currently supported for completions"]
#[test]
fn in_block_nested_op() {
    expect_completions(
        r#"
    namespace Test {
        operation Test() : Unit {
            operation Foo() : Unit {}
            ↘
        }
    }"#,
        &["Foo"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Foo",
                        kind: Function,
                        sort_text: Some(
                            "0500Foo",
                        ),
                        detail: Some(
                            "operation Foo() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn in_block_hidden_nested_op() {
    expect_completions(
        r#"
    namespace Test {
        operation Test() : Unit {
            ↘
        }
        operation Foo() : Unit {
            operation Bar() : Unit {}
        }
    }"#,
        &["Bar"],
        &expect![[r#"
            [
                None,
            ]
        "#]],
    );
}

#[test]
fn in_namespace_contains_open() {
    expect_completions(
        r#"
    namespace Test {
        ↘
        operation Test() : Unit {
        }
    }"#,
        &["open"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "open",
                        kind: Keyword,
                        sort_text: Some(
                            "0102open",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn top_level_contains_namespace() {
    expect_completions(
        r#"
        namespace Test {}
        ↘
        "#,
        &["namespace"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "namespace",
                        kind: Keyword,
                        sort_text: Some(
                            "0101namespace",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn attributes() {
    expect_completions(
        r#"
        namespace Test {
            ↘
        }
        "#,
        &["@EntryPoint()"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "@EntryPoint()",
                        kind: Property,
                        sort_text: Some(
                            "0201@EntryPoint()",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn notebook_top_level() {
    expect_notebook_completions(
        &[(
            "cell1",
            r#"operation Foo() : Unit {}
            ↘
        "#,
        )],
        &["operation", "namespace", "let", "Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "operation",
                        kind: Keyword,
                        sort_text: Some(
                            "0101operation",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "namespace",
                        kind: Keyword,
                        sort_text: Some(
                            "1201namespace",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "let",
                        kind: Keyword,
                        sort_text: Some(
                            "0201let",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0700Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                (
                                    Span {
                                        start: 0,
                                        end: 0,
                                    },
                                    "open FakeStdLib;\n    ",
                                ),
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn notebook_top_level_global() {
    expect_notebook_completions(
        &[(
            "cell1",
            r#"operation Foo() : Unit {}
            ↘
        "#,
        )],
        &["Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0700Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                (
                                    Span {
                                        start: 0,
                                        end: 0,
                                    },
                                    "open FakeStdLib;\n    ",
                                ),
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn notebook_top_level_namespace_already_open_for_global() {
    expect_notebook_completions(
        &[(
            "cell1",
            r#"
            open FakeStdLib;
            operation Foo() : Unit {}
            ↘
        "#,
        )],
        &["Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0700Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn notebook_block() {
    expect_notebook_completions(
        &[(
            "cell1",
            r#"operation Foo() : Unit {
                ↘
            }
        "#,
        )],
        &["Fake", "let"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0600Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                (
                                    Span {
                                        start: 0,
                                        end: 0,
                                    },
                                    "open FakeStdLib;\n    ",
                                ),
                            ],
                        ),
                    },
                ),
                Some(
                    CompletionItem {
                        label: "let",
                        kind: Keyword,
                        sort_text: Some(
                            "0101let",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn notebook_wtf() {
    expect_notebook_completions(
        &[
            (
                "vscode-notebook-cell:/c%3A/src/qsharp/pip/samples/sample.ipynb#W3sZmlsZQ%3D%3D",
                "        \r\n\r\noperation Main() : Result {\r\n    ↘\r\n    use q = Qubit();\r\n    X(q);\r\n    Microsoft.Quantum.Diagnostics.DumpMachine();\r\n    let r = M(q);\r\n    Message($\"The result of the measurement is {r}\");\r\n    Reset(q);\r\n    r\r\n}\r\n\r\nMain()",
            ),
            (
                "vscode-notebook-cell:/c%3A/src/qsharp/pip/samples/sample.ipynb#X21sZmlsZQ%3D%3D",
                "        \n\noperation Bar() : Unit {\n    use q = Qubit(); \n    Microsoft.Quantum.Diagnostics.DumpMachine(); \n    X(q);\n} \n    \nBar()",
            ),
            (
                "vscode-notebook-cell:/c%3A/src/qsharp/pip/samples/sample.ipynb#X23sZmlsZQ%3D%3D",
                "        \n\nopen Microsoft.Quantum.Diagnostics;\n\noperation Main() : Unit {\n    Message(\"Generating random bit... \");\n    for i in 0..400000 {\n        use q = Qubit();\n        H(q);\n        let r = M(q);\n        if (i % 100000) == 0 {\n            DumpMachine();\n            Message($\"Result: {r}\");\n        }\n        Reset(q);\n    }\n}\n\nMain()",
            ),
            (
                "vscode-notebook-cell:/c%3A/src/qsharp/pip/samples/sample.ipynb#X25sZmlsZQ%3D%3D",
                "        \n\noperation RandomBit() : Result {\n    use q = Qubit();\n    H(q);\n    let res = M(q);\n    Reset(q);\n    return res;\n}",
            ),
            (
                "vscode-notebook-cell:/c%3A/src/qsharp/pip/samples/sample.ipynb#X41sZmlsZQ%3D%3D",
                "        \n\noperation Bad() : Unit {\n    use q = Qubit();\n    H(q);\n    let res = M(q);\n    if (res == One) {\n        // Do something bad, sometimes\n        use q2 = Qubit();\n        X(q2);\n    }\n}",
            ),
        ],
        &["let"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "let",
                        kind: Keyword,
                        sort_text: Some(
                            "0101let",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}
