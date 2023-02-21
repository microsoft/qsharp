// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod expr;
mod keyword;
mod prim;
mod scan;
mod stmt;
#[cfg(test)]
mod tests;
mod top;
mod ty;

use self::{keyword::Keyword, scan::Scanner};
use crate::lex::TokenKind;
use qsc_ast::ast::{Package, Span};
use std::result;

// TODO: Format errors so they can be displayed to the user.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    span: Span,
}

#[derive(Debug)]
enum ErrorKind {
    Keyword(Keyword),
    Lexical(&'static str),
    Rule(&'static str),
    Token(TokenKind),
}

pub(super) type Result<T> = result::Result<T, Error>;

trait Parser<T>: FnMut(&mut Scanner) -> Result<T> {}

impl<T, F: FnMut(&mut Scanner) -> Result<T>> Parser<T> for F {}

pub(super) fn package(input: &str) -> (Result<Package>, Vec<Error>) {
    let mut scanner = Scanner::new(input);
    let p = top::package(&mut scanner);
    (p, scanner.errors())
}
