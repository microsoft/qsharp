// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::iter::Peekable;

use qsc_data_structures::span::Span;

use super::{cooked, raw};

/// This struct extends cooked tokens to include whitespace and comment tokens.
/// Whitespace and comment tokens were removed during the creation of cooked tokens
/// because they are generally not useful for compilation, but they are reintroduced
/// here because they are needed for formatting.
#[derive(Clone, Copy)]
pub struct ConcreteToken {
    pub kind: ConcreteTokenKind,
    pub span: Span,
}

/// This enum extends the cooked token kind to include whitespace and comment token kinds.
#[derive(Clone, Copy, Debug, PartialEq)]
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
        let mut cooked_tokens = cooked::Lexer::new(code).peekable();
        let non_compilation_tokens = match cooked_tokens.peek() {
            Some(first) => {
                let lo = match first {
                    Ok(okay) => okay.span.lo,
                    Err(err) => err.span().lo,
                };
                if lo != 0 {
                    match get_tokens_from_span(code, 0, lo) {
                        Some(iter) => iter,
                        None => raw::Lexer::new("").peekable(),
                    }
                } else {
                    raw::Lexer::new("").peekable()
                }
            }
            None => raw::Lexer::new(code).peekable(),
        };
        Self {
            code,
            cooked_tokens,
            non_compilation_tokens,
        }
    }

    fn get_tokens_from_span(&mut self, lo: u32, hi: u32) {
        if let Some(iter) = get_tokens_from_span(self.code, lo, hi) {
            self.non_compilation_tokens = iter;
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

fn get_tokens_from_span(code: &str, lo: u32, hi: u32) -> Option<Peekable<raw::Lexer<'_>>> {
    let starting_offset = lo;
    let lo = lo as usize;
    let hi = hi as usize;
    code.get(lo..hi)
        .map(|slice| raw::Lexer::new_with_starting_offset(slice, starting_offset).peekable())
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
