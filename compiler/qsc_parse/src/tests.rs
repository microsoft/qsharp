// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{scan::ParserContext, Parser};
use crate::prim::FinalSep;
use expect_test::Expect;
use qsc_data_structures::language_features::LanguageFeatures;
use std::fmt::Display;

mod implicit_namespace;

pub(super) fn check<T: Display>(parser: impl Parser<T>, input: &str, expect: &Expect) {
    check_map(parser, input, expect, ToString::to_string);
}

pub(super) fn check_opt<T: Display>(parser: impl Parser<Option<T>>, input: &str, expect: &Expect) {
    check_map(parser, input, expect, |value| match value {
        Some(value) => value.to_string(),
        None => "None".to_string(),
    });
}

pub(super) fn check_vec<T: Display>(parser: impl Parser<Vec<T>>, input: &str, expect: &Expect) {
    check_map(parser, input, expect, |values| {
        values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",\n")
    });
}

/// This function is the same as `check_vec`, but it uses the v2 preview syntax language feature.
pub(super) fn check_vec_v2_preview<T: Display>(
    parser: impl Parser<Vec<T>>,
    input: &str,
    expect: &Expect,
) {
    check_map_v2_preview(parser, input, expect, |values| {
        values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",\n")
    });
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
    });
}

/// This function is the same as `check_map`, but it uses the v2 preview syntax language feature.
fn check_map_v2_preview<T>(
    mut parser: impl Parser<T>,
    input: &str,
    expect: &Expect,
    f: impl FnOnce(&T) -> String,
) {
    let mut scanner = ParserContext::new(input, LanguageFeatures::V2PreviewSyntax);
    let result = parser(&mut scanner);
    let errors = scanner.into_errors();
    match result {
        Ok(value) if errors.is_empty() => expect.assert_eq(&f(&value)),
        Ok(value) => expect.assert_eq(&format!("{}\n\n{errors:#?}", f(&value))),
        Err(error) if errors.is_empty() => expect.assert_debug_eq(&error),
        Err(error) => expect.assert_eq(&format!("{error:#?}\n\n{errors:#?}")),
    }
}

fn check_map<T>(
    mut parser: impl Parser<T>,
    input: &str,
    expect: &Expect,
    f: impl FnOnce(&T) -> String,
) {
    let mut scanner = ParserContext::new(input, LanguageFeatures::default());
    let result = parser(&mut scanner);
    let errors = scanner.into_errors();
    match result {
        Ok(value) if errors.is_empty() => expect.assert_eq(&f(&value)),
        Ok(value) => expect.assert_eq(&format!("{}\n\n{errors:#?}", f(&value))),
        Err(error) if errors.is_empty() => expect.assert_debug_eq(&error),
        Err(error) => expect.assert_eq(&format!("{error:#?}\n\n{errors:#?}")),
    }
}

#[test]
fn test_completion_end_of_keyword() {
    let input = "namespace Foo { open ".to_string();
    let cursor = 20_u32;
    let mut scanner = ParserContext::predict_mode(&input, cursor);
    let _ = crate::item::parse_namespaces(&mut scanner);
    let v = scanner.into_predictions();

    assert_eq!(format!("{v:?}"), "[Keyword(\"internal\"), Keyword(\"open\"), Keyword(\"newtype\"), Keyword(\"function\"), Keyword(\"operation\")]");
}

#[test]
fn test_completion_after_open() {
    let input = "namespace Foo { open ".to_string();
    let cursor = 21_u32;
    let mut scanner = ParserContext::predict_mode(&input, cursor);
    let _ = crate::item::parse_namespaces(&mut scanner);
    let v = scanner.into_predictions();

    // a namespace follows the open keyword
    assert_eq!(format!("{v:?}"), "[Namespace]");
}

#[test]
fn test_completion_begin_ident() {
    let input = "namespace Foo { open X".to_string();
    let cursor = 21_u32;
    let mut scanner = ParserContext::predict_mode(&input, cursor);
    let _ = crate::item::parse_namespaces(&mut scanner);
    let v = scanner.into_predictions();

    // right at the beginning of the namespace name.
    assert_eq!(format!("{v:?}"), "[Namespace]");
}

