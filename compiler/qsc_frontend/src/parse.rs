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

use crate::lex::{self, TokenKind};
use miette::Diagnostic;
use qsc_ast::ast::{Item, Namespace, Span, Stmt};
use scan::Scanner;
use std::result;
use thiserror::Error;

pub(super) use keyword::Keyword;

#[derive(Clone, Copy, Debug, Diagnostic, Eq, Error, PartialEq)]
pub(super) enum Error {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Lex(lex::Error),
    #[error("expected {0}, found {1}")]
    Token(TokenKind, TokenKind, #[label("expected {0}")] Span),
    #[error("expected keyword `{0}`, found {1}")]
    Keyword(Keyword, TokenKind, #[label("expected keyword `{0}`")] Span),
    #[error("expected {0}, found {1}")]
    Rule(&'static str, TokenKind, #[label("expected {0}")] Span),
    #[error("expected {0}, found keyword `{1}`")]
    RuleKeyword(&'static str, Keyword, #[label("expected {0}")] Span),
    #[error("expected {0}, found {1}")]
    Convert(&'static str, &'static str, #[label("expected {0}")] Span),
}

pub(super) type Result<T> = result::Result<T, Error>;

trait Parser<T>: FnMut(&mut Scanner) -> Result<T> {}

impl<T, F: FnMut(&mut Scanner) -> Result<T>> Parser<T> for F {}

pub(super) fn namespaces(input: &str) -> (Vec<Namespace>, Vec<Error>) {
    let mut scanner = Scanner::new(input);
    match top::namespaces(&mut scanner) {
        Ok(namespaces) => (namespaces, scanner.errors()),
        Err(err) => {
            let mut errors = scanner.errors();
            errors.push(err);
            (Vec::new(), errors)
        }
    }
}

pub(super) fn item(input: &str) -> (Item, Vec<Error>) {
    let mut scanner = Scanner::new(input);
    match top::item(&mut scanner) {
        Ok(item) => (item, scanner.errors()),
        Err(err) => {
            let mut errors = scanner.errors();
            errors.push(err);
            (Item::default(), errors)
        }
    }
}

#[must_use]
pub(super) fn stmt(input: &str) -> (Stmt, Vec<Error>) {
    let mut scanner = Scanner::new(input);
    match stmt::stmt(&mut scanner) {
        Ok(stmt) => (stmt, scanner.errors()),
        Err(err) => {
            let mut errors = scanner.errors();
            errors.push(err);
            (Stmt::default(), errors)
        }
    }
}
