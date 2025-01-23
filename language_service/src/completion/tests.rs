// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{get_completions, Compilation, CompletionItem};
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
use qsc::line_column::Position;
use std::fmt::Write;

mod class_completions;

fn do_check(
    compilation: &Compilation,
    cursor_uri: &str,
    cursor_position: Position,
    completions_to_check: &[&str],
    expect: &Expect,
) {
    let actual_completions =
        get_completions(compilation, cursor_uri, cursor_position, Encoding::Utf8);

    let mut checked_completions: Vec<(String, Option<&CompletionItem>)> = completions_to_check
        .iter()
        .map(|comp| {
            (
                (*comp).to_string(),
                actual_completions
                    .items
                    .iter()
                    .find(|item| item.label == **comp),
            )
        })
        .collect();

    // Sort by actual items' sort text
    checked_completions.sort_by_key(|c| {
        c.1.map_or(String::new(), |c| c.sort_text.clone().unwrap_or_default())
    });

    let mut output = String::new();

    let mut first = true;
    for (label, c) in &checked_completions {
        if c.is_none() {
            if first {
                first = false;
                writeln!(output, "not in list: ").unwrap();
            }
            let _ = writeln!(output, "  {label}");
        }
    }

    first = true;

    for (label, c) in &checked_completions {
        if let Some(c) = c {
            if first {
                first = false;
                writeln!(output, "in list (sorted):").unwrap();
            }
            let _ = writeln!(output, "  {label} ({:?})", c.kind,);
            let _ = writeln!(output, "    detail: {:?}", c.detail);

            match &c.additional_text_edits {
                Some(edits) => {
                    let _ = writeln!(output, "    additional_text_edits:");
                    for edit in edits {
                        let _ = writeln!(
                            output,
                            "      [{}:{}-{}:{}] {:?}",
                            edit.range.start.line,
                            edit.range.start.column,
                            edit.range.end.line,
                            edit.range.end.column,
                            edit.new_text,
                        );
                    }
                }
                None => {
                    let _ = writeln!(output, "    additional_text_edits: None");
                }
            }
        }
    }

    expect.assert_eq(&output);
    // TODO: this is too much at the moment
    // assert_no_duplicates(actual_completions);
}

fn check(source_with_cursor: &str, completions_to_check: &[&str], expect: &Expect) {
    let (compilation, cursor_position, _) = compile_with_markers(source_with_cursor, true);

    do_check(
        &compilation,
        "<source>",
        cursor_position,
        completions_to_check,
        expect,
    );
}

fn check_with_stdlib(source_with_cursor: &str, completions_to_check: &[&str], expect: &Expect) {
    let (compilation, cursor_position, _) = compile_with_markers(source_with_cursor, false);

    do_check(
        &compilation,
        "<source>",
        cursor_position,
        completions_to_check,
        expect,
    );
}

fn check_project(
    sources_with_markers: &[(&str, &str)],
    completions_to_check: &[&str],
    expect: &Expect,
) {
    let (compilation, cursor_uri, cursor_position, _) =
        compile_project_with_markers(sources_with_markers, true);

    do_check(
        &compilation,
        &cursor_uri,
        cursor_position,
        completions_to_check,
        expect,
    );
}

fn check_notebook(
    cells_with_markers: &[(&str, &str)],
    completions_to_check: &[&str],
    expect: &Expect,
) {
    let (compilation, cell_uri, cursor_position, _) =
        compile_notebook_with_markers(cells_with_markers);

    do_check(
        &compilation,
        &cell_uri,
        cursor_position,
        completions_to_check,
        expect,
    );
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

    do_check(
        &compilation,
        &cursor_uri,
        cursor_position,
        completions_to_check,
        expect,
    );
}

fn check_no_completions(source_with_cursor: &str) {
    let (compilation, cursor_position, _) = compile_with_markers(source_with_cursor, true);
    let actual_completions =
        get_completions(&compilation, "<source>", cursor_position, Encoding::Utf8);
    assert_eq!(actual_completions.items, Vec::default());
}

