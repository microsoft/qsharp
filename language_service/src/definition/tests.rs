// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use expect_test::{expect, Expect};
use qsc::location::Location;

use super::get_definition;
use crate::{
    test_utils::{
        compile_notebook_with_fake_stdlib_and_markers, compile_with_fake_stdlib_and_markers,
    },
    Encoding,
};

/// Asserts that the definition given at the cursor position matches the expected range.
/// The cursor position is indicated by a `↘` marker in the source text.
/// The expected definition range is indicated by `◉` markers in the source text.
fn assert_definition(source_with_markers: &str) {
    let (compilation, cursor_position, target_spans) =
        compile_with_fake_stdlib_and_markers(source_with_markers);
    let actual_definition =
        get_definition(&compilation, "<source>", cursor_position, Encoding::Utf8);
    let expected_definition = if target_spans.is_empty() {
        None
    } else {
        Some(Location {
            source: "<source>".into(),
            range: target_spans[0],
        })
    };
    assert_eq!(&expected_definition, &actual_definition);
}

fn assert_definition_notebook(cells_with_markers: &[(&str, &str)]) {
    let (compilation, cell_uri, position, target_spans) =
        compile_notebook_with_fake_stdlib_and_markers(cells_with_markers);
    let actual_definition = get_definition(&compilation, &cell_uri, position, Encoding::Utf8);
    let expected_definition = if target_spans.is_empty() {
        None
    } else {
        Some(target_spans[0].clone())
    };
    assert_eq!(&expected_definition, &actual_definition);
}

fn check(source_with_markers: &str, expect: &Expect) {
    let (compilation, cursor_position, _) =
        compile_with_fake_stdlib_and_markers(source_with_markers);
    let actual_definition =
        get_definition(&compilation, "<source>", cursor_position, Encoding::Utf8);
    expect.assert_debug_eq(&actual_definition);
}

#[test]
fn callable() {
    assert_definition(
        r#"
    namespace Test {
        operation ◉F↘oo◉() : Unit {
        }
    }
    "#,
    );
}

#[test]
fn callable_ref() {
    assert_definition(
        r#"
    namespace Test {
        operation ◉Callee◉() : Unit {
        }

        operation Caller() : Unit {
            C↘allee();
        }
    }
    "#,
    );
}

#[test]
fn variable() {
    assert_definition(
        r#"
    namespace Test {
        operation Foo() : Unit {
            let ◉↘x◉ = 3;
        }
    }
    "#,
    );
}

#[test]
fn variable_ref() {
    assert_definition(
        r#"
    namespace Test {
        operation Foo() : Unit {
            let ◉x◉ = 3;
            let y = ↘x;
        }
    }
    "#,
    );
}

#[test]
fn parameter() {
    assert_definition(
        r#"
    namespace Test {
        operation Foo(◉↘x◉: Int) : Unit {
        }
    }
    "#,
    );
}

#[test]
fn parameter_ref() {
    assert_definition(
        r#"
    namespace Test {
        operation Foo(◉x◉: Int) : Unit {
            let y = ↘x;
        }
    }
    "#,
    );
}

#[test]
fn udt() {
    assert_definition(
        r#"
    namespace Test {
        newtype ◉B↘ar◉ = (a: Int, b: Double);
    }
    "#,
    );
}

#[test]
fn udt_ref() {
    assert_definition(
        r#"
    namespace Test {
        newtype ◉Bar◉ = (a: Int, b: Double);

        operation Foo() : Unit {
            let x = B↘ar(1, 2.3);
        }
    }
    "#,
    );
}

#[test]
fn udt_ref_sig() {
    assert_definition(
        r#"
    namespace Test {
        newtype ◉Bar◉ = (a: Int, b: Double);

        operation Foo() : B↘ar {
            Bar(1, 2.3)
        }
    }
    "#,
    );
}

#[test]
fn udt_ref_param() {
    assert_definition(
        r#"
    namespace Test {
        newtype ◉Bar◉ = (a: Int, b: Double);

        operation Foo(x: B↘ar) : Unit {
        }
    }
    "#,
    );
}

#[test]
fn udt_ref_anno() {
    assert_definition(
        r#"
    namespace Test {
        newtype ◉Bar◉ = (a: Int, b: Double);

        operation Foo() : Unit {
            let x: B↘ar = Bar(1, 2.3);
        }
    }
    "#,
    );
}

#[test]
fn udt_ref_ty_def() {
    assert_definition(
        r#"
    namespace Test {
        newtype ◉Bar◉ = (a: Int, b: Double);
        newtype Foo = (a: B↘ar, b: Double);
    }
    "#,
    );
}

#[test]
fn udt_field() {
    assert_definition(
        r#"
    namespace Test {
        newtype Pair = (◉f↘st◉: Int, snd: Double);
    }
    "#,
    );
}

