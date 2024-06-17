// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::lex::{ClosedBinOp, Error, Lexer, Token, TokenKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TokenW {
    Token(Token),
    Cursor,
    Error(Error),
}

#[derive(PartialEq, Debug)]
enum State {
    Normal,
    Cursor,
    End,
}

pub(crate) struct CursorAwareLexer<'a> {
    pub at_cursor: bool,
    tokens: Lexer<'a>,
    cursor_offset: u32,
    state: State,
    len: usize,
}

impl<'a> CursorAwareLexer<'a> {
    pub(crate) fn new(input: &'a str, cursor_offset: u32) -> Self {
        println!("input:\n{input}");
        Self {
            tokens: Lexer::new(input),
            cursor_offset,
            state: if cursor_offset == 0 {
                State::Cursor
            } else {
                State::Normal
            },
            len: input.len(),
            at_cursor: false,
        }
    }
}

impl Iterator for CursorAwareLexer<'_> {
    type Item = Result<Token, crate::lex::Error>;

    #[allow(clippy::single_match_else)]
    fn next(&mut self) -> Option<Self::Item> {
        let r = match self.state {
            State::Normal => {
                match self.tokens.next() {
                    Some(next_token) => {
                        match next_token {
                            Ok(token) => {
                                println!(
                                    "token: {:?}-{:?} cursor: {:?}",
                                    token.span.lo, token.span.hi, self.cursor_offset
                                );
                                if token.span.lo >= self.cursor_offset {
                                    // We moved past the cursor already, so cursor was in whitespace, comment, or error token
                                    // The distinction is important, but we'll take care of that later.
                                    // For now assume it was whitespace.
                                    // Insert cursor, then end
                                    println!("cursor was in whitespace");
                                    self.state = State::End;
                                    Some(TokenW::Cursor)
                                } else if token.span.lo < self.cursor_offset
                                    && token.span.hi >= self.cursor_offset
                                {
                                    // Cursor is in the middle or end of the next token.
                                    // word token (ident / keyword / "and" / "or") - drop token, cursor, then end
                                    // end of non-word token - insert cursor *after* token, then end
                                    // middle of non-word token (e.g. ==) - no cursor, end
                                    match token.kind {
                                        TokenKind::Ident
                                        | TokenKind::Keyword(_)
                                        | TokenKind::ClosedBinOp(
                                            ClosedBinOp::And | ClosedBinOp::Or,
                                        ) => {
                                            println!("cursor was at end of word");
                                            self.state = State::End;
                                            Some(TokenW::Cursor)
                                        }
                                        _ => {
                                            if token.span.hi == self.cursor_offset {
                                                println!("cursor was at end of nonword");
                                                self.state = State::Cursor;
                                                Some(TokenW::Token(token))
                                            } else {
                                                println!("cursor was at middle of nonword");
                                                self.state = State::End;
                                                None
                                            }
                                        }
                                    }
                                } else {
                                    // State remains State::Normal
                                    Some(TokenW::Token(token))
                                }
                            }
                            Err(e) => Some(TokenW::Error(e)), // State remains State::Normal (cursor could be in this range, need to handle)
                        }
                    }
                    None => {
                        // We got to the end so presumably the cursor was somewhere after the very last token
                        println!("cursor at end ({})", self.len);
                        self.state = State::End;
                        Some(TokenW::Cursor)
                    }
                }
            }
            State::Cursor => {
                println!("advanced past cursor");
                self.state = State::End;
                Some(TokenW::Cursor)
            }
            State::End => {
                println!("at end");
                None
            }
        };

        let (result, at_cursor) = match r {
            Some(t) => match t {
                TokenW::Token(token) => (Some(Ok(token)), false),
                TokenW::Cursor => (None, true),
                TokenW::Error(err) => (Some(Err(err)), false),
            },
            None => (None, false),
        };
        println!("at_cursor {at_cursor:?}");
        self.at_cursor = at_cursor;
        result
    }
}
