// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The qsc parser uses recursive descent to handle turning an input string into a Q# abstract syntax tree.
//! The parser produces a tree with placeholder node identifiers that are expected to be replaced with
//! unique identifiers by a later stage.

mod expr;
mod keyword;
mod prim;
mod scan;
mod stmt;
#[cfg(test)]
mod tests;
mod top;
mod ty;

use self::scan::Scanner;
use qsc_ast::ast::{Package, Span};
use std::result;

pub(super) use self::keyword::Keyword;
pub(super) use crate::lex::TokenKind;

#[derive(Debug)]
pub(super) struct Error {
    pub(super) kind: ErrorKind,
    pub(super) span: Span,
}

#[derive(Debug)]
pub(super) enum ErrorKind {
    Keyword(Keyword),
    Lexical(&'static str),
    Rule(&'static str),
    Token(TokenKind),
}

pub(super) type Result<T> = result::Result<T, Error>;

trait Parser<T>: FnMut(&mut Scanner) -> Result<T> {}

impl<T, F: FnMut(&mut Scanner) -> Result<T>> Parser<T> for F {}

pub(super) fn package(input: &str) -> (Package, Vec<Error>) {
    let mut scanner = Scanner::new(input);
    match top::package(&mut scanner) {
        Ok(pack) => (pack, scanner.errors()),
        Err(err) => {
            let mut errors = scanner.errors();
            errors.push(err);
            (Package::default(), errors)
        }
    }
}
