// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{get_definition, Definition};
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};

/// Asserts that the definition found at the given cursor position matches the expected position.
/// The cursor position is indicated by a `↘` marker in the source text.
/// The expected definition position is indicated by a `◉` marker in the source text.
fn assert_definition(source_with_markers: &str) {
    let (source, cursor_offsets, target_offsets) =
        get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual_definition = get_definition(&compilation, "<source>", cursor_offsets[0]);
    let expected_definition = if target_offsets.is_empty() {
        None
    } else {
        Some(Definition {
            source: "<source>".to_string(),
            offset: target_offsets[0],
        })
    };
    assert_eq!(&expected_definition, &actual_definition);
}

#[test]
fn callable() {
    assert_definition(
        r#"
    namespace Test {
        operation ◉F↘oo() : Unit {
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
        operation ◉Callee() : Unit {
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
            let ◉↘x = 3;
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
            let ◉x = 3;
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
        operation Foo(◉↘x: Int) : Unit {
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
        operation Foo(◉x: Int) : Unit {
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
        newtype ◉B↘ar = (a: Int, b: Double);
    }
    "#,
    );
}

#[test]
fn udt_ref() {
    assert_definition(
        r#"
    namespace Test {
        newtype ◉Bar = (a: Int, b: Double);

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
        newtype ◉Bar = (a: Int, b: Double);

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
        newtype ◉Bar = (a: Int, b: Double);

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
        newtype ◉Bar = (a: Int, b: Double);

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
        newtype ◉Bar = (a: Int, b: Double);
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
        newtype Pair = (◉f↘st: Int, snd: Double);
    }
    "#,
    );
}

#[test]
fn udt_field_ref() {
    assert_definition(
        r#"
    namespace Test {
        newtype Pair = (fst: Int, ◉snd: Double);
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
            let local = (◉↘x, y) => x;
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
            let local = (◉x, y) => ↘x;
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
            let ◉a = "Hello";
            let local = (x, y) => ↘a;
            let z = local(1, 2.3);
        }
    }
    "#,
    );
}

#[test]
fn std_call_no_goto() {
    assert_definition(
        r#"
    namespace Test {
        open FakeStdLib;
        operation Foo() : Unit {
            F↘ake();
        }
    }
    "#,
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
        operation ◉Bar() : Unit {}
    }
    "#,
    );
}

#[test]
fn parameter_ref_with_body_specialization() {
    assert_definition(
        r#"
    namespace Test {
        operation Foo(◉x: Int) : Unit is Adj {
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
        operation Foo(◉x: Int) : Unit is Adj {
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
            controlled (◉c↘s, ...) {}
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
            controlled (◉cs, ...) {
                let y = c↘s;
            }
        }
    }
    "#,
    );
}
