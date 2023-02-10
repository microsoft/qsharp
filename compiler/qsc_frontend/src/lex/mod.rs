// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod cooked;
mod raw;

use enum_iterator::Sequence;

// TODO: These will be used via the parser.
#[allow(unused_imports)]
pub(crate) use cooked::{ClosedBinOp, Error, Lexer, Token, TokenKind};

/// A delimiter token.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub(crate) enum Delim {
    /// `{` or `}`
    Brace,
    /// `[` or `]`
    Bracket,
    /// `(` or `)`
    Paren,
}
