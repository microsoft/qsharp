// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod cooked;
pub mod raw;

use enum_iterator::Sequence;

pub use cooked::{ClosedBinOp, TokenKind};
pub(super) use cooked::{Error, Lexer, Token};

/// A delimiter token.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum Delim {
    /// `{` or `}`
    Brace,
    /// `[` or `]`
    Bracket,
    /// `(` or `)`
    Paren,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum Radix {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

impl From<Radix> for u32 {
    fn from(value: Radix) -> Self {
        match value {
            Radix::Binary => 2,
            Radix::Octal => 8,
            Radix::Decimal => 10,
            Radix::Hexadecimal => 16,
        }
    }
}