#[allow(dead_code)]
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
            not in list: 
              Microsoft.Quantum.Unstable
            in list (sorted):
              FakeStdLib (Module)
                detail: None
                additional_text_edits: None
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
            not in list: 
              UnstableFake
            in list (sorted):
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits:
                  [2:12-2:12] "import FakeStdLib.Fake;\n            "
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
            not in list: 
              Hidden
            in list (sorted):
              Baz (Function)
                detail: Some("operation Baz() : Unit")
                additional_text_edits: None
              Foo (Function)
                detail: Some("operation Foo() : Unit")
                additional_text_edits: None
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits:
                  [2:12-2:12] "import FakeStdLib.Fake;\n            "
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
            in list (sorted):
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits: None
              FakeCtlAdj (Function)
                detail: Some("operation FakeCtlAdj() : Unit is Adj + Ctl")
                additional_text_edits: None
              FakeWithParam (Function)
                detail: Some("operation FakeWithParam(x : Int) : Unit")
                additional_text_edits: None
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
            in list (sorted):
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits:
                  [1:4-1:4] "import FakeStdLib.Fake;\n    "
              FakeCtlAdj (Function)
                detail: Some("operation FakeCtlAdj() : Unit is Adj + Ctl")
                additional_text_edits:
                  [1:4-1:4] "import FakeStdLib.FakeCtlAdj;\n    "
              FakeWithParam (Function)
                detail: Some("operation FakeWithParam(x : Int) : Unit")
                additional_text_edits:
                  [1:4-1:4] "import FakeStdLib.FakeWithParam;\n    "
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
            in list (sorted):
              Custom (Interface)
                detail: Some("newtype Custom = String")
                additional_text_edits: None
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits:
                  [2:8-2:8] "import FakeStdLib.Udt;\n        "
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
            not in list: 
              Bar
            in list (sorted):
              Int (Interface)
                detail: None
                additional_text_edits: None
              String (Interface)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits: None
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
            in list (sorted):
              Alias.Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits: None
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
            not in list: 
              Alias.Fake
              Alias.Library
              Foo
            in list (sorted):
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits: None
              Library (Module)
                detail: None
                additional_text_edits: None
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
            not in list: 
              Fake
              Alias.Fake
            in list (sorted):
              Alias (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits: None
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
            in list (sorted):
              Baz (Function)
                detail: Some("operation Baz() : Unit")
                additional_text_edits: None
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
            not in list: 
              Baz
              Bar
            in list (sorted):
              Alias1.Bar (Function)
                detail: Some("operation Bar() : Unit")
                additional_text_edits: None
              Alias.Baz (Function)
                detail: Some("operation Baz() : Unit")
                additional_text_edits: None
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
            not in list: 
              Baz
              Bar
            in list (sorted):
              Alias1.Bar (Function)
                detail: Some("operation Bar() : Unit")
                additional_text_edits: None
              Alias.Baz (Function)
                detail: Some("operation Baz() : Unit")
                additional_text_edits: None
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
            in list (sorted):
              Baz (Function)
                detail: Some("operation Baz() : Unit")
                additional_text_edits: None
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
            in list (sorted):
              Foo (Function)
                detail: Some("operation Foo() : Unit")
                additional_text_edits:
                  [1:4-1:4] "import Other.Foo;\n    "
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
            in list (sorted):
              FooOperation (Function)
                detail: Some("operation FooOperation() : Unit")
                additional_text_edits:
                  [0:16-0:16] "import Foo.FooOperation;\n "
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
            in list (sorted):
              Foo (Function)
                detail: Some("operation Foo() : Unit")
                additional_text_edits: None
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
            not in list: 
              Bar
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
            in list (sorted):
              open (Keyword)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              namespace (Keyword)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              EntryPoint (Interface)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              TakesUdt (Function)
                detail: Some("function TakesUdt(input : Udt) : Udt")
                additional_text_edits:
                  [1:4-1:4] "import FakeStdLib.TakesUdt;\n    "
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
            in list (sorted):
              let (Keyword)
                detail: None
                additional_text_edits: None
              namespace (Keyword)
                detail: None
                additional_text_edits: None
              operation (Keyword)
                detail: None
                additional_text_edits: None
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits:
                  [0:0-0:0] "import FakeStdLib.Fake;\n"
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
            in list (sorted):
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits:
                  [0:0-0:0] "import FakeStdLib.Fake;\n"
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
            in list (sorted):
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits: None
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
            in list (sorted):
              let (Keyword)
                detail: None
                additional_text_edits: None
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits:
                  [0:0-0:0] "import FakeStdLib.Fake;\n"
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
            in list (sorted):
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits:
                  [1:0-1:0] "import FakeStdLib.Fake;\n"
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
            in list (sorted):
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits:
                  [1:0-1:0] "import FakeStdLib.Fake;\n"
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
            in list (sorted):
              Foo (Function)
                detail: Some("function Foo() : Unit")
                additional_text_edits: None
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits:
                  [1:0-1:0] "import FakeStdLib.Fake;\n"
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
            not in list: 
              foo
            in list (sorted):
              bar (Variable)
                detail: Some("bar : Int")
                additional_text_edits: None
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
            in list (sorted):
              Bar (Function)
                detail: Some("operation Bar() : Unit")
                additional_text_edits: None
              Custom (Interface)
                detail: Some("newtype Custom = String")
                additional_text_edits: None
              Foo (Function)
                detail: Some("operation Foo() : Unit")
                additional_text_edits: None
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
            not in list: 
              Bar
            in list (sorted):
              'T (TypeParameter)
                detail: None
                additional_text_edits: None
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
            not in list: 
              foo
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
            in list (sorted):
              bar (Variable)
                detail: Some("bar : Custom")
                additional_text_edits: None
              foo (Variable)
                detail: Some("foo : Int")
                additional_text_edits: None
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
            not in list: 
              foo
              bar
            in list (sorted):
              baz (Variable)
                detail: Some("baz : Int")
                additional_text_edits: None
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
            in list (sorted):
              Bar (Function)
                detail: Some("operation Bar() : Unit")
                additional_text_edits: None
              Foo (Function)
                detail: Some("operation Foo() : Unit")
                additional_text_edits: None
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
            in list (sorted):
              Bar (Function)
                detail: Some("operation Bar() : Unit")
                additional_text_edits:
                  [10:12-10:12] "import Foo.Bar;\n            "
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
            in list (sorted):
              Foo (Function)
                detail: Some("operation Foo() : Unit")
                additional_text_edits: None
              Bar (Function)
                detail: Some("operation Bar() : Unit")
                additional_text_edits:
                  [7:12-7:12] "import FooNs.Bar;\n            "
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
            in list (sorted):
              AllocateQubitArray (Function)
                detail: Some("operation AllocateQubitArray(size : Int) : Qubit[]")
                additional_text_edits:
                  [2:12-2:12] "import QIR.Runtime.AllocateQubitArray;\n            "
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
            in list (sorted):
              Length (Function)
                detail: Some("function Length<'T>(a : 'T[]) : Int")
                additional_text_edits: None
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
            in list (sorted):
              MResetZ (Function)
                detail: Some("operation MResetZ(target : Qubit) : Result")
                additional_text_edits: None
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
            in list (sorted):
              MyCallable (Function)
                detail: Some("function MyCallable() : Unit")
                additional_text_edits: None
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
            in list (sorted):
              MyCallable (Function)
                detail: Some("function MyCallable() : Unit")
                additional_text_edits: None
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
            in list (sorted):
              MyCallable (Function)
                detail: Some("function MyCallable() : Unit")
                additional_text_edits: None
              Bar (Module)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              Foo (Module)
                detail: None
                additional_text_edits: None
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
            not in list: 
              MyCallable2
            in list (sorted):
              MyCallable (Function)
                detail: Some("function MyCallable() : Unit")
                additional_text_edits: None
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
            in list (sorted):
              Fake (Function)
                detail: Some("operation Fake() : Unit")
                additional_text_edits: None
              Library (Module)
                detail: None
                additional_text_edits: None
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
            not in list: 
              Fake
            in list (sorted):
              Library (Module)
                detail: None
                additional_text_edits: None
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
            not in list: 
              Fake
            in list (sorted):
              Library (Module)
                detail: None
                additional_text_edits: None
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
            not in list: 
              Fake
            in list (sorted):
              Library (Module)
                detail: None
                additional_text_edits: None
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
            not in list: 
              Fake
            in list (sorted):
              Library (Module)
                detail: None
                additional_text_edits: None
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
            not in list: 
              Main
              FakeWithParam
            in list (sorted):
              Int (Interface)
                detail: None
                additional_text_edits: None
              Qubit (Interface)
                detail: None
                additional_text_edits: None
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits:
                  [2:12-2:12] "import FakeStdLib.Udt;\n            "
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
            not in list: 
              Main
              FakeWithParam
            in list (sorted):
              Int (Interface)
                detail: None
                additional_text_edits: None
              Qubit (Interface)
                detail: None
                additional_text_edits: None
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits:
                  [2:12-2:12] "import FakeStdLib.Udt;\n            "
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
            not in list: 
              Qubit
              Int
              Main
              FakeWithParam
            in list (sorted):
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits: None
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
            not in list: 
              Qubit
              FakeWithParam
            in list (sorted):
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits: None
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
            not in list: 
              Qubit
              Int
              Main
              FakeWithParam
            in list (sorted):
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits:
                  [2:12-2:12] "import FakeStdLib.Udt;\n            "
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
            not in list: 
              Qubit
              Int
              Main
              FakeWithParam
            in list (sorted):
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits: None
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
            not in list: 
              Qubit
            in list (sorted):
              FakeWithParam (Function)
                detail: Some("operation FakeWithParam(x : Int) : Unit")
                additional_text_edits: None
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits: None
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
            not in list: 
              Udt
              Qubit
              Int
              FakeWithParam
            in list (sorted):
              Main (Function)
                detail: Some("function Main() : Unit")
                additional_text_edits: None
              FakeStdLib (Module)
                detail: None
                additional_text_edits: None
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
            not in list: 
              Qubit
              Int
              Main
              FakeStdLib
            in list (sorted):
              FakeWithParam (Function)
                detail: Some("operation FakeWithParam(x : Int) : Unit")
                additional_text_edits: None
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits: None
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
            in list (sorted):
              Foo (Function)
                detail: Some("function Foo() : Unit")
                additional_text_edits: None
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
            in list (sorted):
              MainFunc (Function)
                detail: Some("function MainFunc() : Unit")
                additional_text_edits:
                  [0:17-0:17] "import MyDep.MainFunc;\n "
              OtherFunc (Function)
                detail: Some("function OtherFunc() : Unit")
                additional_text_edits:
                  [0:17-0:17] "import MyDep.Other.OtherFunc;\n "
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
            not in list: 
              Main
            in list (sorted):
              MyDep (Module)
                detail: None
                additional_text_edits: None
        "#]],
    );
}

