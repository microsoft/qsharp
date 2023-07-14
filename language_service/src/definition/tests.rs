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
fn definition_callable() {
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
fn definition_callable_ref() {
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
fn definition_variable() {
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
fn definition_variable_ref() {
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
fn definition_parameter() {
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
fn definition_parameter_ref() {
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
fn definition_udt() {
    assert_definition(
        r#"
    namespace Test {
        newtype ◉B↘ar = (a: Int, b: Double);
    }
    "#,
    );
}

#[test]
fn definition_udt_ref() {
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
fn definition_udt_ref_sig() {
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
fn definition_udt_ref_param() {
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
fn definition_udt_ref_anno() {
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
fn definition_udt_ref_ty_def() {
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
fn definition_lambda_param() {
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
fn definition_lambda_param_ref() {
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
fn definition_lambda_closure_ref() {
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