#[test]
fn test_completion_middle_ident() {
    let input = "namespace Foo { open ABCD".to_string();
    let cursor = 23_u32;
    let mut scanner = ParserContext::predict_mode(&input, cursor);
    let _ = crate::item::parse_namespaces(&mut scanner);
    let v = scanner.into_predictions();

    // middle of the namespace name
    assert_eq!(format!("{v:?}"), "[Namespace]");
}

#[test]
fn test_completion_end_ident() {
    let input = "namespace Foo { open ABCD ".to_string();
    let cursor = 25_u32;
    let mut scanner = ParserContext::predict_mode(&input, cursor);
    let _ = crate::item::parse_namespaces(&mut scanner);
    let v = scanner.into_predictions();

    // end of the namespace name
    assert_eq!(format!("{v:?}"), "[Namespace]");
}

#[test]
fn test_completion_middle() {
    let input = "namespace Foo { open ABCD; open Foo; operation Main() : Unit {} }".to_string();
    let cursor = 23_u32;
    let mut scanner = ParserContext::predict_mode(&input, cursor);
    let _ = crate::item::parse_namespaces(&mut scanner);
    let v = scanner.into_predictions();

    assert_eq!(format!("{v:?}"), "[Namespace]");
}

#[test]
fn test_completion_lotsawhitespace() {
    let input = r"namespace MyQuantumApp { open Microsoft.Quantum.Diagnostics;      }".to_string();
    let cursor = 61_u32;
    let mut scanner = ParserContext::predict_mode(&input, cursor);
    let _ = crate::item::parse_namespaces(&mut scanner);
    let v = scanner.into_predictions();

    assert_eq!(format!("{v:?}"), "[Keyword(\"internal\"), Keyword(\"open\"), Keyword(\"newtype\"), Keyword(\"function\"), Keyword(\"operation\")]");
}

#[test]
fn test_completion_after_semicolon() {
    let input = r"namespace MyQuantumApp { open Microsoft.Quantum.Diagnostics;      }".to_string();
    let cursor = 60_u32;
    let mut scanner = ParserContext::predict_mode(&input, cursor);
    let _ = crate::item::parse_namespaces(&mut scanner);
    let v = scanner.into_predictions();

    assert_eq!(format!("{v:?}"), "[Keyword(\"internal\"), Keyword(\"open\"), Keyword(\"newtype\"), Keyword(\"function\"), Keyword(\"operation\")]");
}

#[test]
fn test_completion_before_attr() {
    let input =
        r"namespace Foo { open Microsoft.Quantum.Diagnostics;          @EntryPoint() operation Main() : Unit {} }".to_string();
    let cursor = 55_u32;
    let mut scanner = ParserContext::predict_mode(&input, cursor);
    let _ = crate::item::parse_namespaces(&mut scanner);
    let v = scanner.into_predictions();

    assert_eq!(format!("{v:?}"), "[Keyword(\"internal\"), Keyword(\"open\"), Keyword(\"newtype\"), Keyword(\"function\"), Keyword(\"operation\")]");
}

#[test]
fn test_completion_whitespace_at_end() {
    let input = "namespace Foo { open     ".to_string();
    let cursor = 21_u32;
    let mut scanner = ParserContext::predict_mode(&input, cursor);
    let _ = crate::item::parse_namespaces(&mut scanner);
    let v = scanner.into_predictions();

    // a namespace follows the open keyword
    assert_eq!(format!("{v:?}"), "[Namespace]");
}

#[test]
fn test_completion_empty_source() {
    let input = String::new();
    let cursor = 0_u32;
    let mut scanner = ParserContext::predict_mode(&input, cursor);
    let _ = crate::item::parse_namespaces(&mut scanner);
    let v = scanner.into_predictions();

    // a namespace follows the open keyword
    assert_eq!(format!("{v:?}"), "[Keyword(\"namespace\")]");
}
