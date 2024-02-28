use crate::{linter::ast::run_ast_lints, Lint, LintLevel};
use expect_test::{expect, Expect};
use qsc_ast::{
    assigner::Assigner,
    ast::{self, NodeId},
    mut_visit::MutVisitor,
};
use qsc_data_structures::language_features::LanguageFeatures;

#[test]
fn double_parens() {
    check_ast(
        "let x = ((1 + 2));",
        &expect![[r#"
            [
                SrcLint {
                    source: "((1 + 2))",
                    message: "unnecesary double parentheses",
                    level: Warning,
                },
            ]
        "#]],
    );
}

#[test]
fn multiple_lints() {
    check_ast(
        "let x = ((1 + 2)) / 0;",
        &expect![[r#"
        [
            SrcLint {
                source: "((1 + 2)) / 0",
                message: "attempt to divide by zero",
                level: Allow,
            },
            SrcLint {
                source: "((1 + 2))",
                message: "unnecesary double parentheses",
                level: Warning,
            },
        ]
    "#]],
    );
}

/// Checks that the AST of the given source code generates the right Lints.
fn check_ast(source: &str, expected: &Expect) {
    let source = wrap_in_namespace(source);
    let package = parse(&source);
    let actual: Vec<SrcLint> = run_ast_lints(&package)
        .into_iter()
        .map(|lint| SrcLint::from(lint, &source))
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

fn parse(source: &str) -> ast::Package {
    let mut package = ast::Package {
        id: NodeId::FIRST,
        nodes: qsc_parse::top_level_nodes(source, LanguageFeatures::default())
            .0
            .into(),
        entry: None,
    };

    let mut assigner = Assigner::new();
    assigner.visit_package(&mut package);

    package
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
    fn from(lint: Lint, source: &str) -> Self {
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
