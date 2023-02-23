// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod cooked;
mod raw;

use enum_iterator::Sequence;

pub(super) use cooked::{ClosedBinOp, Error, Lexer, Token, TokenKind};

/// A delimiter token.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub(super) enum Delim {
    /// `{` or `}`
    Brace,
    /// `[` or `]`
    Bracket,
    /// `(` or `)`
    Paren,
}
