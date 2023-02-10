// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod cooked;
mod raw;

// TODO: These will be used via the parser.
#[allow(unused_imports)]
pub(crate) use cooked::{ClosedBinOp, Error, Lexer, Token, TokenKind};

/// A delimiter token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Delim {
    /// `{` or `}`
    Brace,
    /// `[` or `]`
    Bracket,
    /// `(` or `)`
    Paren,
}
