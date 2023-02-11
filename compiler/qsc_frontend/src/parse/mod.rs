// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod kw;
mod scan;
mod top;

use self::scan::Scanner;
use qsc_ast::ast::{Package, Span};
use std::result;

#[derive(Debug)]
pub struct Error {
    pub message: &'static str,
    pub span: Span,
}

pub type Result<T> = result::Result<T, Error>;

pub fn package(input: &str) -> (Result<Package>, Vec<Error>) {
    let mut scanner = Scanner::new(input);
    let p = top::package(&mut scanner);
    (p, scanner.errors())
}
