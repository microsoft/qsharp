// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

use qsc_ast::{ast::Package, mut_visit::MutVisitor};

mod id;
mod lex;
pub mod parse;
mod symbol;

pub fn compile(input: &str) -> (Result<Package, parse::Error>, Vec<parse::Error>) {
    let (mut package, errors) = parse::package(input);
    let mut assigner = id::Assigner::new();
    package.iter_mut().for_each(|p| assigner.visit_package(p));
    (package, errors)
}
