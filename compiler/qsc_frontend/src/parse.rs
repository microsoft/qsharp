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

use crate::lex::TokenKind;
use miette::Diagnostic;
use qsc_ast::ast::{Expr, Package, Span};
use scan::Scanner;
use std::result;
use thiserror::Error;

pub use keyword::Keyword;

#[derive(Clone, Debug, Diagnostic, Error)]
#[error("syntax error: {kind:?}")]
pub struct Error {
    pub kind: ErrorKind,
    #[label("here")]
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
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

pub(super) fn expr(input: &str) -> (Expr, Vec<Error>) {
    let mut scanner = Scanner::new(input);
    match expr::expr(&mut scanner) {
        Ok(expr) => (expr, scanner.errors()),
        Err(err) => {
            let mut errors = scanner.errors();
            errors.push(err);
            (Expr::default(), errors)
        }
    }
}
