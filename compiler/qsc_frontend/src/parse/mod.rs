// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod core;
mod kw;
mod scan;

use self::scan::Scanner;
use qsc_ast::ast::{Package, Span};
use std::result;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub span: Span,
}

pub type Result<T> = result::Result<T, Error>;

trait Parser<T>: FnMut(&mut Scanner) -> Result<T> {}

impl<T, F: FnMut(&mut Scanner) -> Result<T>> Parser<T> for F {}

pub fn package(input: &str) -> (Result<Package>, Vec<Error>) {
    let mut scanner = Scanner::new(input);
    let p = core::package(&mut scanner);
    (p, scanner.errors())
}
