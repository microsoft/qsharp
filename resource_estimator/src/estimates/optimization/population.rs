// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::cmp::Ordering;

pub trait Point<Rhs = Self> {
    fn dominates(&self, other: &Rhs) -> bool;
}

pub struct Point2D<T> {
    pub item: T,
    pub value1: f64,
    pub value2: u64,
}

impl<T> Point2D<T> {
    pub fn new(item: T, value1: f64, value2: u64) -> Self {
        Self {
            item,
            value1,
            value2,
        }
    }
}

impl<T> Point for Point2D<T> {
    fn dominates(&self, other: &Self) -> bool {
        (self.value1 < other.value1 && self.value2 <= other.value2)
            || (self.value1 <= other.value1 && self.value2 < other.value2)
    }
}

impl<T> PartialEq for Point2D<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value1 == other.value1 && self.value2 == other.value2
    }
}

impl<T> Eq for Point2D<T> {}

impl<T> PartialOrd for Point2D<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Point2D<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.value1 < other.value1 {
            Ordering::Less
        } else if self.value1 > other.value1 {
            Ordering::Greater
        } else if self.value2 < other.value2 {
            Ordering::Less
        } else if self.value2 > other.value2 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

pub struct Point4D<T> {
    pub item: T,
    pub value1: f64,
    pub value2: u64,
    pub value3: f64,
    pub value4: u64,
}

impl<T> Point4D<T> {
    pub fn new(item: T, value1: f64, value2: u64, value3: f64, value4: u64) -> Self {
        Self {
            item,
            value1,
            value2,
            value3,
            value4,
        }
    }
}

impl<T> Point for Point4D<T> {
    // Allowing == comparison for better speed compared to abs() + cmp
    #[allow(clippy::float_cmp)]
    fn dominates(&self, other: &Self) -> bool {
        self.value1 <= other.value1
            && self.value2 <= other.value2
            && self.value3 <= other.value3
            && self.value4 <= other.value4
            && !(self.value1 == other.value1
                && self.value2 == other.value2
                && self.value3 == other.value3
                && self.value4 == other.value4)
    }
}

impl<T> PartialEq for Point4D<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value1 == other.value1
            && self.value2 == other.value2
            && self.value3 == other.value3
            && self.value4 == other.value4
    }
}

impl<T> Eq for Point4D<T> {}

impl<T> PartialOrd for Point4D<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Point4D<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.value1 < other.value1 {
            Ordering::Less
        } else if self.value1 > other.value1 {
            Ordering::Greater
        } else if self.value2 < other.value2 {
            Ordering::Less
        } else if self.value2 > other.value2 {
            Ordering::Greater
        } else if self.value3 < other.value3 {
            Ordering::Less
        } else if self.value3 > other.value3 {
            Ordering::Greater
        } else if self.value4 < other.value4 {
            Ordering::Less
        } else if self.value4 > other.value4 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

pub struct Population<P>
where
    P: Point,
    P: Ord,
{
    items: Vec<P>,
    nonexecuted_attempts_to_filter_out_dominated: usize,
}

impl<P> Default for Population<P>
where
    P: Point,
    P: Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<P> Population<P>
where
    P: Point,
    P: Ord,
{
    const MAX_NONEXECUTED_ATTEMPTS_TO_FILTER_OUT_DOMINATED: usize = 1000;

    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            nonexecuted_attempts_to_filter_out_dominated: 0,
        }
    }

    pub fn items(&self) -> &[P] {
        &self.items
    }

    pub(crate) fn push(&mut self, point: P) {
        self.items.push(point);
    }

    pub(crate) fn sort_items(&mut self) {
        self.items.sort_by(|a, b| b.cmp(a));
    }

    pub(crate) fn extract(self) -> Vec<P> {
        self.items
    }

    pub(crate) fn attempt_filter_out_dominated(&mut self) {
        if self.nonexecuted_attempts_to_filter_out_dominated
            <= Self::MAX_NONEXECUTED_ATTEMPTS_TO_FILTER_OUT_DOMINATED
        {
            self.nonexecuted_attempts_to_filter_out_dominated += 1;
        } else {
            self.filter_out_dominated();
        }
    }

    pub(crate) fn filter_out_dominated(&mut self) {
        // NOTE: Use drain_filter in the future
        let mut idx = 0;
        let mut len = self.items.len();
        while idx < len {
            let p = &self.items[idx];
            if self.dominates(p) {
                // remove item by swapping with the end
                self.items.swap(idx, len - 1);
                len -= 1;
            } else {
                idx += 1;
            }
        }
        self.nonexecuted_attempts_to_filter_out_dominated = 0;
        self.items.truncate(len);
    }

    pub(crate) fn dominates(&self, other: &P) -> bool {
        self.items.iter().any(|t| t.dominates(other))
    }
}
