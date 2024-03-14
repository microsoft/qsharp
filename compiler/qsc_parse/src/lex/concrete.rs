// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::iter::Peekable;

use qsc_data_structures::span::Span;

use super::{cooked, raw};

/// This struct extends cooked tokens to include whitespace and comment tokens.
/// Whitespace and comment tokens were removed during the creation of cooked tokens
/// because they are generally not useful for compilation, but they are reintroduced
/// here because they are needed for formatting.
pub struct ConcreteToken {
    pub kind: ConcreteTokenKind,
    pub span: Span,
}

/// This enum extends the cooked token kind to include whitespace and comment token kinds.
#[derive(Debug, PartialEq)]
pub enum ConcreteTokenKind {
    Syntax(cooked::TokenKind),
    Error(cooked::Error),
    WhiteSpace,
    Comment,
}

/// This is an iterator over `ConcreteTokens`, creating the tokens from a source str.
/// It works by running the cooked lexer on the source str, and iterating over
/// those cooked tokens. Whenever adjacent cooked tokens are found to have a gap
/// between their spans, the raw lexer is run on that slice of the source str to
/// generate the raw tokens (which should only produce the non-compilation whitespace
/// and comment tokens) for that slice, which are iterated over before continuing
/// with the cooked tokens.
pub struct ConcreteTokenIterator<'a> {
    code: &'a str,
    cooked_tokens: Peekable<cooked::Lexer<'a>>,
    non_compilation_tokens: Peekable<raw::Lexer<'a>>,
}

impl<'a> ConcreteTokenIterator<'a> {
    #[must_use]
    pub fn new(code: &'a str) -> Self {
        Self {
            code,
            cooked_tokens: cooked::Lexer::new(code).peekable(),
            non_compilation_tokens: raw::Lexer::new("").peekable(),
        }
    }

    fn get_tokens_from_span(&mut self, lo: u32, hi: u32) {
        let starting_offset = lo;
        let lo = lo as usize;
        let hi = hi as usize;
        if let Some(slice) = self.code.get(lo..hi) {
            self.non_compilation_tokens =
                raw::Lexer::new_with_starting_offset(slice, starting_offset).peekable();
        }
    }

    fn get_next_lo(&mut self) -> u32 {
        match self.non_compilation_tokens.peek() {
            Some(next) => next.offset,
            None => match self.cooked_tokens.peek() {
                Some(next) => match next {
                    Ok(next) => next.span.lo,
                    Err(err) => err.span().lo,
                },
                None => self
                    .code
                    .len()
                    .try_into()
                    .expect("expected length of code to fit into u32"),
            },
        }
    }
}

impl Iterator for ConcreteTokenIterator<'_> {
    type Item = ConcreteToken;

    fn next(&mut self) -> Option<Self::Item> {
        match self.non_compilation_tokens.next() {
            Some(raw_token) => {
                let next_lo = self.get_next_lo();
                let span = Span {
                    lo: raw_token.offset,
                    hi: next_lo,
                };
                let concrete = match raw_token.kind {
                    raw::TokenKind::Comment(_) => ConcreteToken {
                        kind: ConcreteTokenKind::Comment,
                        span,
                    },
                    raw::TokenKind::Whitespace => ConcreteToken {
                        kind: ConcreteTokenKind::WhiteSpace,
                        span,
                    },
                    _ => {
                        return self.next();
                    }
                };
                Some(concrete)
            }
            None => match self.cooked_tokens.next()? {
                Ok(token) => {
                    let next_lo = self.get_next_lo();
                    self.get_tokens_from_span(token.span.hi, next_lo);
                    let syntax = ConcreteToken {
                        kind: ConcreteTokenKind::Syntax(token.kind),
                        span: token.span,
                    };
                    Some(syntax)
                }
                Err(err) => {
                    let next_lo = self.get_next_lo();
                    let span = err.span();
                    self.get_tokens_from_span(span.hi, next_lo);
                    let error = ConcreteToken {
                        kind: ConcreteTokenKind::Error(err),
                        span,
                    };
                    Some(error)
                }
            },
        }
    }
}
