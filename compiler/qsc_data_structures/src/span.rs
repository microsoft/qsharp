// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::SourceSpan;
use std::{
    fmt::{self, Display, Formatter},
    ops::{Bound, Index, RangeBounds},
};

/// A region between two source code positions. Spans are the half-open interval `[lo, hi)`. The
/// offsets are absolute within an AST, assuming that each file has its own offset.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Span {
    /// The offset of the first byte.
    pub lo: usize,
    /// The offset immediately following the last byte.
    pub hi: usize,
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
