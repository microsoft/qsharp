// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{scan::ParserConfig, Parser};
use crate::prim::FinalSep;
use expect_test::Expect;
use qsc_data_structures::language_features::{LanguageFeature, LanguageFeatures};
use std::{collections::BTreeSet, fmt::Display, vec};

pub(super) fn check<T: Display>(parser: impl Parser<T>, input: &str, expect: &Expect) {
    check_map(parser, input, expect, ToString::to_string, Default::default());
}

pub(super) fn check_opt<T: Display>(parser: impl Parser<Option<T>>, input: &str, expect: &Expect) {
    check_map(parser, input, expect, |value| match value {
        Some(value) => value.to_string(),
        None => "None".to_string(),
    }, Default::default());
}


pub(super) fn check_vec<T: Display>(parser: impl Parser<Vec<T>>, input: &str, expect: &Expect) {
    check_map(parser, input, expect, |values| {
        values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",\n")
    }, Default::default());
}

pub(super) fn check_seq<T: Display>(
    parser: impl Parser<(Vec<T>, FinalSep)>,
    input: &str,
    expect: &Expect,
) {
    check_map(parser, input, expect, |(values, sep)| {
        format!(
            "({}, {sep:?})",
            values
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",\n")
        )
    }, Default::default());
}

fn check_map<T>(
    mut parser: impl Parser<T>,
    input: &str,
    expect: &Expect,
    f: impl FnOnce(&T) -> String,
    features: LanguageFeatures,
) {
    let mut scanner = ParserConfig::new(input, Default::default());
    let result = parser(&mut scanner);
    let errors = scanner.into_errors();
    match result {
        Ok(value) if errors.is_empty() => expect.assert_eq(&f(&value)),
        Ok(value) => expect.assert_eq(&format!("{}\n\n{errors:#?}", f(&value))),
        Err(error) if errors.is_empty() => expect.assert_debug_eq(&error),
        Err(error) => expect.assert_eq(&format!("{error:#?}\n\n{errors:#?}")),
    }
}
