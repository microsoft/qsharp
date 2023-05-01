// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::SourceSpan;
use std::{
    fmt::{self, Display, Formatter},
    ops::{Add, Bound, Index, RangeBounds},
};

/// A region between two offsets in an array. Spans are the half-open interval `[lo, hi)`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Span {
    /// The smallest offset contained in the span.
    pub lo: usize,
    /// The next offset after the largest offset contained in the span.
    pub hi: usize,
}

impl Add<usize> for Span {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            lo: self.lo + rhs,
            hi: self.hi + rhs,
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
        &self[index.lo..index.hi]
    }
}

impl Index<&Span> for str {
    type Output = str;

    fn index(&self, index: &Span) -> &Self::Output {
        &self[index.lo..index.hi]
    }
}

impl Index<Span> for String {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.lo..index.hi]
    }
}

impl RangeBounds<usize> for &Span {
    fn start_bound(&self) -> Bound<&usize> {
        Bound::Included(&self.lo)
    }

    fn end_bound(&self) -> Bound<&usize> {
        Bound::Excluded(&self.hi)
    }
}

impl From<Span> for SourceSpan {
    fn from(value: Span) -> Self {
        Self::from(value.lo..value.hi)
    }
}
