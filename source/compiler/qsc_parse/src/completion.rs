// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(crate) mod collector;
#[cfg(test)]
mod tests;
mod word_kinds;

use crate::{item, scan::ParserContext};
use collector::ValidWordCollector;
use qsc_data_structures::language_features::LanguageFeatures;
pub use word_kinds::*;

/// Returns the words that would be valid syntax at a particular offset
/// in the given source file (using the source file parser).
///
/// This is useful for providing completions in an editor.
#[must_use]
pub fn possible_words_at_offset_in_source(
    input: &str,
    source_name: Option<&str>,
    language_features: LanguageFeatures,
    at_offset: u32,
) -> WordKinds {
    let mut collector = ValidWordCollector::new(at_offset);
    let mut scanner = ParserContext::with_word_collector(input, language_features, &mut collector);
    let _ = item::parse_namespaces_or_implicit(&mut scanner, source_name);
    collector.into_words()
}

/// Returns the words that would be valid syntax at a particular offset
/// in the given notebook cell (using the fragments parser).
///
/// This is useful for providing completions in an editor.
#[must_use]
pub fn possible_words_at_offset_in_fragments(
    input: &str,
    language_features: LanguageFeatures,
    at_offset: u32,
) -> WordKinds {
    let mut collector = ValidWordCollector::new(at_offset);
    let mut scanner = ParserContext::with_word_collector(input, language_features, &mut collector);
    let _ = item::parse_top_level_nodes(&mut scanner);
    collector.into_words()
}
