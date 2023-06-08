// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{fmt, rc::Rc};

/// Many LLVM objects have a `Name`, which is either a string name, or just a
/// sequential numbering (e.g. `%3`).
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash)]
pub enum Name {
    /// has a string name
    Name(Rc<str>),
    /// doesn't have a string name and was given this sequential number
    Number(usize),
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Name::Name(s.into())
    }
}

impl From<usize> for Name {
    fn from(u: usize) -> Self {
        Name::Number(u)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Name::Name(s) => write!(f, "%{s}"),
            Name::Number(n) => write!(f, "%{n}"),
        }
    }
}
