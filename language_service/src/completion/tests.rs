// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use expect_test::{expect, Expect};

use super::{get_completions, CompletionItem};
use crate::{
    protocol::CompletionList,
    test_utils::{
        compile_notebook_with_markers, compile_project_with_markers, compile_with_markers,
    },
    Encoding,
};
use indoc::indoc;

fn check(source_with_cursor: &str, completions_to_check: &[&str], expect: &Expect) {
    let (compilation, cursor_position, _) = compile_with_markers(source_with_cursor, true);
    let actual_completions =
        get_completions(&compilation, "<source>", cursor_position, Encoding::Utf8);
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

fn check_with_stdlib(source_with_cursor: &str, completions_to_check: &[&str], expect: &Expect) {
    let (compilation, cursor_position, _) = compile_with_markers(source_with_cursor, false);
    let actual_completions =
        get_completions(&compilation, "<source>", cursor_position, Encoding::Utf8);
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

fn check_project(
    sources_with_markers: &[(&str, &str)],
    completions_to_check: &[&str],
    expect: &Expect,
) {
    let (compilation, cursor_uri, cursor_position, _) =
        compile_project_with_markers(sources_with_markers, true);
    let actual_completions =
        get_completions(&compilation, &cursor_uri, cursor_position, Encoding::Utf8);
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
    assert_no_duplicates(actual_completions);
}

fn check_notebook(
    cells_with_markers: &[(&str, &str)],
    completions_to_check: &[&str],
    expect: &Expect,
) {
    let (compilation, cell_uri, cursor_position, _) =
        compile_notebook_with_markers(cells_with_markers);
    let actual_completions =
        get_completions(&compilation, &cell_uri, cursor_position, Encoding::Utf8);
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
    assert_no_duplicates(actual_completions);
}

fn assert_no_duplicates(mut actual_completions: CompletionList) {
    actual_completions
        .items
        .sort_by_key(|item| item.label.clone());
    let mut dups: Vec<&CompletionItem> = vec![];
    let mut last: Option<&CompletionItem> = None;
    for completion in &actual_completions.items {
        if let Some(last) = last.take() {
            if last.label == completion.label {
                dups.push(last);
                dups.push(completion);
            }
        }
        last.replace(completion);
    }

    assert!(dups.is_empty(), "duplicate completions found: {dups:#?}");
}

#[test]
fn ignore_unstable_namespace() {
    check(
        r#"
        namespace Test {
            open ↘
        }"#,
        &["FakeStdLib", "Microsoft.Quantum.Unstable"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "FakeStdLib",
                        kind: Module,
                        sort_text: Some(
                            "1101FakeStdLib",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                None,
            ]
        "#]],
    );
}

#[test]
fn ignore_unstable_callable() {
    check(
        r#"
        namespace Test {
            import Microsoft.Quantum.Unstable.*;
            operation Foo() : Unit {
                ↘
            }
        }"#,
        &["Fake", "UnstableFake"],
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
                                TextEdit {
                                    new_text: "import FakeStdLib.Fake;\n            ",
                                    range: Range {
                                        start: Position {
                                            line: 2,
                                            column: 12,
                                        },
                                        end: Position {
                                            line: 2,
                                            column: 12,
                                        },
                                    },
                                },
                            ],
                        ),
                    },
                ),
                None,
            ]
        "#]],
    );
}

