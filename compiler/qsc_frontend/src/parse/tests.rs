// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{scan::Scanner, Parser};
use expect_test::Expect;
use std::fmt::Debug;

pub(super) fn check<T: Debug>(mut parser: impl Parser<T>, input: &str, expect: &Expect) {
    let mut scanner = Scanner::new(input);
    let actual = parser(&mut scanner);
    expect.assert_debug_eq(&actual);
}
