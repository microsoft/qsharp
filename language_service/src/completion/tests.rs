// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{get_completions, CompletionItem};
use crate::{
    protocol::CompletionList,
    test_utils::{
        compile_notebook_with_markers, compile_project_with_markers,
        compile_with_dependency_with_markers, compile_with_markers,
    },
    Encoding,
};
use expect_test::{expect, Expect};
use indoc::indoc;

mod class_completions;

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

fn check_with_dependency(
    source_with_cursor: &str,
    dependency_alias: &str,
    dependency_source: &str,
    completions_to_check: &[&str],
    expect: &Expect,
) {
    let (compilation, cursor_uri, cursor_position, _) = compile_with_dependency_with_markers(
        &[("<source>", source_with_cursor)],
        dependency_alias,
        &[("<dependency_source>", dependency_source)],
    );
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

fn check_no_completions(source_with_cursor: &str) {
    let (compilation, cursor_position, _) = compile_with_markers(source_with_cursor, true);
    let actual_completions =
        get_completions(&compilation, "<source>", cursor_position, Encoding::Utf8);
    assert_eq!(actual_completions.items, Vec::default());
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
                            "0100FakeStdLib",
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
                            "0401Fake",
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
fn ignore_internal_callable() {
    check(
        r#"
        namespace Test {
            internal operation Foo() : Unit {}
            operation Bar() : Unit {
                ↘
            }
        }

        namespace Test {
            internal operation Baz() : Unit {}
        }"#,
        &["Fake", "Foo", "Baz", "Hidden"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0401Fake",
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
                Some(
                    CompletionItem {
                        label: "Foo",
                        kind: Function,
                        sort_text: Some(
                            "0300Foo",
                        ),
                        detail: Some(
                            "operation Foo() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Baz",
                        kind: Function,
                        sort_text: Some(
                            "0300Baz",
                        ),
                        detail: Some(
                            "operation Baz() : Unit",
                        ),
                        additional_text_edits: None,
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
                            "0400Fake",
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
                            "0400FakeWithParam",
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
                            "0400FakeCtlAdj",
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
                            "0401Fake",
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
                            "0401FakeWithParam",
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
                            "0401FakeCtlAdj",
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

#[test]
fn in_block_contains_newtypes() {
    check(
        r#"
    namespace Test {
        newtype Custom = String;
        operation Foo() : Unit {
            let x: ↘
        }
    }"#,
        &["Custom", "Udt"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Custom",
                        kind: Interface,
                        sort_text: Some(
                            "0400Custom",
                        ),
                        detail: Some(
                            "newtype Custom = String",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0501Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import FakeStdLib.Udt;\n        ",
                                    range: Range {
                                        start: Position {
                                            line: 2,
                                            column: 8,
                                        },
                                        end: Position {
                                            line: 2,
                                            column: 8,
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
                            "0200Int",
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
                            "0200String",
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
                            "0400Fake",
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
                            "0400Alias.Fake",
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
fn members_of_aliased_namespace() {
    check(
        indoc! {r#"
    namespace Test {
        open FakeStdLib as Alias;
        operation Foo() : Unit {
            Alias.↘
        }
    }"#},
        &["Fake", "Alias.Fake", "Library", "Alias.Library", "Foo"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0300Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
                Some(
                    CompletionItem {
                        label: "Library",
                        kind: Module,
                        sort_text: Some(
                            "0600Library",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                None,
                None,
            ]
        "#]],
    );
}

#[test]
fn aliased_exact_import() {
    check(
        indoc! {r#"
    namespace Test {
        import FakeStdLib.Fake as Alias;
        operation Foo() : Unit {
            ↘
        }
    }"#},
        &["Fake", "Alias.Fake", "Alias"],
        &expect![[r#"
            [
                None,
                None,
                Some(
                    CompletionItem {
                        label: "Alias",
                        kind: Function,
                        sort_text: Some(
                            "0400Alias",
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
fn open_from_dependency() {
    check_with_dependency(
        r"
        namespace Test {
            open MyDep.Dependency;
            operation Foo() : Unit {
                ↘
            }
        }",
        "MyDep",
        "namespace Dependency { operation Baz() : Unit {} export Baz; }",
        &["Baz"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Baz",
                        kind: Function,
                        sort_text: Some(
                            "0400Baz",
                        ),
                        detail: Some(
                            "operation Baz() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn open_with_alias_from_dependency() {
    check_with_dependency(
        r"
        namespace Test {
            open MyDep.Dependency as Alias;
            open SamePackage as Alias1;
            operation Foo() : Unit {
                ↘
            }
        }
        namespace SamePackage { operation Bar() : Unit {} }",
        "MyDep",
        "namespace Dependency { operation Baz() : Unit {} export Baz; }",
        &["Alias.Baz", "Baz", "Alias1.Bar", "Bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Alias.Baz",
                        kind: Function,
                        sort_text: Some(
                            "0400Alias.Baz",
                        ),
                        detail: Some(
                            "operation Baz() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
                Some(
                    CompletionItem {
                        label: "Alias1.Bar",
                        kind: Function,
                        sort_text: Some(
                            "0300Alias1.Bar",
                        ),
                        detail: Some(
                            "operation Bar() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
            ]
        "#]],
    );
}

#[test]
fn import_ns_with_alias_from_dependency() {
    check_with_dependency(
        r"
        namespace Test {
            import MyDep.Dependency as Alias;
            import SamePackage as Alias1;
            operation Foo() : Unit {
                ↘
            }
        }
        namespace SamePackage { operation Bar() : Unit {} }",
        "MyDep",
        "namespace Dependency { operation Baz() : Unit {} export Baz; }",
        &["Alias.Baz", "Baz", "Alias1.Bar", "Bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Alias.Baz",
                        kind: Function,
                        sort_text: Some(
                            "0400Alias.Baz",
                        ),
                        detail: Some(
                            "operation Baz() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
                Some(
                    CompletionItem {
                        label: "Alias1.Bar",
                        kind: Function,
                        sort_text: Some(
                            "0300Alias1.Bar",
                        ),
                        detail: Some(
                            "operation Bar() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
            ]
        "#]],
    );
}

#[test]
fn exact_import_from_dependency() {
    check_with_dependency(
        r"
        namespace Test {
            import MyDep.Dependency.Baz;
            operation Foo() : Unit {
                ↘
            }
        }",
        "MyDep",
        "namespace Dependency { operation Baz() : Unit {} export Baz; }",
        &["Baz"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Baz",
                        kind: Function,
                        sort_text: Some(
                            "0400Baz",
                        ),
                        detail: Some(
                            "operation Baz() : Unit",
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
                            "0301Foo",
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
                            "0301FooOperation",
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
                            "0000open",
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
                            "0000namespace",
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
            @↘
        }
        "#},
        &["EntryPoint"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "EntryPoint",
                        kind: Interface,
                        sort_text: Some(
                            "0000EntryPoint",
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
                            "0401TakesUdt",
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
                            "0000operation",
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
                            "0000namespace",
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
                            "0000let",
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
                            "0401Fake",
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
                            "0401Fake",
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
                            "0400Fake",
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
                            "0401Fake",
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
                            "0000let",
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
                            "0401Fake",
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
                            "0401Fake",
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
fn notebook_last_expr() {
    check_notebook(
        &[(
            "cell1",
            indoc! {"
                    //qsharp
                    function Foo() : Unit {}
                    3 + ↘"
            },
        )],
        &["Foo", "Fake"],
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
                            "function Foo() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0401Fake",
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
            let x: ↘
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
                            "0300Foo",
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
                            "0300Bar",
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

// expect an auto-import for `Foo.Bar`, separate from the preexisting glob import `Foo.Bar.*`
#[test]
fn glob_import_item_with_same_name() {
    check(
        r#"
        namespace Foo {
            operation Bar() : Unit {
            }
        }

        namespace Foo.Bar {
        }

        namespace Baz {
            import Foo.Bar.*;
            operation Main(): Unit {
                ↘
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
                            "0301Bar",
                        ),
                        detail: Some(
                            "operation Bar() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import Foo.Bar;\n            ",
                                    range: Range {
                                        start: Position {
                                            line: 10,
                                            column: 12,
                                        },
                                        end: Position {
                                            line: 10,
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
                            "0300Foo",
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
                            "0301Bar",
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
                            "0201AllocateQubitArray",
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
                            "0200Length",
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
                            "0400MResetZ",
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
                            "0300MyCallable",
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

#[test]
fn member_completion() {
    check(
        r#"
        namespace Test {
            function MyCallable() : Unit {}
        }

        namespace Main {
            operation Main() : Unit {
               Test.↘
            }
        }

        "#,
        &["MyCallable"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "MyCallable",
                        kind: Function,
                        sort_text: Some(
                            "0200MyCallable",
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

#[test]
fn member_completion_in_imported_namespace() {
    check(
        r#"
        namespace Test.Foo {
            function MyCallable() : Unit {}
        }

        namespace Test.Foo.Bar {
            function CallableInBar()  : Unit {}
        }

        namespace Main {
            open Test;
            operation Main() : Unit {
               Foo.↘
            }
        }

        "#,
        &["MyCallable", "Bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "MyCallable",
                        kind: Function,
                        sort_text: Some(
                            "0200MyCallable",
                        ),
                        detail: Some(
                            "function MyCallable() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Bar",
                        kind: Module,
                        sort_text: Some(
                            "0500Bar",
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
fn namespace_completion() {
    check(
        r#"
        namespace Test.Foo {
            function MyCallable() : Unit {}
        }

        namespace Main {
            operation Main() : Unit {
               Test.↘
            }
        }

        "#,
        &["Foo"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Foo",
                        kind: Module,
                        sort_text: Some(
                            "0500Foo",
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
fn nested_namespace() {
    check(
        r#"
        namespace Test.Foo {
            function MyCallable() : Unit {}
        }

        namespace Test {
            function MyCallable2() : Unit {
                Foo.↘
            }
        }"#,
        &["MyCallable", "MyCallable2"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "MyCallable",
                        kind: Function,
                        sort_text: Some(
                            "0200MyCallable",
                        ),
                        detail: Some(
                            "function MyCallable() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
            ]
        "#]],
    );
}

#[test]
fn std_member() {
    check(
        r#"
        namespace Test {
            function MyCallable2() : Unit {
                FakeStdLib.↘
            }
        }"#,
        &["Fake", "Library"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0300Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Library",
                        kind: Module,
                        sort_text: Some(
                            "0600Library",
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
fn open_namespace() {
    check(
        r#"
        namespace Test {
            open FakeStdLib.↘;
        }"#,
        &["Fake", "Library"],
        &expect![[r#"
            [
                None,
                Some(
                    CompletionItem {
                        label: "Library",
                        kind: Module,
                        sort_text: Some(
                            "0300Library",
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
fn open_namespace_no_semi() {
    check(
        r#"
        namespace Test {
            open FakeStdLib.↘
        }"#,
        &["Fake", "Library"],
        &expect![[r#"
            [
                None,
                Some(
                    CompletionItem {
                        label: "Library",
                        kind: Module,
                        sort_text: Some(
                            "0300Library",
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
fn open_namespace_no_semi_followed_by_decl() {
    check(
        r#"
        namespace Test {
            open FakeStdLib.↘
            operation Foo() : Unit {}
        }"#,
        &["Fake", "Library"],
        &expect![[r#"
            [
                None,
                Some(
                    CompletionItem {
                        label: "Library",
                        kind: Module,
                        sort_text: Some(
                            "0300Library",
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
fn open_namespace_partial_path_part() {
    check(
        r#"
        namespace Test {
            open FakeStdLib.↘F
            operation Foo() : Unit {}
        }"#,
        &["Fake", "Library"],
        &expect![[r#"
            [
                None,
                Some(
                    CompletionItem {
                        label: "Library",
                        kind: Module,
                        sort_text: Some(
                            "0300Library",
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
fn let_stmt_type() {
    check(
        r#"
        namespace Test {
            function Main() : Unit {
                let x: ↘
            }
        }"#,
        &["Udt", "Qubit", "Int", "Main", "FakeWithParam"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0501Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import FakeStdLib.Udt;\n            ",
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
                Some(
                    CompletionItem {
                        label: "Qubit",
                        kind: Interface,
                        sort_text: Some(
                            "0200Qubit",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Int",
                        kind: Interface,
                        sort_text: Some(
                            "0200Int",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                None,
                None,
            ]
        "#]],
    );
}

#[test]
fn let_stmt_type_before_next_stmt() {
    check(
        r#"
        namespace Test {
            function Main() : Unit {
                use q = Qubit();
                let x: ↘
                H(q);
            }
        }"#,
        &["Udt", "Qubit", "Int", "Main", "FakeWithParam"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0501Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import FakeStdLib.Udt;\n            ",
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
                Some(
                    CompletionItem {
                        label: "Qubit",
                        kind: Interface,
                        sort_text: Some(
                            "0200Qubit",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Int",
                        kind: Interface,
                        sort_text: Some(
                            "0200Int",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                None,
                None,
            ]
        "#]],
    );
}

#[test]
fn type_position_namespace() {
    check(
        r#"
        namespace Test {
            function Main() : Unit {
                let x: FakeStdLib.↘ ;
            }
        }"#,
        &["Udt", "Qubit", "Int", "Main", "FakeWithParam"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0300Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
                None,
                None,
                None,
            ]
        "#]],
    );
}

#[test]
fn udt_base_type_part() {
    check(
        r#"
        namespace Test {
            newtype Foo = FakeStdLib.↘
        }"#,
        &["Udt", "Qubit", "FakeWithParam"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0300Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
                None,
            ]
        "#]],
    );
}

#[test]
fn struct_init() {
    check(
        r#"
        namespace Test {
            function Main() : Unit {
                let x = new ↘ ;
            }
        }"#,
        &["Udt", "Qubit", "Int", "Main", "FakeWithParam"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0301Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import FakeStdLib.Udt;\n            ",
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
                None,
                None,
                None,
            ]
        "#]],
    );
}

#[test]
fn struct_init_path_part() {
    check(
        r#"
        namespace Test {
            function Main() : Unit {
                let x = new FakeStdLib.↘ ;
            }
        }"#,
        &["Udt", "Qubit", "Int", "Main", "FakeWithParam"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0300Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
                None,
                None,
                None,
            ]
        "#]],
    );
}

#[test]
fn struct_init_path_part_in_field_assigment() {
    check(
        r#"
        namespace Test {
            function Main() : Unit {
                let x = new FakeStdLib.Udt { x = FakeStdLib.↘ } ;
            }
        }"#,
        &["Udt", "Qubit", "FakeWithParam"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0300Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
                Some(
                    CompletionItem {
                        label: "FakeWithParam",
                        kind: Function,
                        sort_text: Some(
                            "0300FakeWithParam",
                        ),
                        detail: Some(
                            "operation FakeWithParam(x : Int) : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn export_path() {
    check(
        r#"
        namespace Test {
            export ↘ ;
            function Main() : Unit {
            }
        }"#,
        &["Udt", "Qubit", "Int", "Main", "FakeWithParam", "FakeStdLib"],
        &expect![[r#"
            [
                None,
                None,
                None,
                Some(
                    CompletionItem {
                        label: "Main",
                        kind: Function,
                        sort_text: Some(
                            "0200Main",
                        ),
                        detail: Some(
                            "function Main() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
                Some(
                    CompletionItem {
                        label: "FakeStdLib",
                        kind: Module,
                        sort_text: Some(
                            "0400FakeStdLib",
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
fn export_path_part() {
    check(
        r#"
        namespace Test {
            export FakeStdLib.↘ ;
            function Main() : Unit {
            }
        }"#,
        &["Udt", "Qubit", "Int", "Main", "FakeWithParam", "FakeStdLib"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0300Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
                None,
                None,
                Some(
                    CompletionItem {
                        label: "FakeWithParam",
                        kind: Function,
                        sort_text: Some(
                            "0300FakeWithParam",
                        ),
                        detail: Some(
                            "operation FakeWithParam(x : Int) : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
            ]
        "#]],
    );
}

#[test]
fn partially_typed_name() {
    check(
        r#"
        namespace Test {
            export Fo↘
            function Foo() : Unit {
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
                            "0200Foo",
                        ),
                        detail: Some(
                            "function Foo() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn from_dependency_main() {
    check_with_dependency(
        "namespace Test { function Foo() : Unit { ↘ } }",
        "MyDep",
        "namespace Main { export MainFunc; function MainFunc() : Unit {} }
        namespace Other { export OtherFunc; function OtherFunc() : Unit {} }
        ",
        &["MainFunc", "OtherFunc"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "MainFunc",
                        kind: Function,
                        sort_text: Some(
                            "0401MainFunc",
                        ),
                        detail: Some(
                            "function MainFunc() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import MyDep.MainFunc;\n ",
                                    range: Range {
                                        start: Position {
                                            line: 0,
                                            column: 17,
                                        },
                                        end: Position {
                                            line: 0,
                                            column: 17,
                                        },
                                    },
                                },
                            ],
                        ),
                    },
                ),
                Some(
                    CompletionItem {
                        label: "OtherFunc",
                        kind: Function,
                        sort_text: Some(
                            "0401OtherFunc",
                        ),
                        detail: Some(
                            "function OtherFunc() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                TextEdit {
                                    new_text: "import MyDep.Other.OtherFunc;\n ",
                                    range: Range {
                                        start: Position {
                                            line: 0,
                                            column: 17,
                                        },
                                        end: Position {
                                            line: 0,
                                            column: 17,
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
fn package_aliases() {
    check_with_dependency(
        "namespace Test { function Foo() : Unit { ↘ } }",
        "MyDep",
        "namespace Main { export MainFunc; function MainFunc() : Unit {} }",
        &["MyDep", "Main"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "MyDep",
                        kind: Module,
                        sort_text: Some(
                            "0600MyDep",
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
fn package_alias_members() {
    check_with_dependency(
        "namespace Test { function Foo() : Unit { MyDep.↘ } }",
        "MyDep",
        "namespace Main { export MainFunc; function MainFunc() : Unit {} }
        namespace Other { export OtherFunc; function OtherFunc() : Unit {} }
        namespace Other.Sub { export OtherFunc; function OtherFunc() : Unit {} }
        ",
        &["Main", "Other", "MainFunc", "Other.Sub", "Sub"],
        &expect![[r#"
            [
                None,
                Some(
                    CompletionItem {
                        label: "Other",
                        kind: Module,
                        sort_text: Some(
                            "0700Other",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "MainFunc",
                        kind: Function,
                        sort_text: Some(
                            "0300MainFunc",
                        ),
                        detail: Some(
                            "function MainFunc() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
                None,
            ]
        "#]],
    );
}

#[test]
fn dependency_namespace_members() {
    check_with_dependency(
        "namespace Test { function Foo() : Unit { MyDep.Other.↘ } }",
        "MyDep",
        "namespace Main { export MainFunc; function MainFunc() : Unit {} }
        namespace Other { export OtherFunc; function OtherFunc() : Unit {} }
        namespace Other.Sub { export OtherFunc; function OtherFunc() : Unit {} }
        ",
        &["Main", "Other", "MainFunc", "Other.Sub", "Sub", "OtherFunc"],
        &expect![[r#"
            [
                None,
                None,
                None,
                None,
                Some(
                    CompletionItem {
                        label: "Sub",
                        kind: Module,
                        sort_text: Some(
                            "0700Sub",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "OtherFunc",
                        kind: Function,
                        sort_text: Some(
                            "0300OtherFunc",
                        ),
                        detail: Some(
                            "function OtherFunc() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn package_alias_members_in_open() {
    check_with_dependency(
        "namespace Test { open MyDep.↘  }",
        "MyDep",
        "namespace Main { export MainFunc; function MainFunc() : Unit {} }
        namespace Other { export OtherFunc; function OtherFunc() : Unit {} }
        namespace Other.Sub { export OtherFunc; function OtherFunc() : Unit {} }
        ",
        &["Main", "Other", "MainFunc", "Other.Sub", "Sub"],
        &expect![[r#"
            [
                None,
                Some(
                    CompletionItem {
                        label: "Other",
                        kind: Module,
                        sort_text: Some(
                            "0300Other",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                None,
                None,
                None,
            ]
        "#]],
    );
}

#[test]
fn member_completion_in_imported_namespace_from_dependency() {
    check_with_dependency(
        "namespace Main {
            open MyDep.Test;
            operation Main() : Unit {
               Foo.↘
            }
        }",
        "MyDep",
        "
        namespace Test.Foo {
            function CallableInFoo() : Unit {}
            export CallableInFoo;
        }

        namespace Test.Foo.Bar {
            function CallableInBar()  : Unit {}
            export CallableInBar;
        }
        ",
        &["CallableInFoo", "Bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "CallableInFoo",
                        kind: Function,
                        sort_text: Some(
                            "0300CallableInFoo",
                        ),
                        detail: Some(
                            "function CallableInFoo() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Bar",
                        kind: Module,
                        sort_text: Some(
                            "0700Bar",
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
fn aliased_namespace_in_dependency() {
    check_with_dependency(
        "namespace Main {
            open MyDep.Test.Foo as Alias;
            operation Main() : Unit {
               Alias.↘
            }
        }",
        "MyDep",
        "
        namespace Test.Foo {
            function CallableInFoo() : Unit {}
            export CallableInFoo;
        }

        namespace Test.Foo.Bar {
            function CallableInBar()  : Unit {}
            export CallableInBar;
        }
        ",
        &["CallableInFoo", "Bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "CallableInFoo",
                        kind: Function,
                        sort_text: Some(
                            "0300CallableInFoo",
                        ),
                        detail: Some(
                            "function CallableInFoo() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Bar",
                        kind: Module,
                        sort_text: Some(
                            "0700Bar",
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
fn open_does_not_match_pkg_alias() {
    check_with_dependency(
        "namespace Main {
            open Test.Foo as Alias;
            operation Main() : Unit {
               Alias.↘
            }
        }",
        "MyDep",
        "
        namespace Test.Foo {
            function CallableInFoo() : Unit {}
            export CallableInFoo;
        }

        namespace Test.Foo.Bar {
            function CallableInBar()  : Unit {}
            export CallableInBar;
        }
        ",
        &["CallableInFoo", "Bar"],
        &expect![[r#"
            [
                None,
                None,
            ]
        "#]],
    );
}

#[test]
fn field_access_expr() {
    check(
        "namespace Test {
        struct Foo {
            bar : Int,
        }

        function Main() : Unit {
            (new Foo { bar = 3 }).↘
        }
    }",
        &["bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "bar",
                        kind: Field,
                        sort_text: Some(
                            "0100bar",
                        ),
                        detail: Some(
                            "Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn input_type_missing() {
    check(
        "namespace Test { function Foo(x : FakeStdLib.↘ ) : Unit { body intrinsic; } }",
        &["Udt", "Library"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0300Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Library",
                        kind: Module,
                        sort_text: Some(
                            "0600Library",
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
fn notebook_top_level_path_part() {
    check_notebook(
        &[(
            "cell1",
            ("
        FakeStdLib.↘
    "),
        )],
        &["Udt", "Library", "FakeStdLib", "FakeWithParam"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0300Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Library",
                        kind: Module,
                        sort_text: Some(
                            "0600Library",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                None,
                Some(
                    CompletionItem {
                        label: "FakeWithParam",
                        kind: Function,
                        sort_text: Some(
                            "0300FakeWithParam",
                        ),
                        detail: Some(
                            "operation FakeWithParam(x : Int) : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn field_access_path() {
    check(
        "namespace Test {
        struct Foo {
            bar : Int,
        }

        function Main() : Unit {
            let foo = new Foo { bar = 3 };
            foo.↘
        }
    }",
        &["bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "bar",
                        kind: Field,
                        sort_text: Some(
                            "0100bar",
                        ),
                        detail: Some(
                            "Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn notebook_top_level_path_part_in_type() {
    check_notebook(
        &[(
            "cell1",
            ("
        let x : FakeStdLib.↘
    "),
        )],
        &["Udt", "Library", "FakeStdLib", "FakeWithParam"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0300Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Library",
                        kind: Module,
                        sort_text: Some(
                            "0600Library",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                None,
                None,
            ]
        "#]],
    );
}

#[test]
fn prefix_ops() {
    check(
        "namespace Test { function Main() : Unit { let x = ↘ ; } }",
        &["and", "or", "not", "Adjoint"],
        &expect![[r#"
            [
                None,
                None,
                Some(
                    CompletionItem {
                        label: "not",
                        kind: Keyword,
                        sort_text: Some(
                            "0000not",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Adjoint",
                        kind: Keyword,
                        sort_text: Some(
                            "0000Adjoint",
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
fn binary_ops() {
    check(
        "namespace Test { function Main() : Unit { let x = 1 ↘ ; } }",
        &["and", "or", "not"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "and",
                        kind: Keyword,
                        sort_text: Some(
                            "0000and",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "or",
                        kind: Keyword,
                        sort_text: Some(
                            "0000or",
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
fn array_size() {
    check(
        "namespace Test { function Main() : Unit { let x = [0, ↘] ; } }",
        &["size"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "size",
                        kind: Keyword,
                        sort_text: Some(
                            "0000size",
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
fn path_segment_partial_ident_is_keyword() {
    check(
        "namespace Test { import FakeStdLib.struct↘ }",
        &["StructFn"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "StructFn",
                        kind: Interface,
                        sort_text: Some(
                            "0300StructFn",
                        ),
                        detail: Some(
                            "struct StructFn { inner : (Int -> Int) }",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn path_segment_followed_by_wslash() {
    // `w/` is a single token, so it gets tricky
    // to separate out the `w` and treat it as an identifier.
    // We're just not going to worry about doing anything clever here.
    check(
        "namespace Test { import FakeStdLib.w↘/ }",
        &["StructFn"],
        &expect![[r#"
            [
                None,
            ]
        "#]],
    );
}

#[test]
fn path_segment_followed_by_op_token() {
    // Invoking in the middle of a multi-character op token
    // shouldn't break anything.
    check(
        "namespace Test { import FakeStdLib.<↘<< }",
        &["StructFn"],
        &expect![[r#"
            [
                None,
            ]
        "#]],
    );
}

#[test]
fn path_segment_before_glob() {
    check(
        "namespace Test { import FakeStdLib.↘* }",
        &["StructFn"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "StructFn",
                        kind: Interface,
                        sort_text: Some(
                            "0300StructFn",
                        ),
                        detail: Some(
                            "struct StructFn { inner : (Int -> Int) }",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn path_segment_before_glob_with_alias() {
    check(
        "namespace Test { import FakeStdLib.↘* as Alias }",
        &["StructFn"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "StructFn",
                        kind: Interface,
                        sort_text: Some(
                            "0300StructFn",
                        ),
                        detail: Some(
                            "struct StructFn { inner : (Int -> Int) }",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn field_in_initializer() {
    check(
        "namespace Test {
        struct Foo {
            bar : Int,
        }

        function Main() : Unit {
            new Foo { ↘ };
        }
    }",
        &["bar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "bar",
                        kind: Field,
                        sort_text: Some(
                            "0100bar",
                        ),
                        detail: Some(
                            "Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn stdlib_struct_field_init() {
    check(
        "namespace Test {
            import FakeStdLib.FakeStruct as StructAlias;
            function Main() : Unit {
                new StructAlias { ↘ };
            }
        }",
        &["x"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "x",
                        kind: Field,
                        sort_text: Some(
                            "0100x",
                        ),
                        detail: Some(
                            "Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn newtype_named_field() {
    check(
        "namespace Test {
            newtype Foo = (field : Int);
            function Main() : Unit {
                Foo(3).↘
            }
        }",
        &["field"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "field",
                        kind: Field,
                        sort_text: Some(
                            "0100field",
                        ),
                        detail: Some(
                            "Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn field_access_path_chained() {
    check(
        "namespace Test {
            newtype Foo = ( fieldFoo : Int );
            struct Bar { fieldBar : Foo );
            function Main() : Unit {
                let bar = new Bar { fieldBar = Foo(3) };
                bar.fieldBar.↘
            }
        }",
        &["fieldFoo", "fieldBar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "fieldFoo",
                        kind: Field,
                        sort_text: Some(
                            "0100fieldFoo",
                        ),
                        detail: Some(
                            "Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
            ]
        "#]],
    );
}

#[test]
fn field_access_expr_chained() {
    check(
        "namespace Test {
            newtype Foo = ( fieldFoo : Int );
            struct Bar { fieldBar : Foo );
            function Main() : Unit {
                (new Bar { fieldBar = Foo(3) }).fieldBar.↘
            }
        }",
        &["fieldFoo", "fieldBar"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "fieldFoo",
                        kind: Field,
                        sort_text: Some(
                            "0100fieldFoo",
                        ),
                        detail: Some(
                            "Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
            ]
        "#]],
    );
}

#[test]
fn field_assignment_rhs() {
    check(
        "namespace Test {
        struct Foo {
            bar : Int,
        }

        function Main() : Unit {
            let var = 3;
            new Foo { bar = ↘ };
        }
    }",
        &["bar", "var"],
        &expect![[r#"
            [
                None,
                Some(
                    CompletionItem {
                        label: "var",
                        kind: Variable,
                        sort_text: Some(
                            "0100var",
                        ),
                        detail: Some(
                            "var : Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn field_access_local_shadows_global() {
    check(
        "namespace Test {
        struct Foo {
            bar : Int,
        }

        function Main() : Unit {
            let FakeStdLib = new Foo { bar = 3 };
            FakeStdLib.↘
        }
    }",
        &["Fake", "bar"],
        &expect![[r#"
            [
                None,
                Some(
                    CompletionItem {
                        label: "bar",
                        kind: Field,
                        sort_text: Some(
                            "0100bar",
                        ),
                        detail: Some(
                            "Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn ty_param_in_signature() {
    check(
        r"namespace Test {
            operation Test<'T>(x: ↘) : Unit {}
        }",
        &["'T", "FakeStdLib"],
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
                Some(
                    CompletionItem {
                        label: "FakeStdLib",
                        kind: Module,
                        sort_text: Some(
                            "0600FakeStdLib",
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
fn ty_param_in_return_type() {
    check(
        r"namespace Test {
            operation Test<'T>(x: 'T) : ↘ {}
        }",
        &["'T", "FakeStdLib"],
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
                Some(
                    CompletionItem {
                        label: "FakeStdLib",
                        kind: Module,
                        sort_text: Some(
                            "0600FakeStdLib",
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
fn path_segment_in_return_type() {
    check(
        r"namespace Test {
            operation Test(x: 'T) : FakeStdLib.↘ {}
        }",
        &["Udt"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Udt",
                        kind: Interface,
                        sort_text: Some(
                            "0300Udt",
                        ),
                        detail: Some(
                            "struct Udt { x : Int, y : Int }",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn return_type_in_partial_callable_signature() {
    check(
        r"namespace Test {
            operation Test<'T>() : ↘
        }",
        &["'T", "FakeStdLib"],
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
                Some(
                    CompletionItem {
                        label: "FakeStdLib",
                        kind: Module,
                        sort_text: Some(
                            "0600FakeStdLib",
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
fn arg_type_in_partial_callable_signature() {
    check(
        r"namespace Test {
            operation Test<'T>(x: ↘)
        }",
        &["'T", "FakeStdLib"],
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
                Some(
                    CompletionItem {
                        label: "FakeStdLib",
                        kind: Module,
                        sort_text: Some(
                            "0600FakeStdLib",
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
fn incomplete_return_type_in_partial_callable_signature() {
    check(
        r"namespace Test {
            operation Test<'T>() : () => ↘
        }",
        &["'T", "FakeStdLib"],
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
                Some(
                    CompletionItem {
                        label: "FakeStdLib",
                        kind: Module,
                        sort_text: Some(
                            "0600FakeStdLib",
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
fn no_path_segment_completion_inside_attr() {
    check_no_completions(
        "namespace Test {

        @Config(FakeStdLib.↘)
        function Main() : Unit {
        }
    }",
    );
}

#[test]
fn no_completion_inside_attr() {
    check_no_completions(
        "namespace Test {

        @Config(↘)
        function Main() : Unit {
        }
    }",
    );
}

#[test]
fn in_comment() {
    check_no_completions(
        "namespace Test {
            import Foo;
            // Hello there ↘
            import Bar;
        }",
    );
}

#[test]
fn in_doc_comment() {
    check_no_completions(
        "namespace Test {
            import Foo;
            /// Hello there ↘
            import Bar;
        }",
    );
}

#[test]
fn in_trailing_comment() {
    check_no_completions(
        "namespace Test {
            import Foo; // Hello there ↘
        }",
    );
}