#[test]
fn udt_field_ref() {
    assert_definition(
        r#"
    namespace Test {
        newtype Pair = (fst: Int, ◉snd◉: Double);
        operation Foo() : Unit {
            let a = Pair(1, 2.3);
            let b = a::s↘nd;
        }
    }
    "#,
    );
}

#[test]
fn lambda_param() {
    assert_definition(
        r#"
    namespace Test {
        operation Foo() : Unit {
            let local = (◉↘x◉, y) => x;
            let z = local(1, 2.3);
        }
    }
    "#,
    );
}

#[test]
fn lambda_param_ref() {
    assert_definition(
        r#"
    namespace Test {
        operation Foo() : Unit {
            let local = (◉x◉, y) => ↘x;
            let z = local(1, 2.3);
        }
    }
    "#,
    );
}

#[test]
fn lambda_closure_ref() {
    assert_definition(
        r#"
    namespace Test {
        operation Foo() : Unit {
            let ◉a◉ = "Hello";
            let local = (x, y) => ↘a;
            let z = local(1, 2.3);
        }
    }
    "#,
    );
}

#[test]
fn std_call() {
    check(
        r#"
    namespace Test {
        open FakeStdLib;
        operation Foo() : Unit {
            F↘ake();
        }
    }
    "#,
        &expect![[r#"
            Some(
                Location {
                    source: "qsharp-library-source:<std>",
                    range: Range {
                        start: Position {
                            line: 1,
                            column: 26,
                        },
                        end: Position {
                            line: 1,
                            column: 30,
                        },
                    },
                },
            )
        "#]],
    );
}

#[test]
fn other_namespace_call_ref() {
    assert_definition(
        r#"
    namespace Test {
        open Other;
        operation Foo() : Unit {
            B↘ar();
        }
    }

    namespace Other {
        operation ◉Bar◉() : Unit {}
    }
    "#,
    );
}

#[test]
fn parameter_ref_with_body_specialization() {
    assert_definition(
        r#"
    namespace Test {
        operation Foo(◉x◉: Int) : Unit is Adj {
            body ... {
                let y = ↘x;
            }
        }
    }
    "#,
    );
}

#[test]
fn parameter_ref_with_adj_specialization() {
    assert_definition(
        r#"
    namespace Test {
        operation Foo(◉x◉: Int) : Unit is Adj {
            body ... {}
            adjoint ... {
                let y = ↘x;
            }
        }
    }
    "#,
    );
}

#[test]
fn ctl_specialization_parameter() {
    assert_definition(
        r#"
    namespace Test {
        operation Foo(x: Int) : Unit is Ctl {
            body ... {}
            controlled (◉c↘s◉, ...) {}
        }
    }
    "#,
    );
}

#[test]
fn ctl_specialization_parameter_ref() {
    assert_definition(
        r#"
    namespace Test {
        operation Foo(x: Int) : Unit is Ctl {
            body ... {}
            controlled (◉cs◉, ...) {
                let y = c↘s;
            }
        }
    }
    "#,
    );
}

#[test]
fn std_udt() {
    check(
        r#"
    namespace Test {
        operation Foo() : FakeStdLib.Ud↘t {
        }
    }
    "#,
        &expect![[r#"
            Some(
                Location {
                    source: "qsharp-library-source:<std>",
                    range: Range {
                        start: Position {
                            line: 4,
                            column: 24,
                        },
                        end: Position {
                            line: 4,
                            column: 27,
                        },
                    },
                },
            )
        "#]],
    );
}

#[test]
fn std_udt_udt_field() {
    check(
        r#"
    namespace Test {
        open FakeStdLib;
        operation Foo() : Udt {
            let f = UdtWrapper(TakesUdt);
            f::inner::x↘
        }
    }
    "#,
        &expect![[r#"
            Some(
                Location {
                    source: "qsharp-library-source:<std>",
                    range: Range {
                        start: Position {
                            line: 4,
                            column: 31,
                        },
                        end: Position {
                            line: 4,
                            column: 32,
                        },
                    },
                },
            )
        "#]],
    );
}

#[test]
fn ty_param_def() {
    assert_definition(
        r#"
        namespace Test {
            operation Foo<◉'↘T◉>(x : 'T) : 'T { x }
        }
    "#,
    );
}

#[test]
fn ty_param_ref() {
    assert_definition(
        r#"
        namespace Test {
            operation Foo<◉'T◉>(x : '↘T) : 'T { x }
        }
    "#,
    );
}

#[test]
fn notebook_callable_def_across_cells() {
    assert_definition_notebook(&[
        ("cell1", "operation ◉Callee◉() : Unit {}"),
        ("cell2", "C↘allee();"),
    ]);
}

#[test]
fn notebook_callable_defined_in_later_cell() {
    assert_definition_notebook(&[
        ("cell1", "C↘allee();"),
        ("cell2", "operation Callee() : Unit {}"),
    ]);
}
