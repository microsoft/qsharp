// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This crate contains the linter for the Q# language.
//!
//! It includes lints for the following stages of the compilation process:
//!  - AST
//!  - HIR
//!
//! # Usage
//!
//! The entry points to the linter are the `run_*_lints` functions, which take
//! a `qsc_*::*::Package` as input and output a [`Vec<Lint>`](Lint).
//!
//! ## Example
//!
//! ```
//! use linter::run_ast_lints;;
//! use qsc_ast::ast::Package;
//!
//! let package: Package = ...;
//! let lints: Vec<Lint> = run_ast_lints(&package);
//! ```
//!
//! # How to add a new Lint
//!
//! Adding a new lint has three steps:
//!  1. Declaring the lint: here you set the lint name, the default [`LintLevel`], and the message the user will see.
//!  2. Implementing the lint: here you write the pattern matching logic of the new lint.
//!  3. Adding the new lint to the right linter.
//!
//! Below is a full example of how to a new AST lint.
//!
//! ## Example
//!
//! First, we declare and implement our new lint in `src/lints/ast.rs`.
//! ```
//! declare_lint!(DoubleParens, LintLevel::Warn, "unnecesary double parentheses")
//!
//! // implement the right LintPass for our new lint,
//! // in this case [`linter::ast::AstLintPass`]
//! impl linter::ast::AstLintPass for DoubleParens {
//!     // we only need to impl the relevant check_* method, all the other ones
//!     // will default to an empty method that will get optmized by rust
//!     fn check_expr(expr: &qsc_ast::ast::Expr, buffer: &mut Vec<Lint>) {
//!         // we match the relevant pattern
//!         if let ExprKind::Paren(ref inner_expr) = *expr.kind {
//!             if matches!(*inner_expr.kind, ExprKind::Paren(_)) {
//!                 // we push the lint to the buffer
//!                 push_lint!(Self, expr, buffer);
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! Finally we add our new lint to `impl CombinedAstLints { ... }` in `src/linter/ast.rs`.
//!
//! ```
//! impl AstLintPass for CombinedAstLints {
//!     fn check_expr(expr: &Expr) {
//!         // ... some other lints
//!
//!         DoubleParens::check_expr(expr); // add your new lint here
//!     }
//! }
//! ```

#![deny(missing_docs)]
#![warn(clippy::pedantic, clippy::unwrap_used)]

mod linter;
mod lints;
#[cfg(test)]
mod tests;

pub use linter::{ast::run_ast_lints, hir::run_hir_lints, Lint, LintLevel};
use miette::{Diagnostic, LabeledSpan};

/// Wrapper around a Lint to provide an error representation
#[derive(Debug, Clone, thiserror::Error)]
pub struct Error(pub Lint);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.message)
    }
}

impl Diagnostic for Error {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn severity(&self) -> Option<miette::Severity> {
        match self.0.level {
            LintLevel::Allow => None,
            LintLevel::Warning | LintLevel::ForceWarning => Some(miette::Severity::Warning),
            LintLevel::Error | LintLevel::ForceError => Some(miette::Severity::Error),
        }
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        None
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        let source_span = miette::SourceSpan::from(self.0.span);
        let labeled_span = LabeledSpan::new_with_span(Some(self.to_string()), source_span);
        Some(Box::new(vec![labeled_span].into_iter()))
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        None
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        None
    }
}
