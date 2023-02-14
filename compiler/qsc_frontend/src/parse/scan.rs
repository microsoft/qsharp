// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::Error;
use crate::lex::{self, Lexer, Token, TokenKind};
use qsc_ast::ast::Span;
use std::result;

pub(super) struct Scanner<'a> {
    input: &'a str,
    tokens: Lexer<'a>,
    errors: Vec<Error>,
    peek: Token,
    offset: usize,
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
            offset: 0,
        }
    }

    pub(super) fn peek(&self) -> Token {
        self.peek
    }

    pub(super) fn read(&self) -> &str {
        &self.input[self.peek.span]
    }

    pub(super) fn advance(&mut self) {
        if self.peek.kind != TokenKind::Eof {
            self.offset = self.peek.span.hi;
            let (peek, errors) = next_ok(&mut self.tokens);
            self.errors.extend(errors.iter().map(lex_error));
            self.peek = peek.unwrap_or_else(|| eof(self.input.len()));
        }
    }

    pub(super) fn span(&self, from: usize) -> Span {
        Span {
            lo: from,
            hi: self.offset,
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
