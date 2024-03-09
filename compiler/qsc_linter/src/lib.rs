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
#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod linter;
mod lints;
#[cfg(test)]
mod tests;

pub use linter::{ast::run_ast_lints, hir::run_hir_lints, Lint, LintLevel};
