// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use enum_iterator::Sequence;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Sequence)]
pub enum Keyword {
    OpenQASM,
    Include,
    Defcalgrammar,
    Def,
    Cal,
    Gate,
    Extern,
    Box,
    Let,

    Break,
    Continue,
    If,
    Else,
    End,
    Return,
    For,
    While,
    In,
    Switch,
    Case,
    Default,

    Pragma,
    Annotation,
}

impl Keyword {
    pub(super) fn as_str(self) -> &'static str {
        todo!()
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
            "openqasm" => Ok(Self::OpenQASM),
            "include" => Ok(Self::Include),
            "defcalgrammar" => Ok(Self::Defcalgrammar),
            "def" => Ok(Self::Def),
            "cal" => Ok(Self::Cal),
            "gate" => Ok(Self::Gate),
            "extern" => Ok(Self::Extern),
            "box" => Ok(Self::Box),
            "let" => Ok(Self::Let),
            "break" => Ok(Self::Break),
            "continue" => Ok(Self::Continue),
            "if" => Ok(Self::If),
            "else" => Ok(Self::Else),
            "end" => Ok(Self::End),
            "return" => Ok(Self::Return),
            "for" => Ok(Self::For),
            "while" => Ok(Self::While),
            "in" => Ok(Self::In),
            "switch" => Ok(Self::Switch),
            "case" => Ok(Self::Case),
            "default" => Ok(Self::Default),
            "pragma" => Ok(Self::Pragma),
            "annotation" => Ok(Self::Annotation),
            _ => Err(()),
        }
    }
}
