// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::completion::possible_words_at_offset_in_source;
use expect_test::expect;
use qsc_data_structures::language_features::LanguageFeatures;

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
    let w = possible_words_at_offset_in_source(
        &input,
        Some("test"),
        LanguageFeatures::default(),
        cursor,
    );
    expect.assert_debug_eq(&w);
}

fn check_valid_words_no_source_name(input: &str, expect: &expect_test::Expect) {
    let (input, cursor) = get_source_and_cursor(input);
    let w = possible_words_at_offset_in_source(&input, None, LanguageFeatures::default(), cursor);
    expect.assert_debug_eq(&w);
}

#[test]
fn end_of_keyword() {
    check_valid_words(
        "namespace Foo { open| ",
        &expect![[r"
            WordKinds(
                Export | Function | Import | Internal | Newtype | Open | Operation | Struct,
            )
        "]],
    );
}

#[test]
fn after_open() {
    check_valid_words(
        "namespace Foo { open |",
        &expect![[r"
            WordKinds(
                PathNamespace,
            )
        "]],
    );
}

#[test]
fn begin_ident() {
    check_valid_words(
        "namespace Foo { open |X",
        // right at the beginning of the namespace name.
        &expect![[r"
            WordKinds(
                PathNamespace,
            )
        "]],
    );
}

#[test]
fn middle_ident() {
    check_valid_words(
        "namespace Foo { open AB|CD",
        &expect![[r"
            WordKinds(
                PathNamespace,
            )
        "]],
    );
}

#[test]
fn end_ident() {
    check_valid_words(
        "namespace Foo { open ABCD| ",
        &expect![[r"
            WordKinds(
                PathNamespace,
            )
        "]],
    );
}

#[test]
fn middle() {
    check_valid_words(
        "namespace Foo { open AB|CD; open Foo; operation Main() : Unit {} }",
        &expect![[r"
            WordKinds(
                PathNamespace,
            )
        "]],
    );
}

#[test]
fn in_whitespace() {
    check_valid_words(
        "namespace MyQuantumApp { open Microsoft.Quantum.Diagnostics; |     }",
        &expect![[r"
            WordKinds(
                Export | Function | Import | Internal | Newtype | Open | Operation | Struct,
            )
        "]],
    );
}

#[test]
fn after_semicolon() {
    check_valid_words(
        "namespace MyQuantumApp { open Microsoft.Quantum.Diagnostics;|      }",
        &expect![[r"
            WordKinds(
                Export | Function | Import | Internal | Newtype | Open | Operation | Struct,
            )
        "]],
    );
}

#[test]
fn whitespace_at_end() {
    check_valid_words(
        "namespace Foo { open |    ",
        &expect![[r"
            WordKinds(
                PathNamespace,
            )
        "]],
    );
}

#[test]
fn path_part() {
    check_valid_words(
        "namespace Foo { operation Main() : Unit { Foo.| } }",
        &expect![[r"
            WordKinds(
                PathSegment,
            )
        "]],
    );
}

#[test]
fn namespace_part() {
    check_valid_words(
        "namespace Foo { open Foo.| }",
        &expect![[r"
            WordKinds(
                PathSegment,
            )
        "]],
    );
}

#[test]
fn type_position() {
    check_valid_words(
        "namespace Foo { operation Main() : Unit { let x:| ; } }",
        &expect![[r"
            WordKinds(
                PathTy | TyParam | Underscore,
            )
        "]],
    );
}

#[test]
fn namespace_declaration() {
    check_valid_words(
        "namespace |",
        // No word kinds expected here.
        &expect![[r"
            WordKinds(
                0x0,
            )
        "]],
    );
}

#[test]
fn empty_source_no_source_name() {
    check_valid_words_no_source_name(
        "|",
        &expect![[r"
            WordKinds(
                Namespace,
            )
        "]],
    );
}

#[test]
fn implicit_namespace_items_empty() {
    check_valid_words(
        "|",
        &expect![[r"
            WordKinds(
                Export | Function | Import | Internal | Namespace | Newtype | Open | Operation | Struct,
            )
        "]],
    );
}

#[test]
fn implicit_namespace_items_beginning() {
    // Ideally, `Namespace` would not be in the list since we would gather from context
    // that we're already in an implicit namespace. However, given the current design
    // of the expected word collector, we don't have that context at the cursor location.
    // (Note that the test cases is equivalent to the empty source file case, since we
    // never look further than the cursor location).
    check_valid_words(
        "| operation Foo() : Unit {}",
        &expect![[r"
            WordKinds(
                Export | Function | Import | Internal | Namespace | Newtype | Open | Operation | Struct,
            )
        "]],
    );
}

#[test]
fn implicit_namespace_items_end() {
    check_valid_words(
        "operation Foo() : Unit {} |",
        &expect![[r"
            WordKinds(
                Export | Function | Import | Internal | Newtype | Open | Operation | Struct,
            )
        "]],
    );
}

#[test]
fn import_missing_semi() {
    check_valid_words(
        "import | operation Foo()",
        &expect![[r"
            WordKinds(
                PathImport,
            )
        "]],
    );
}

#[test]
fn import_with_semi() {
    check_valid_words(
        "import | ; operation Foo()",
        &expect![[r"
            WordKinds(
                PathImport,
            )
        "]],
    );
}

#[test]
fn import_eof() {
    check_valid_words(
        "import |",
        &expect![[r"
            WordKinds(
                PathImport,
            )
            "]],
    );
}

#[test]
fn import_end_of_namespace() {
    check_valid_words(
        "namespace Foo { import | }",
        &expect![[r"
            WordKinds(
                PathImport,
            )
        "]],
    );
}

#[test]
fn export_part_missing_semi() {
    check_valid_words(
        "export Foo.| operation Foo()",
        &expect![[r"
            WordKinds(
                PathSegment,
            )
        "]],
    );
}

#[test]
fn export_part_with_semi() {
    check_valid_words(
        "export Foo.| ; operation Foo()",
        &expect![[r"
            WordKinds(
                PathSegment,
            )
        "]],
    );
}

#[test]
fn export_part_eof() {
    check_valid_words(
        "export Foo. |",
        &expect![[r"
            WordKinds(
                PathSegment,
            )
        "]],
    );
}

#[test]
fn export_part_end_of_namespace() {
    check_valid_words(
        "namespace Bar { export Foo.| }",
        &expect![[r"
            WordKinds(
                PathSegment,
            )
        "]],
    );
}

#[test]
fn base_type() {
    check_valid_words(
        "newtype Foo = |",
        &expect![[r"
            WordKinds(
                PathTy | TyParam | Underscore,
            )
        "]],
    );
}

#[test]
fn base_type_tuple() {
    check_valid_words(
        "newtype Foo = (|",
        &expect![[r"
            WordKinds(
                PathTy | TyParam | Underscore,
            )
        "]],
    );
}

#[test]
fn keyword_after_incomplete_path() {
    check_valid_words(
        "import Foo.in|",
        &expect![[r"
            WordKinds(
                PathSegment,
            )
        "]],
    );
}