#[test]
// #[ignore = "Main and Std show up as namespaces under MyDep, but they shouldn't"]
fn package_alias_members() {
    check_with_dependency(
        "namespace Test { function Foo() : Unit { MyDep.↘ } }",
        "MyDep",
        "namespace Main { export MainFunc; function MainFunc() : Unit {} }
        namespace Other { export OtherFunc; function OtherFunc() : Unit {} }
        namespace Other.Sub { export OtherFunc; function OtherFunc() : Unit {} }
        ",
        &["Main", "Other", "MainFunc", "Other.Sub", "Sub", "Std"],
        &expect![[r#"
            not in list: 
              Main
              Other.Sub
              Sub
              Std
            in list (sorted):
              MainFunc (Function)
                detail: Some("function MainFunc() : Unit")
                additional_text_edits: None
              Other (Module)
                detail: None
                additional_text_edits: None
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
            not in list: 
              Main
              Other
              MainFunc
              Other.Sub
            in list (sorted):
              OtherFunc (Function)
                detail: Some("function OtherFunc() : Unit")
                additional_text_edits: None
              Sub (Module)
                detail: None
                additional_text_edits: None
        "#]],
    );
}

#[test]
// #[ignore = "Main shows up as a namespace under MyDep, but it shouldn't"]
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
            not in list: 
              Main
              MainFunc
              Other.Sub
              Sub
            in list (sorted):
              Other (Module)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              CallableInFoo (Function)
                detail: Some("function CallableInFoo() : Unit")
                additional_text_edits: None
              Bar (Module)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              CallableInFoo (Function)
                detail: Some("function CallableInFoo() : Unit")
                additional_text_edits: None
              Bar (Module)
                detail: None
                additional_text_edits: None
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
            not in list: 
              CallableInFoo
              Bar
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
            in list (sorted):
              bar (Field)
                detail: Some("Int")
                additional_text_edits: None
        "#]],
    );
}