#[test]
fn in_block_contains_std_functions_from_open_namespace() {
    check(
        r#"
    namespace Test {
        open FakeStdLib;
        operation Foo() : Unit {
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
                            "0700Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "FakeWithParam",
                        kind: Function,
                        sort_text: Some(
                            "0700FakeWithParam",
                        ),
                        detail: Some(
                            "operation FakeWithParam(x : Int) : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "FakeCtlAdj",
                        kind: Function,
                        sort_text: Some(
                            "0700FakeCtlAdj",
                        ),
                        detail: Some(
                            "operation FakeCtlAdj() : Unit is Adj + Ctl",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[allow(clippy::too_many_lines)]
#[test]
fn in_block_contains_std_functions() {
    check(
        indoc! {r#"
    namespace Test {
        operation Foo() : Unit {
            ↘
        }
    }"#},
        &["Fake", "FakeWithParam", "FakeCtlAdj"],
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
                                TextEdit {
                                    new_text: "import FakeStdLib.Fake;\n    ",
                                    range: Range {
                                        start: Position {
                                            line: 1,
                                            column: 4,
                                        },
                                        end: Position {
                                            line: 1,
                                            column: 4,
                                        },
                                    },
                                },
                            ],
                        ),
                    },
                ),
                Some(
                    CompletionItem {
                        label: "FakeWithParam",
                        kind: Function,
                        sort_text: Some(
                            "0700FakeWithParam",
                        ),
                        detail: Some(
                            "operation FakeWithParam(x : Int) : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import FakeStdLib.FakeWithParam;\n    ",
                                    range: Range {
                                        start: Position {
                                            line: 1,
                                            column: 4,
                                        },
                                        end: Position {
                                            line: 1,
                                            column: 4,
                                        },
                                    },
                                },
                            ],
                        ),
                    },
                ),
                Some(
                    CompletionItem {
                        label: "FakeCtlAdj",
                        kind: Function,
                        sort_text: Some(
                            "0700FakeCtlAdj",
                        ),
                        detail: Some(
                            "operation FakeCtlAdj() : Unit is Adj + Ctl",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import FakeStdLib.FakeCtlAdj;\n    ",
                                    range: Range {
                                        start: Position {
                                            line: 1,
                                            column: 4,
                                        },
                                        end: Position {
                                            line: 1,
                                            column: 4,
                                        },
                                    },
                                },
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[ignore = "need to implement newtypes"]
#[test]
fn in_block_contains_newtypes() {
    check(
        r#"
    namespace Test {
        newtype Custom;
        operation Foo() : Unit {
            ↘
        }
    }"#,
        &["Custom", "Udt"],
        &expect![[r#"
            [
                some_valid_completion,
                some_valid_completion,
            ]
        "#]],
    );
}

#[ignore = "need more error recovery in parser to narrow down context in parameter list"]
#[test]
fn types_only_in_signature() {
    check(
        r#"
    namespace Test {
        operation Foo(foo: ↘) : Unit {
        }
        operation Bar() : Unit {
        }
    }"#,
        &["Int", "String", "Bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Int",
                        kind: Interface,
                        sort_text: Some(
                            "0102Int",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "String",
                        kind: Interface,
                        sort_text: Some(
                            "0110String",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                None,
            ]
        "#]],
    );
}

#[test]
fn in_block_no_auto_open() {
    check(
        indoc! {r#"
    namespace Test {
        open FakeStdLib;
        operation Foo() : Unit {
            ↘
        }
    }"#},
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
fn in_block_with_alias() {
    check(
        indoc! {r#"
    namespace Test {
        open FakeStdLib as Alias;
        operation Foo() : Unit {
            ↘
        }
    }"#},
        &["Alias.Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Alias.Fake",
                        kind: Function,
                        sort_text: Some(
                            "0700Alias.Fake",
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
    check(
        indoc! {r#"
    namespace Test {
        operation Bar() : Unit {
            ↘
        }
        export Bar;
    }
    namespace Other {
        operation Foo() : Unit {}
        export Foo;
    }"#},
        &["Foo"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Foo",
                        kind: Function,
                        sort_text: Some(
                            "0600Foo",
                        ),
                        detail: Some(
                            "operation Foo() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import Other.Foo;\n    ",
                                    range: Range {
                                        start: Position {
                                            line: 1,
                                            column: 4,
                                        },
                                        end: Position {
                                            line: 1,
                                            column: 4,
                                        },
                                    },
                                },
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn auto_open_multiple_files() {
    check_project(
        &[
            (
                "foo.qs",
                indoc! {r#"namespace Foo { operation FooOperation() : Unit {} export FooOperation; }
                "#},
            ),
            (
                "bar.qs",
                indoc! {r#"namespace Bar { operation BarOperation() : Unit { ↘ } export BarOperation; }
                "#},
            ),
        ],
        &["FooOperation"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "FooOperation",
                        kind: Function,
                        sort_text: Some(
                            "0600FooOperation",
                        ),
                        detail: Some(
                            "operation FooOperation() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import Foo.FooOperation;\n ",
                                    range: Range {
                                        start: Position {
                                            line: 0,
                                            column: 16,
                                        },
                                        end: Position {
                                            line: 0,
                                            column: 16,
                                        },
                                    },
                                },
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn in_block_nested_op() {
    check(
        indoc! {r#"
    namespace Test {
        operation Bar() : Unit {
            operation Foo() : Unit {}
            ↘
        }
    }"#},
        &["Foo"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Foo",
                        kind: Function,
                        sort_text: Some(
                            "0100Foo",
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
    check(
        indoc! {r#"
    namespace Test {
        operation Baz() : Unit {
            ↘
        }
        operation Foo() : Unit {
            operation Bar() : Unit {}
        }
    }"#},
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
    check(
        indoc! {r#"
    namespace Test {
        ↘
        operation Foo() : Unit {
        }
    }"#},
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
    check(
        indoc! {r#"
        namespace Test {}
        ↘
        "#},
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
    check(
        indoc! {r#"
        namespace Test {
            ↘
        }
        "#},
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
fn stdlib_udt() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                ↘
            }
        "#},
        &["TakesUdt"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "TakesUdt",
                        kind: Function,
                        sort_text: Some(
                            "0700TakesUdt",
                        ),
                        detail: Some(
                            "function TakesUdt(input : Udt) : Udt",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import FakeStdLib.TakesUdt;\n    ",
                                    range: Range {
                                        start: Position {
                                            line: 1,
                                            column: 4,
                                        },
                                        end: Position {
                                            line: 1,
                                            column: 4,
                                        },
                                    },
                                },
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn notebook_top_level() {
    check_notebook(
        &[(
            "cell1",
            indoc! {r#"operation Foo() : Unit {}
            ↘
        "#},
        )],
        &["operation", "namespace", "let", "Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "operation",
                        kind: Keyword,
                        sort_text: Some(
                            "0201operation",
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
                            "1301namespace",
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
                            "0301let",
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
                            "0800Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import FakeStdLib.Fake;\n",
                                    range: Range {
                                        start: Position {
                                            line: 0,
                                            column: 0,
                                        },
                                        end: Position {
                                            line: 0,
                                            column: 0,
                                        },
                                    },
                                },
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
    check_notebook(
        &[(
            "cell1",
            indoc! {r#"operation Foo() : Unit {}
            ↘
        "#},
        )],
        &["Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0800Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import FakeStdLib.Fake;\n",
                                    range: Range {
                                        start: Position {
                                            line: 0,
                                            column: 0,
                                        },
                                        end: Position {
                                            line: 0,
                                            column: 0,
                                        },
                                    },
                                },
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
    check_notebook(
        &[(
            "cell1",
            indoc! {r#"
            open FakeStdLib;
            operation Foo() : Unit {}
            ↘
        "#},
        )],
        &["Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0800Fake",
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
    check_notebook(
        &[(
            "cell1",
            indoc! {r#"operation Foo() : Unit {
                ↘
            }
        "#},
        )],
        &["Fake", "let"],
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
                                TextEdit {
                                    new_text: "import FakeStdLib.Fake;\n",
                                    range: Range {
                                        start: Position {
                                            line: 0,
                                            column: 0,
                                        },
                                        end: Position {
                                            line: 0,
                                            column: 0,
                                        },
                                    },
                                },
                            ],
                        ),
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
            ]
        "#]],
    );
}

#[test]
fn notebook_auto_open_start_of_cell_empty() {
    check_notebook(
        &[
            (
                "cell1",
                indoc! {"
                    //qsharp
                    namespace Foo { operation Bar() : Unit {} }"
                },
            ),
            (
                "cell2",
                indoc! {"
                    //qsharp
                    ↘"
                },
            ),
        ],
        &["Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0800Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import FakeStdLib.Fake;\n",
                                    range: Range {
                                        start: Position {
                                            line: 1,
                                            column: 0,
                                        },
                                        end: Position {
                                            line: 1,
                                            column: 0,
                                        },
                                    },
                                },
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn notebook_auto_open_start_of_cell() {
    check_notebook(
        &[
            (
                "cell1",
                indoc! {"
                    //qsharp
                    namespace Foo { operation Bar() : Unit {} }"
                },
            ),
            (
                "cell2",
                indoc! {r#"
                    //qsharp
                    Message("hi")
                    ↘"#
                },
            ),
        ],
        &["Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0800Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import FakeStdLib.Fake;\n",
                                    range: Range {
                                        start: Position {
                                            line: 1,
                                            column: 0,
                                        },
                                        end: Position {
                                            line: 1,
                                            column: 0,
                                        },
                                    },
                                },
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn local_vars() {
    check(
        r#"
    namespace Test {
        operation Foo() : Unit {
            let bar = 3;
            ↘
            let foo = 3;
        }
    }"#,
        &["foo", "bar"],
        &expect![[r#"
            [
                None,
                Some(
                    CompletionItem {
                        label: "bar",
                        kind: Variable,
                        sort_text: Some(
                            "0100bar",
                        ),
                        detail: Some(
                            "bar : Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn local_items() {
    check(
        r#"
    namespace Test {
        operation Baz() : Unit {
            operation Foo() : Unit {}
            ↘
            operation Bar() : Unit {}
            newtype Custom = String;
        }
    }"#,
        &["Foo", "Bar", "Custom"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Foo",
                        kind: Function,
                        sort_text: Some(
                            "0100Foo",
                        ),
                        detail: Some(
                            "operation Foo() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Bar",
                        kind: Function,
                        sort_text: Some(
                            "0100Bar",
                        ),
                        detail: Some(
                            "operation Bar() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Custom",
                        kind: Interface,
                        sort_text: Some(
                            "0100Custom",
                        ),
                        detail: Some(
                            "newtype Custom = String",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn type_params() {
    check(
        r#"
    namespace Test {
        operation Foo<'T>() : Unit {
            ↘
        }
    }"#,
        &["'T", "Bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "'T",
                        kind: TypeParameter,
                        sort_text: Some(
                            "0100'T",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                None,
            ]
        "#]],
    );
}

#[test]
fn scoped_local_vars() {
    check(
        r#"
    namespace Test {
        operation Foo() : Unit {
            {
                let foo = 3;
            }
            ↘
        }
    }"#,
        &["foo"],
        &expect![[r#"
            [
                None,
            ]
        "#]],
    );
}

#[test]
fn callable_params() {
    check(
        r#"
    namespace Test {
        newtype Custom = String;
        operation Foo(foo: Int, bar: Custom) : Unit {
            {
                ↘
            }
        }
    }"#,
        &["foo", "bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "foo",
                        kind: Variable,
                        sort_text: Some(
                            "0100foo",
                        ),
                        detail: Some(
                            "foo : Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "bar",
                        kind: Variable,
                        sort_text: Some(
                            "0100bar",
                        ),
                        detail: Some(
                            "bar : Custom",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn local_var_in_callable_parent_scope() {
    check(
        r#"
    namespace Test {
        operation Foo(foo: Int) : Unit {
            let bar = 3;
            operation Bar() : Unit {
                let baz = 3;
                ↘
            }
        }
    }"#,
        &["foo", "bar", "baz"],
        &expect![[r#"
            [
                None,
                None,
                Some(
                    CompletionItem {
                        label: "baz",
                        kind: Variable,
                        sort_text: Some(
                            "0100baz",
                        ),
                        detail: Some(
                            "baz : Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
#[ignore = "completion list ignores shadowing rules for open statements"]
fn local_var_and_open_shadowing_rules() {
    check(
        r#"
        namespace Foo {
            operation Bar() : Unit {
            }
        }

        namespace Test {
            operation Main() : Unit {
                let Bar = 3;
                Bar;
                {
                    // open Foo should shadow the local Bar declaration
                    open Foo;
                    Bar;
                    ↘
                }

            }
        }"#,
        &["Bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Bar",
                        kind: Function,
                        sort_text: Some(
                            "0700Bar",
                        ),
                        detail: Some(
                            "operation Bar() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

// no additional text edits for Foo or Bar because FooNs is already glob imported
#[test]
fn dont_import_if_already_glob_imported() {
    check(
        r#"
        namespace FooNs {
            operation Foo() : Unit {
            }
            operation Bar() : Unit { }
        }

        namespace Test {
            import FooNs.*;
            operation Main() : Unit {
                ↘
            }
        }"#,
        &["Foo", "Bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Foo",
                        kind: Function,
                        sort_text: Some(
                            "0600Foo",
                        ),
                        detail: Some(
                            "operation Foo() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Bar",
                        kind: Function,
                        sort_text: Some(
                            "0600Bar",
                        ),
                        detail: Some(
                            "operation Bar() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

// no additional text edits for Foo because Foo is directly imported,
// but additional text edits for Bar because Bar is not directly imported
#[test]
fn dont_import_if_already_directly_imported() {
    check(
        r#"
        namespace FooNs {
            operation Foo() : Unit { }
            operation Bar() : Unit { }
        }

        namespace Test {
            import FooNs.Foo;
            operation Main() : Unit {
                ↘
            }
        }"#,
        &["Foo", "Bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Foo",
                        kind: Function,
                        sort_text: Some(
                            "0100Foo",
                        ),
                        detail: Some(
                            "operation Foo() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Bar",
                        kind: Function,
                        sort_text: Some(
                            "0600Bar",
                        ),
                        detail: Some(
                            "operation Bar() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import FooNs.Bar;\n            ",
                                    range: Range {
                                        start: Position {
                                            line: 7,
                                            column: 12,
                                        },
                                        end: Position {
                                            line: 7,
                                            column: 12,
                                        },
                                    },
                                },
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn auto_import_from_qir_runtime() {
    check_with_stdlib(
        r#"
        namespace Test {
            operation Main() : Unit {
               AllocateQubitA↘
            }
        }"#,
        &["AllocateQubitArray"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "AllocateQubitArray",
                        kind: Function,
                        sort_text: Some(
                            "0800AllocateQubitArray",
                        ),
                        detail: Some(
                            "operation AllocateQubitArray(size : Int) : Qubit[]",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import QIR.Runtime.AllocateQubitArray;\n            ",
                                    range: Range {
                                        start: Position {
                                            line: 2,
                                            column: 12,
                                        },
                                        end: Position {
                                            line: 2,
                                            column: 12,
                                        },
                                    },
                                },
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn dont_generate_import_for_core_prelude() {
    check_with_stdlib(
        r#"
        namespace Test {
            operation Main() : Unit {
               Length↘
            }
        }"#,
        &["Length"],
        // additional text edits should be None because Length is in the core prelude
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Length",
                        kind: Function,
                        sort_text: Some(
                            "0800Length",
                        ),
                        detail: Some(
                            "function Length<'T>(a : 'T[]) : Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn dont_generate_import_for_stdlib_prelude() {
    check_with_stdlib(
        r#"
        namespace Test {
            operation Main() : Unit {
               MResetZ↘
            }
        }"#,
        &["MResetZ"],
        // additional text edits should be None because MResetZ is in Std.Measurement, which
        // is in the prelude.
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "MResetZ",
                        kind: Function,
                        sort_text: Some(
                            "0700MResetZ",
                        ),
                        detail: Some(
                            "operation MResetZ(target : Qubit) : Result",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn callable_from_same_file() {
    check(
        r#"
        namespace Test {
            function MyCallable() : Unit {}
            operation Main() : Unit {
               MyCall↘
            }
        }"#,
        &["MyCallable"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "MyCallable",
                        kind: Function,
                        sort_text: Some(
                            "0600MyCallable",
                        ),
                        detail: Some(
                            "function MyCallable() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}
