// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{scan::Scanner, Parser};
use expect_test::Expect;
use std::fmt::{Debug, Display};

pub(super) fn check<T: Display + Debug>(mut parser: impl Parser<T>, input: &str, expect: &Expect) {
    let mut scanner = Scanner::new(input);
    let actual = parser(&mut scanner);
    match actual {
        Ok(ast) => expect.assert_eq(&ast.to_string()),
        Err(_) => expect.assert_debug_eq(&actual),
    }
}

pub(super) fn check_opt<T: Display + Debug>(
    mut parser: impl Parser<Option<T>>,
    input: &str,
    expect: &Expect,
) {
    let mut scanner = Scanner::new(input);
    let actual = parser(&mut scanner);
    match actual {
        Ok(Some(ast)) => expect.assert_eq(&ast.to_string()),
        _ => expect.assert_debug_eq(&actual),
    }
}

pub(super) fn check_vec<T: Display + Debug>(
    mut parser: impl Parser<Vec<T>>,
    input: &str,
    expect: &Expect,
) {
    let mut scanner = Scanner::new(input);
    let actual = parser(&mut scanner);
    match actual {
        Ok(ast_arr) => expect.assert_eq(
            &ast_arr
                .into_iter()
                .map(|ast| ast.to_string())
                .collect::<Vec<_>>()
                .join(",\n"),
        ),
        Err(_) => expect.assert_debug_eq(&actual),
    }
}

pub(super) fn check_seq<T: Display + Debug, S: Debug>(
    mut parser: impl Parser<(Vec<T>, S)>,
    input: &str,
    expect: &Expect,
) {
    let mut scanner = Scanner::new(input);
    let actual = parser(&mut scanner);
    match actual {
        Ok((ast_arr, sep)) => expect.assert_eq(&format!(
            "({}, {sep:?})",
            &ast_arr
                .into_iter()
                .map(|ast| ast.to_string())
                .collect::<Vec<_>>()
                .join(",\n")
        )),
        Err(_) => expect.assert_debug_eq(&actual),
    }
}
