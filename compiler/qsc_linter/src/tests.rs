// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    linter::{ast::run_ast_lints, hir::run_hir_lints},
    Lint, LintConfig, LintLevel,
};
use expect_test::{expect, Expect};
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_frontend::compile::{self, CompileUnit, PackageStore, RuntimeCapabilityFlags, SourceMap};
use qsc_passes::PackageType;

#[test]
fn multiple_lints() {
    check(
        "let x = ((1 + 2)) / 0;;;;",
        &expect![[r#"
            [
                SrcLint {
                    source: ";;;",
                    message: "redundant semicolons",
                    level: Warn,
                },
                SrcLint {
                    source: "((1 + 2)) / 0",
                    message: "attempt to divide by zero",
                    level: Allow,
                },
                SrcLint {
                    source: "((1 + 2))",
                    message: "unnecessary parentheses",
                    level: Warn,
                },
            ]
        "#]],
    );
}

#[test]
fn double_parens() {
    check(
        "let x = ((1 + 2));",
        &expect![[r#"
            [
                SrcLint {
                    source: "((1 + 2))",
                    message: "unnecessary parentheses",
                    level: Warn,
                },
            ]
        "#]],
    );
}

#[test]
fn division_by_zero() {
    check(
        "let x = 2 / 0;",
        &expect![[r#"
            [
                SrcLint {
                    source: "2 / 0",
                    message: "attempt to divide by zero",
                    level: Allow,
                },
            ]
        "#]],
    );
}

#[test]
fn needless_parens_in_assignment() {
    check(
        "let x = (42);",
        &expect![[r#"
            [
                SrcLint {
                    source: "(42)",
                    message: "unnecessary parentheses",
                    level: Warn,
                },
                SrcLint {
                    source: "42",
                    message: "remove this stump after addding the first HIR lint",
                    level: Warn,
                },
            ]
        "#]],
    );
}

#[test]
fn needless_parens() {
    check(
        "let x = (2) + (5 * 4 * (2 ^ 10));",
        &expect![[r#"
            [
                SrcLint {
                    source: "(2)",
                    message: "unnecessary parentheses",
                    level: Warn,
                },
                SrcLint {
                    source: "(5 * 4 * (2 ^ 10))",
                    message: "unnecessary parentheses",
                    level: Warn,
                },
                SrcLint {
                    source: "(2 ^ 10)",
                    message: "unnecessary parentheses",
                    level: Warn,
                },
            ]
        "#]],
    );
}

#[test]
fn redundant_semicolons() {
    check(
        "let x = 2;;;;;",
        &expect![[r#"
            [
                SrcLint {
                    source: ";;;;",
                    message: "redundant semicolons",
                    level: Warn,
                },
            ]
        "#]],
    );
}

#[test]
fn hir_stump() {
    check(
        "let stump = 42;",
        &expect![[r#"
            [
                SrcLint {
                    source: "42",
                    message: "remove this stump after addding the first HIR lint",
                    level: Warn,
                },
            ]
        "#]],
    );
}

fn check(source: &str, expected: &Expect) {
    let source = wrap_in_namespace(source);
    let mut store = PackageStore::new(compile::core());
    let std = store.insert(compile::std(&store, RuntimeCapabilityFlags::all()));
    let sources = SourceMap::new([("source.qs".into(), source.clone().into())], None);
    let (package, _) = qsc::compile::compile(
        &store,
        &[std],
        sources,
        PackageType::Exe,
        RuntimeCapabilityFlags::all(),
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
        "namespace foo {{
        operation RunProgram(vector : Double[]) : Unit {{
            {source}
        }}
    }}"
    )
}

/// A version of Lint that replaces the span by source code
/// to make unit tests easier to write and verify.
#[derive(Debug)]
struct SrcLint {
    source: String,
    message: &'static str,
    level: LintLevel,
}

impl SrcLint {
    fn from(lint: &Lint, source: &str) -> Self {
        Self {
            source: source[lint.span].into(),
            message: lint.message,
            level: lint.level,
        }
    }
}

impl std::fmt::Display for SrcLint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Lint {{
                source: {},
                message: {},
                level: {},
            }}",
            self.source, self.message, self.level
        )
    }
}

fn run_lints(compile_unit: &CompileUnit, config: Option<&[LintConfig]>) -> Vec<Lint> {
    let mut ast_lints = run_ast_lints(&compile_unit.ast.package, config);
    let mut hir_lints = run_hir_lints(&compile_unit.package, config);

    println!("{:#?}", &compile_unit.package);

    let mut lints = Vec::new();
    lints.append(&mut ast_lints);
    lints.append(&mut hir_lints);
    lints
}
