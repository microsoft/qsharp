// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// while we work through the conversion, allow dead code to avoid warnings
#![allow(dead_code)]

pub(crate) mod collector;
#[cfg(test)]
mod tests;
pub(crate) mod word_kinds;

pub(crate) use collector::ValidWordCollector;
pub(crate) use word_kinds::WordKinds;

use super::{prgm, ParserContext};

/// Returns the words that would be valid syntax at a particular offset
/// in the given source file (using the source file parser).
///
/// This is useful for providing completions in an editor.
#[must_use]
pub fn possible_words_at_offset_in_source(input: &str, at_offset: u32) -> WordKinds {
    let mut collector = ValidWordCollector::new(at_offset);
    let mut scanner = ParserContext::with_word_collector(input, &mut collector);
    let _ = prgm::parse(&mut scanner);
    collector.into_words()
}

/// Returns the words that would be valid syntax at a particular offset
/// in the given notebook cell (using the fragments parser).
///
/// This is useful for providing completions in an editor.
#[must_use]
pub fn possible_words_at_offset_in_fragments(input: &str, at_offset: u32) -> WordKinds {
    let mut collector = ValidWordCollector::new(at_offset);
    let mut scanner = ParserContext::with_word_collector(input, &mut collector);
    let _ = prgm::parse(&mut scanner);
    collector.into_words()
}
