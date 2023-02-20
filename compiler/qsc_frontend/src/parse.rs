// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The qsc parser uses recursive descent and Pratt parsing (or “top-down operator-precedence parsing”) to handle
//! turning an input string into a Q# abstract syntax tree. The parser is stateless, so produces a tree with
//! placeholder node identifiers that are expected to be replaced with unique identifiers by a later stage.

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

pub type Result<T> = result::Result<T, Error>;

trait Parser<T>: FnMut(&mut Scanner) -> Result<T> {}

impl<T, F: FnMut(&mut Scanner) -> Result<T>> Parser<T> for F {}

pub fn package(input: &str) -> (Result<Package>, Vec<Error>) {
    let mut scanner = Scanner::new(input);
    let p = top::package(&mut scanner);
    (p, scanner.errors())
}
