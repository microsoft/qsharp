// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use enum_iterator::Sequence;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Sequence)]
pub enum Keyword {
    Box,
    Break,
    Cal,
    Case,
    Continue,
    Def,
    Default,
    Defcalgrammar,
    Else,
    End,
    Extern,
    For,
    Gate,
    If,
    In,
    Include,
    Let,
    OpenQASM,
    Pragma,
    Return,
    Switch,
    While,
}

impl Keyword {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Keyword::Box => "box",
            Keyword::Break => "break",
            Keyword::Cal => "cal",
            Keyword::Case => "case",
            Keyword::Continue => "continue",
            Keyword::Def => "def",
            Keyword::Default => "default",
            Keyword::Defcalgrammar => "defcalgrammar",
            Keyword::Else => "else",
            Keyword::End => "end",
            Keyword::Extern => "extern",
            Keyword::For => "for",
            Keyword::Gate => "gate",
            Keyword::If => "if",
            Keyword::In => "in",
            Keyword::Include => "include",
            Keyword::Let => "let",
            Keyword::OpenQASM => "openqasm",
            Keyword::Pragma => "pragma",
            Keyword::Return => "return",
            Keyword::Switch => "switch",
            Keyword::While => "while",
        }
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Keyword {
    type Err = ();

    // This is a hot function. Use a match expression so that the Rust compiler
    // can optimize the string comparisons better, and order the cases by
    // frequency in Q# so that fewer comparisons are needed on average.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "box" => Ok(Self::Box),
            "break" => Ok(Self::Break),
            "cal" => Ok(Self::Cal),
            "case" => Ok(Self::Case),
            "continue" => Ok(Self::Continue),
            "def" => Ok(Self::Def),
            "default" => Ok(Self::Default),
            "defcalgrammar" => Ok(Self::Defcalgrammar),
            "else" => Ok(Self::Else),
            "end" => Ok(Self::End),
            "extern" => Ok(Self::Extern),
            "for" => Ok(Self::For),
            "gate" => Ok(Self::Gate),
            "if" => Ok(Self::If),
            "in" => Ok(Self::In),
            "include" => Ok(Self::Include),
            "let" => Ok(Self::Let),
            "openqasm" => Ok(Self::OpenQASM),
            "pragma" => Ok(Self::Pragma),
            "return" => Ok(Self::Return),
            "switch" => Ok(Self::Switch),
            "while" => Ok(Self::While),
            _ => Err(()),
        }
    }
}
