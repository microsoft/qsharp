// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{kw, Error, Result};
use crate::lex::{self, Lexer, Token, TokenKind};
use qsc_ast::ast::Span;
use std::result;

pub(super) struct Scanner<'a> {
    input: &'a str,
    tokens: Lexer<'a>,
    errors: Vec<Error>,
    peek: Token,
    span: Span,
}

impl<'a> Scanner<'a> {
    pub(super) fn new(input: &'a str) -> Self {
        let mut tokens = Lexer::new(input);
        let (peek, errors) = next_ok(&mut tokens);
        Self {
            input,
            tokens,
            errors: errors.iter().map(lex_error).collect(),
            peek: peek.unwrap_or_else(|| eof(input.len())),
            span: Span { lo: 0, hi: 0 },
        }
    }

    pub(super) fn error(&self, message: String) -> Error {
        Error {
            message,
            span: self.peek.span,
        }
    }

    pub(super) fn errors(self) -> Vec<Error> {
        self.errors
    }

    pub(super) fn span(&self) -> Span {
        self.span
    }

    pub(super) fn expect(&mut self, kind: TokenKind) -> Result<()> {
        if self.peek.kind == kind {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("Expecting {kind:?}.")))
        }
    }

    pub(super) fn keyword(&mut self, kw: &str) -> Result<()> {
        if kw::is_keyword(kw)
            && self.peek.kind == TokenKind::Ident
            && &self.input[self.peek.span] == kw
        {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("Expecting keyword `{kw}`.")))
        }
    }

    pub(super) fn ident(&mut self) -> Result<&str> {
        if self.peek.kind == TokenKind::Ident && !kw::is_keyword(&self.input[self.peek.span]) {
            let name = &self.input[self.peek.span];
            self.advance();
            Ok(name)
        } else {
            Err(self.error("Expecting identifier.".to_string()))
        }
    }

    fn advance(&mut self) {
        if self.peek.kind != TokenKind::Eof {
            self.span = self.peek.span;
            let (peek, errors) = next_ok(&mut self.tokens);
            self.errors.extend(errors.iter().map(lex_error));
            self.peek = peek.unwrap_or_else(|| eof(self.input.len()));
        }
    }
}

fn eof(offset: usize) -> Token {
    Token {
        kind: TokenKind::Eof,
        span: Span {
            lo: offset,
            hi: offset,
        },
    }
}

fn next_ok<T, E>(iter: impl Iterator<Item = result::Result<T, E>>) -> (Option<T>, Vec<E>) {
    let mut errors = Vec::new();
    for result in iter {
        match result {
            Ok(v) => return (Some(v), errors),
            Err(e) => errors.push(e),
        }
    }

    (None, errors)
}

fn lex_error(error: &lex::Error) -> Error {
    Error {
        message: error.message.to_string(),
        span: error.span,
    }
}