#[test]
fn input_type_missing() {
    check(
        "namespace Test { function Foo(x : FakeStdLib.↘ ) : Unit { body intrinsic; } }",
        &["Udt", "Library"],
        &expect![[r#"
            in list (sorted):
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits: None
              Library (Module)
                detail: None
                additional_text_edits: None
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
            not in list: 
              FakeStdLib
            in list (sorted):
              FakeWithParam (Function)
                detail: Some("operation FakeWithParam(x : Int) : Unit")
                additional_text_edits: None
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits: None
              Library (Module)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              bar (Field)
                detail: Some("Int")
                additional_text_edits: None
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
            not in list: 
              FakeStdLib
              FakeWithParam
            in list (sorted):
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits: None
              Library (Module)
                detail: None
                additional_text_edits: None
        "#]],
    );
}

#[test]
fn prefix_ops() {
    check(
        "namespace Test { function Main() : Unit { let x = ↘ ; } }",
        &["and", "or", "not", "Adjoint"],
        &expect![[r#"
            not in list: 
              and
              or
            in list (sorted):
              Adjoint (Keyword)
                detail: None
                additional_text_edits: None
              not (Keyword)
                detail: None
                additional_text_edits: None
        "#]],
    );
}

#[test]
fn binary_ops() {
    check(
        "namespace Test { function Main() : Unit { let x = 1 ↘ ; } }",
        &["and", "or", "not"],
        &expect![[r#"
            not in list: 
              not
            in list (sorted):
              and (Keyword)
                detail: None
                additional_text_edits: None
              or (Keyword)
                detail: None
                additional_text_edits: None
        "#]],
    );
}

#[test]
fn array_size() {
    check(
        "namespace Test { function Main() : Unit { let x = [0, ↘] ; } }",
        &["size"],
        &expect![[r#"
            in list (sorted):
              size (Keyword)
                detail: None
                additional_text_edits: None
        "#]],
    );
}

#[test]
fn path_segment_partial_ident_is_keyword() {
    check(
        "namespace Test { import FakeStdLib.struct↘ }",
        &["StructFn"],
        &expect![[r#"
            in list (sorted):
              StructFn (Interface)
                detail: Some("struct StructFn { inner : (Int -> Int) }")
                additional_text_edits: None
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
            not in list: 
              StructFn
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
            not in list: 
              StructFn
        "#]],
    );
}

#[test]
fn path_segment_before_glob() {
    check(
        "namespace Test { import FakeStdLib.↘* }",
        &["StructFn"],
        &expect![[r#"
            in list (sorted):
              StructFn (Interface)
                detail: Some("struct StructFn { inner : (Int -> Int) }")
                additional_text_edits: None
        "#]],
    );
}

#[test]
fn path_segment_before_glob_with_alias() {
    check(
        "namespace Test { import FakeStdLib.↘* as Alias }",
        &["StructFn"],
        &expect![[r#"
            in list (sorted):
              StructFn (Interface)
                detail: Some("struct StructFn { inner : (Int -> Int) }")
                additional_text_edits: None
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
            in list (sorted):
              bar (Field)
                detail: Some("Int")
                additional_text_edits: None
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
            in list (sorted):
              x (Field)
                detail: Some("Int")
                additional_text_edits: None
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
            in list (sorted):
              field (Field)
                detail: Some("Int")
                additional_text_edits: None
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
            not in list: 
              fieldBar
            in list (sorted):
              fieldFoo (Field)
                detail: Some("Int")
                additional_text_edits: None
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
            not in list: 
              fieldBar
            in list (sorted):
              fieldFoo (Field)
                detail: Some("Int")
                additional_text_edits: None
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
            not in list: 
              bar
            in list (sorted):
              var (Variable)
                detail: Some("var : Int")
                additional_text_edits: None
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
            not in list: 
              Fake
            in list (sorted):
              bar (Field)
                detail: Some("Int")
                additional_text_edits: None
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
            in list (sorted):
              'T (TypeParameter)
                detail: None
                additional_text_edits: None
              FakeStdLib (Module)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              'T (TypeParameter)
                detail: None
                additional_text_edits: None
              FakeStdLib (Module)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              Udt (Interface)
                detail: Some("struct Udt { x : Int, y : Int }")
                additional_text_edits: None
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
            in list (sorted):
              'T (TypeParameter)
                detail: None
                additional_text_edits: None
              FakeStdLib (Module)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              'T (TypeParameter)
                detail: None
                additional_text_edits: None
              FakeStdLib (Module)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              'T (TypeParameter)
                detail: None
                additional_text_edits: None
              FakeStdLib (Module)
                detail: None
                additional_text_edits: None
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

#[ignore = "https://github.com/microsoft/qsharp/issues/1955"]
// `Qux` and `Baz` should appear *without* an auto-import edit since they're already in scope.
#[test]
fn reexport_item_from_dependency() {
    check_with_dependency(
        r"
        namespace Test {
            open MyDep;
            operation Foo() : Unit {
                ↘
            }
        }
        ",
        "MyDep",
        "
        namespace Bar {
            operation Baz() : Unit {}
            export Baz;
        }
        namespace Main {
            operation Qux() : Unit {}
            export Qux, Bar.Baz;
        }
        ",
        &["Qux", "Baz", "Bar"],
        &expect![[r#"
            not in list: 
              Bar
            in list (sorted):
              Baz (Function)
                detail: Some("operation Baz() : Unit")
                additional_text_edits: None
              Qux (Function)
                detail: Some("operation Qux() : Unit")
                additional_text_edits: None
        "#]],
    );
}

#[test]
#[ignore = "`BazAlias` should show up in list without text edits since it's in scope"]
fn reexport_item_with_alias_from_dependency() {
    check_with_dependency(
        r"
        namespace Test {
            open MyDep;
            operation Foo() : Unit {
                ↘
            }
        }
        ",
        "MyDep",
        "
        namespace Bar {
            operation Baz() : Unit {}
            export Baz;
        }
        namespace Main {
            export Bar.Baz as BazAlias;
        }
        ",
        &["BazAlias"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "BazAlias",
                        kind: Function,
                        sort_text: Some(
                            "0400BazAlias",
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
#[ignore = "expect `Bar` and `Qux` but not `Foo`, I think"]
fn reexport_namespace_from_dependency_qualified() {
    check_with_dependency(
        r"
        namespace Test {
            open MyDep.Baz.↘
        }",
        "MyDep",
        "namespace Foo.Bar {
         }
         namespace Baz {
            operation Qux() : Unit {}
            export Qux, Foo.Bar;
         }",
        &["Qux", "Bar", "Foo"],
        &expect![[r#"
            [
                None,
                Some(
                    CompletionItem {
                        label: "Bar",
                        kind: Module,
                        sort_text: Some(
                            "0100Bar",
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

#[ignore = "https://github.com/microsoft/qsharp/issues/1955"]
// `Baz` should be in the list
#[test]
fn reexport_item_from_dependency_qualified() {
    check_with_dependency(
        r"
            namespace Test {
                operation Test() : Unit {
                    MyDep.↘
                }
            }",
        "MyDep",
        "namespace Foo {
                operation Baz() : Unit {}
                export Baz;
             }
             namespace Main {
                operation Qux() : Unit {}
                export Qux, Foo.Baz;
             }",
        &["Qux", "Baz"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Qux",
                        kind: Function,
                        sort_text: Some(
                            "0100Qux",
                        ),
                        detail: Some(
                            "operation Qux() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
                // THERE SHOULD BE `Baz` HERE
            ]
        "#]],
    );
}
