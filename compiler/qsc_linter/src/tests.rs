// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{run_lints, Lint, LintLevel};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use qsc_frontend::compile::{self, PackageStore, SourceMap};
use qsc_hir::hir::CallableKind;
use qsc_passes::PackageType;

#[test]
fn multiple_lints() {
    check(
        &wrap_in_callable("let x = ((1 + 2)) / 0;;;;", CallableKind::Operation),
        &expect![[r#"
            [
                SrcLint {
                    source: ";;;",
                    level: Warn,
                    message: "redundant semicolons",
                    help: "remove the redundant semicolons",
                },
                SrcLint {
                    source: "((1 + 2)) / 0",
                    level: Error,
                    message: "attempt to divide by zero",
                    help: "division by zero will fail at runtime",
                },
                SrcLint {
                    source: "((1 + 2))",
                    level: Allow,
                    message: "unnecessary parentheses",
                    help: "remove the extra parentheses for clarity",
                },
                SrcLint {
                    source: "RunProgram",
                    level: Allow,
                    message: "operation does not contain any quantum operations",
                    help: "this callable can be declared as a function instead",
                },
            ]
        "#]],
    );
}

#[test]
fn double_parens() {
    check(
        &wrap_in_callable("let x = ((1 + 2));", CallableKind::Function),
        &expect![[r#"
            [
                SrcLint {
                    source: "((1 + 2))",
                    level: Allow,
                    message: "unnecessary parentheses",
                    help: "remove the extra parentheses for clarity",
                },
            ]
        "#]],
    );
}

#[test]
fn division_by_zero() {
    check(
        &wrap_in_callable("let x = 2 / 0;", CallableKind::Function),
        &expect![[r#"
            [
                SrcLint {
                    source: "2 / 0",
                    level: Error,
                    message: "attempt to divide by zero",
                    help: "division by zero will fail at runtime",
                },
            ]
        "#]],
    );
}

#[test]
fn needless_parens_in_assignment() {
    check(
        &wrap_in_callable("let x = (42);", CallableKind::Function),
        &expect![[r#"
            [
                SrcLint {
                    source: "(42)",
                    level: Allow,
                    message: "unnecessary parentheses",
                    help: "remove the extra parentheses for clarity",
                },
            ]
        "#]],
    );
}

#[test]
fn needless_parens() {
    check(
        &wrap_in_callable("let x = (2) + (5 * 4 * (2 ^ 10));", CallableKind::Function),
        &expect![[r#"
            [
                SrcLint {
                    source: "(2)",
                    level: Allow,
                    message: "unnecessary parentheses",
                    help: "remove the extra parentheses for clarity",
                },
                SrcLint {
                    source: "(5 * 4 * (2 ^ 10))",
                    level: Allow,
                    message: "unnecessary parentheses",
                    help: "remove the extra parentheses for clarity",
                },
                SrcLint {
                    source: "(2 ^ 10)",
                    level: Allow,
                    message: "unnecessary parentheses",
                    help: "remove the extra parentheses for clarity",
                },
            ]
        "#]],
    );
}

#[test]
fn redundant_semicolons() {
    check(
        &wrap_in_callable("let x = 2;;;;;", CallableKind::Function),
        &expect![[r#"
            [
                SrcLint {
                    source: ";;;;",
                    level: Warn,
                    message: "redundant semicolons",
                    help: "remove the redundant semicolons",
                },
            ]
        "#]],
    );
}

#[test]
fn needless_operation_lambda_operations() {
    check(
        &wrap_in_callable("let a = (a) => a + 1;", CallableKind::Function),
        &expect![[r#"
            [
                SrcLint {
                    source: "",
                    level: Allow,
                    message: "operation does not contain any quantum operations",
                    help: "this callable can be declared as a function instead",
                },
            ]
        "#]],
    );
}

#[test]
fn needless_operation_no_lint_for_valid_lambda_operations() {
    check(
        &wrap_in_callable("let op = (q) => H(q);", CallableKind::Function),
        &expect![[r"
            []
        "]],
    );
}

#[test]
fn needless_operation_non_empty_op_and_no_specialization() {
    check(
        &wrap_in_callable("let x = 2;", CallableKind::Operation),
        &expect![[r#"
            [
                SrcLint {
                    source: "RunProgram",
                    level: Allow,
                    message: "operation does not contain any quantum operations",
                    help: "this callable can be declared as a function instead",
                },
            ]
        "#]],
    );
}

#[test]
fn needless_operation_non_empty_op_and_specialization() {
    check(
        indoc! {"
        operation Run(target : Qubit) : Unit is Adj {
            body ... {
                Message(\"hi\");
            }
            adjoint self;
        }
    "},
        &expect![[r#"
            [
                SrcLint {
                    source: "Run",
                    level: Allow,
                    message: "operation does not contain any quantum operations",
                    help: "this callable can be declared as a function instead",
                },
            ]
        "#]],
    );
}

#[test]
fn needless_operation_no_lint_for_empty_op_explicit_specialization() {
    check(
        indoc! {"
        operation I(target : Qubit) : Unit {
            body ... {}
            adjoint self;
        }

    "},
        &expect![[r"
            []
        "]],
    );
}

#[test]
fn needless_operation_no_lint_for_empty_op_implicit_specialization() {
    check(
        indoc! {"
        operation DoNothing() : Unit is Adj + Ctl {}
    "},
        &expect![[r"
            []
        "]],
    );
}

#[test]
fn needless_operation_partial_application() {
    check(
        indoc! {"
        operation PrepareBellState(q1 : Qubit, q2 : Qubit) : Unit {
            H(q1);
            CNOT(q1, q2);
        }

        operation PartialApplication(q1 : Qubit) : Qubit => Unit {
            return PrepareBellState(q1, _);
        }
    "},
        &expect![[r#"
            [
                SrcLint {
                    source: "PartialApplication",
                    level: Allow,
                    message: "operation does not contain any quantum operations",
                    help: "this callable can be declared as a function instead",
                },
            ]
        "#]],
    );
}

#[test]
fn deprecated_newtype_usage() {
    check(
        indoc! {"
        newtype Foo = ();
    "},
        &expect![[r#"
            [
                SrcLint {
                    source: "newtype Foo = ();",
                    level: Warn,
                    message: "deprecated `newtype` declarations",
                    help: "`newtype` declarations are deprecated, use `struct` instead",
                },
            ]
        "#]],
    );
}

#[test]
fn deprecated_function_cons() {
    check(
        indoc! {"
        struct Foo {}
        function Bar() : Foo { Foo() }
    "},
        &expect![[r#"
            [
                SrcLint {
                    source: "Foo",
                    level: Warn,
                    message: "deprecated function constructors",
                    help: "function constructors for struct types are deprecated, use `new` instead",
                },
            ]
        "#]],
    );
}

#[test]
fn deprecated_with_op_for_structs() {
    check(
        indoc! {"
        struct Foo { x : Int}
        function Bar() : Foo {
            let foo = new Foo { x = 2 };
            foo w/ x <- 3
        }
    "},
        &expect![[r#"
            [
                SrcLint {
                    source: "foo w/ x <- 3",
                    level: Warn,
                    message: "deprecated `w/` and `w/=` operators for structs",
                    help: "`w/` and `w/=` operators for structs are deprecated, use `new` instead",
                },
            ]
        "#]],
    );
}

#[test]
fn deprecated_with_eq_op_for_structs() {
    check(
        indoc! {"
        struct Foo { x : Int}
        function Bar() : Foo {
            mutable foo = new Foo { x = 2 };
            set foo w/= x <- 3;
            foo
        }
    "},
        &expect![[r#"
            [
                SrcLint {
                    source: "set foo w/= x <- 3",
                    level: Warn,
                    message: "deprecated `w/` and `w/=` operators for structs",
                    help: "`w/` and `w/=` operators for structs are deprecated, use `new` instead",
                },
            ]
        "#]],
    );
}

fn check(source: &str, expected: &Expect) {
    let source = wrap_in_namespace(source);
    let mut store = PackageStore::new(compile::core());
    let std = store.insert(compile::std(&store, TargetCapabilityFlags::all()));
    let sources = SourceMap::new([("source.qs".into(), source.clone().into())], None);
    let (unit, _) = qsc::compile::compile(
        &store,
        &[std],
        sources,
        PackageType::Exe,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );

    let package_id = store.insert(unit);
    let compile_unit = store
        .get(package_id)
        .expect("expected to find user package");

    let actual: Vec<SrcLint> = run_lints(&store, package_id, compile_unit, None)
        .into_iter()
        .map(|lint| SrcLint::from(&lint, &source))
        .collect();

    expected.assert_debug_eq(&actual);
}

/// Wraps some source code into a namespace, to make testing easier.
fn wrap_in_namespace(source: &str) -> String {
    format!(
        "namespace Foo {{
            {source}
        }}"
    )
}

fn wrap_in_callable(source: &str, callable_type: CallableKind) -> String {
    format!(
        "{callable_type} RunProgram() : Unit {{
            {source}
        }}"
    )
}

/// A version of Lint that replaces the span by source code
/// to make unit tests easier to write and verify.
#[derive(Debug)]
struct SrcLint {
    source: String,
    level: LintLevel,
    message: &'static str,
    help: &'static str,
}

impl SrcLint {
    fn from(lint: &Lint, source: &str) -> Self {
        Self {
            source: source[lint.span].into(),
            level: lint.level,
            message: lint.message,
            help: lint.help,
        }
    }
}

impl std::fmt::Display for SrcLint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Lint {{
                source: {},
                level: {},
                message: {},
                help: {},
            }}",
            self.source, self.level, self.message, self.help
        )
    }
}
