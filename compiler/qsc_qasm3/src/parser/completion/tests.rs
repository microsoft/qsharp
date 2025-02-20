// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::completion::possible_words_at_offset_in_source;
use expect_test::expect;

fn get_source_and_cursor(input: &str) -> (String, u32) {
    let mut cursor = -1;
    let mut source = String::new();
    for c in input.chars() {
        if c == '|' {
            cursor = i32::try_from(source.len()).expect("input length should fit into u32");
        } else {
            source.push(c);
        }
    }
    let cursor = u32::try_from(cursor).expect("missing cursor marker in input");
    (source, cursor)
}

fn check_valid_words(input: &str, expect: &expect_test::Expect) {
    let (input, cursor) = get_source_and_cursor(input);
    let w = possible_words_at_offset_in_source(&input, cursor);
    expect.assert_debug_eq(&w);
}

fn check_valid_words_no_source_name(input: &str, expect: &expect_test::Expect) {
    let (input, cursor) = get_source_and_cursor(input);
    let w = possible_words_at_offset_in_source(&input, cursor);
    expect.assert_debug_eq(&w);
}

#[test]
fn begin_document() {
    check_valid_words(
        "|OPENQASM 3;",
        &expect![[r#"
            WordKinds(
                Annotation | Break | Continue | CReg | Def | Extern | For | Gate | If | Include | Input | OpenQASM | Output | Pragma | QReg | Qubit | Return | Switch | While,
            )
        "#]],
    );
}

#[test]
fn end_of_version() {
    check_valid_words(
        "OPENQASM 3;|",
        &expect![[r#"
            WordKinds(
                Annotation | Break | Continue | CReg | Def | Extern | For | Gate | If | Include | Input | Output | Pragma | QReg | Qubit | Return | Switch | While,
            )
        "#]],
    );
}
