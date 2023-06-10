// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The qsc parser uses recursive descent to handle turning an input string into a Q# abstract syntax tree.
//! The parser produces a tree with placeholder node identifiers that are expected to be replaced with
//! unique identifiers by a later stage.

mod expr;
mod keyword;
mod lex;
mod prim;
mod scan;
mod stmt;
#[cfg(test)]
mod tests;
mod top;
mod ty;

use keyword::Keyword;
use lex::TokenKind;
use miette::Diagnostic;
use qsc_ast::ast::{Expr, Namespace};
use qsc_data_structures::span::Span;
use scan::Scanner;
use std::result;
use thiserror::Error;

pub use top::Fragment;

#[derive(Clone, Copy, Debug, Diagnostic, Eq, Error, PartialEq)]
#[error(transparent)]
#[diagnostic(transparent)]
pub struct Error(ErrorKind);

impl Error {
    pub fn with_offset(self, offset: u32) -> Self {
        Self(self.0.with_offset(offset))
    }
}

#[derive(Clone, Copy, Debug, Diagnostic, Eq, Error, PartialEq)]
enum ErrorKind {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Lex(lex::Error),
    #[error("invalid {0} literal")]
    Lit(&'static str, #[label] Span),
    #[error("unknown escape sequence: `{0}`")]
    Escape(char, #[label] Span),
    #[error("expected {0}, found {1}")]
    Token(TokenKind, TokenKind, #[label] Span),
    #[error("expected keyword `{0}`, found {1}")]
    Keyword(Keyword, TokenKind, #[label] Span),
    #[error("expected {0}, found {1}")]
    Rule(&'static str, TokenKind, #[label] Span),
    #[error("expected {0}, found keyword `{1}`")]
    RuleKeyword(&'static str, Keyword, #[label] Span),
    #[error("expected {0}, found {1}")]
    Convert(&'static str, &'static str, #[label] Span),
    #[error("expected statement to end with a semicolon")]
    MissingSemi(#[label] Span),
}

impl ErrorKind {
    fn with_offset(self, offset: u32) -> Self {
        match self {
            Self::Lex(error) => Self::Lex(error.with_offset(offset)),
            Self::Lit(name, span) => Self::Lit(name, span + offset),
            Self::Escape(ch, span) => Self::Escape(ch, span + offset),
            Self::Token(expected, actual, span) => Self::Token(expected, actual, span + offset),
            Self::Keyword(keyword, token, span) => Self::Keyword(keyword, token, span + offset),
            Self::Rule(name, token, span) => Self::Rule(name, token, span + offset),
            Self::RuleKeyword(name, keyword, span) => {
                Self::RuleKeyword(name, keyword, span + offset)
            }
            Self::Convert(expected, actual, span) => Self::Convert(expected, actual, span + offset),
            Self::MissingSemi(span) => Self::MissingSemi(span + offset),
        }
    }
}

type Result<T> = result::Result<T, Error>;

trait Parser<T>: FnMut(&mut Scanner) -> Result<T> {}

impl<T, F: FnMut(&mut Scanner) -> Result<T>> Parser<T> for F {}

pub fn namespaces(input: &str) -> (Vec<Namespace>, Vec<Error>) {
    let mut scanner = Scanner::new(input);
    match top::namespaces(&mut scanner) {
        Ok(namespaces) => (namespaces, scanner.into_errors()),
        Err(error) => {
            let mut errors = scanner.into_errors();
            errors.push(error);
            (Vec::new(), errors)
        }
    }
}

pub fn fragments(input: &str) -> (Vec<Fragment>, Vec<Error>) {
    let mut scanner = Scanner::new(input);
    match top::fragments(&mut scanner) {
        Ok(fragments) => (fragments, scanner.into_errors()),
        Err(error) => {
            let mut errors = scanner.into_errors();
            errors.push(error);
            (Vec::new(), errors)
        }
    }
}

pub fn expr(input: &str) -> (Box<Expr>, Vec<Error>) {
    let mut scanner = Scanner::new(input);
    match expr::expr(&mut scanner) {
        Ok(expr) => (expr, scanner.into_errors()),
        Err(error) => {
            let mut errors = scanner.into_errors();
            errors.push(error);
            (Box::default(), errors)
        }
    }
}
