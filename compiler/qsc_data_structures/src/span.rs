// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::SourceSpan;
use std::{
    fmt::{self, Display, Formatter},
    ops::{Add, Index, Sub},
};

/// A region between two offsets in an array. Spans are the half-open interval `[lo, hi)`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Span {
    /// The smallest offset contained in the span.
    pub lo: u32,
    /// The next offset after the largest offset contained in the span.
    pub hi: u32,
}

impl Span {
    pub fn range(lo: u32, hi: u32) -> Self {
        Self { lo, hi }
    }
}

impl Add<u32> for Span {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self {
            lo: self.lo + rhs,
            hi: self.hi + rhs,
        }
    }
}

impl Sub<u32> for Span {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        Self {
            lo: self.lo - rhs,
            hi: self.hi - rhs,
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[{}-{}]", self.lo, self.hi)?;
        Ok(())
    }
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[(index.lo as usize)..(index.hi as usize)]
    }
}

impl Index<&Span> for str {
    type Output = str;

    fn index(&self, index: &Span) -> &Self::Output {
        &self[(index.lo as usize)..(index.hi as usize)]
    }
}

impl Index<Span> for String {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[(index.lo as usize)..(index.hi as usize)]
    }
}

impl From<Span> for SourceSpan {
    fn from(value: Span) -> Self {
        Self::from((value.lo as usize)..(value.hi as usize))
    }
}
