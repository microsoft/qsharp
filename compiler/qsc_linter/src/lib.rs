// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This crate contains the linter for the Q# language.
//!
//! It includes lints for the following stages of the compilation process:
//!  - AST
//!  - HIR
//!
//! The entry points to the linter are the run_*_lints functions, which take
//! a [`qsc_*::*::Package`] as input and outputs a Vec<[`Lint`]>.
//!
//! # Examples
//!
//! ```
//! use linter::run_ast_lints;;
//! use qsc_ast::ast::Package;
//!
//! let package: Package = todo!();
//! let lints: Vec<Lint> = run_ast_lints(&package);
//! ```

#![deny(missing_docs)]
#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod linter;
mod lints;

pub use linter::{ast::run_ast_lints, hir::run_hir_lints, Lint, LintLevel};
