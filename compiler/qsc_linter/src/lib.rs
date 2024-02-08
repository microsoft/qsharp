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
//!     fn check_expr(expr: &qsc_ast::ast::Expr) {
//!         // we match the relevant pattern
//!         if let ExprKind::Paren(ref inner_expr) = *expr.kind {
//!             if matches!(*inner_expr.kind, ExprKind::Paren(_)) {
//!                 // we push the lint to an internal stack
//!                 push_lint!(Self, expr);
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! Finally we add our new lint to `impl AstLintPass for CombinedAstLints { ... }`
//! in `src/linter/ast.rs`.
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
#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod linter;
mod lints;

pub use linter::{ast::run_ast_lints, hir::run_hir_lints, Lint, LintLevel};
