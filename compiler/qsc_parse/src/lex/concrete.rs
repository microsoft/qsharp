// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::iter::Peekable;

use qsc_data_structures::span::Span;

use super::{cooked, raw};

pub(super) struct ConcreteToken {
    pub kind: ConcreteTokenKind,
    pub span: Span,
}

#[derive(PartialEq)]
pub(super) enum ConcreteTokenKind {
    Cooked(cooked::TokenKind),
    Error(cooked::Error),
    WhiteSpace,
    Comment,
}

pub(super) struct ConcreteTokenIterator<'a> {
    code: &'a str,
    cooked_tokens: Peekable<cooked::Lexer<'a>>,
    non_compilation_tokens: Peekable<raw::Lexer<'a>>,
}

impl<'a> ConcreteTokenIterator<'a> {
    pub(super) fn new(code: &'a str) -> Self {
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
                    Err(err) => err.get_span().lo,
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
                    _ => panic!("only comments and whitespace should be non-compilable tokens"), // Todo: might need better handling
                };
                Some(concrete)
            }
            None => match self.cooked_tokens.next()? {
                Ok(token) => {
                    let next_lo = self.get_next_lo();
                    self.get_tokens_from_span(token.span.hi, next_lo);
                    Some(ConcreteToken {
                        kind: ConcreteTokenKind::Cooked(token.kind),
                        span: token.span,
                    })
                }
                Err(err) => {
                    let next_lo = self.get_next_lo();
                    let span = err.get_span();
                    self.get_tokens_from_span(span.hi, next_lo);
                    Some(ConcreteToken {
                        kind: ConcreteTokenKind::Error(err),
                        span,
                    })
                }
            },
        }
    }
}
