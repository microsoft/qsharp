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
//! The entry points to the linter is the `run_lints` function, which takes
//! a [`qsc_frontend::compile::CompileUnit`] as input and outputs a [`Vec<Lint>`](Lint).
//!
//! ## Example
//!
//! ```
//! use linter::run_lints;;
//! use qsc::compile::compile;
//!
//! let unit: CompileUnit = compile(...);
//!
//! // The second argument is an optional user configuration.
//! let lints: Vec<Lint> = run_ast_lints(&package, None);
//! ```
//!
//! # How to add a new Lint
//!
//! We can add a new lint in two steps:
//!  1. Declaring the lint: here you set the lint name, the default [`LintLevel`], and the message the user will see.
//!  2. Implementing the lint: here you write the pattern matching logic of the new lint.
//!
//! Below is a full example of how to a new AST lint.
//!
//! ## Example
//!
//! First, we add our lint to `src/lints/ast.rs`.
//! ```
//! declare_ast_lints!{
//!   ...
//!   (DoubleParens, LintLevel::Warn, "unnecesary double parentheses"),
//! }
//! ```
//!
//! Then we implement the right `LintPass` for our new lint, in this case `linter::ast::AstLintPass`
//! ```
//! impl linter::ast::AstLintPass for DoubleParens {
//!     // we only need to impl the relevant check_* method, all the other ones
//!     // will default to an empty method that will get optmized by rust
//!     fn check_expr(expr: &qsc_ast::ast::Expr, buffer: &mut Vec<Lint>) {
//!         // we match the relevant pattern
//!         if let ExprKind::Paren(ref inner_expr) = *expr.kind {
//!             if matches!(*inner_expr.kind, ExprKind::Paren(_)) {
//!                 // we push the lint to the buffer
//!                 push_lint!(Self, expr.span, buffer);
//!             }
//!         }
//!     }
//! }
//! ```

#![deny(missing_docs)]

mod linter;
mod lints;
#[cfg(test)]
mod tests;

pub use linter::{run_lints, Lint, LintConfig, LintKind, LintLevel};
