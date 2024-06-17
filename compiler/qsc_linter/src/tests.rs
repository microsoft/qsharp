// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    linter::{ast::run_ast_lints, hir::run_hir_lints},
    Lint, LintConfig, LintLevel,
};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use qsc_frontend::compile::{self, CompileUnit, PackageStore, SourceMap};
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

fn check(source: &str, expected: &Expect) {
    let source = wrap_in_namespace(source);
    let mut store = PackageStore::new(compile::core());
    let std = store.insert(compile::std(&store, TargetCapabilityFlags::all()));
    let sources = SourceMap::new([("source.qs".into(), source.clone().into())], None);
    let (package, _) = qsc::compile::compile(
        &store,
        &[std],
        sources,
        PackageType::Exe,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );

    let actual: Vec<SrcLint> = run_lints(&package, None)
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

fn run_lints(compile_unit: &CompileUnit, config: Option<&[LintConfig]>) -> Vec<Lint> {
    let mut ast_lints = run_ast_lints(&compile_unit.ast.package, config);
    let mut hir_lints = run_hir_lints(&compile_unit.package, config);
    let mut lints = Vec::new();
    lints.append(&mut ast_lints);
    lints.append(&mut hir_lints);
    lints
}
