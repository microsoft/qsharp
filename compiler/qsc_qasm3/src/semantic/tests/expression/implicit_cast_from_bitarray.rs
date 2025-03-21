// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
#[ignore = "not yet implemented"]
fn to_int_decl_implicitly() {
    let input = r#"
        bit[5] reg;
        int b = reg;
    "#;

    check_classical_decls(input, &expect![[r#""#]]);
}

#[test]
#[ignore = "not yet implemented"]
fn to_int_assignment_implicitly() {
    let input = r#"
        bit[5] reg;
        int a;
        a = reg;
    "#;

    check_classical_decls(input, &expect![[r#""#]]);
}

#[test]
#[ignore = "not yet implemented"]
fn to_int_with_equal_width_in_assignment_implicitly() {
    let input = r#"
        bit[5] reg;
        int[5] a;
        a = reg;
    "#;

    check_classical_decls(input, &expect![[r#""#]]);
}

#[test]
#[ignore = "not yet implemented"]
fn to_int_with_equal_width_in_decl_implicitly() {
    let input = r#"
        bit[5] reg;
        int[5] a = reg;
    "#;

    check_classical_decls(input, &expect![[r#""#]]);
}

#[test]
#[ignore = "not yet implemented"]
fn to_int_with_higher_width_implicitly_fails() {
    let input = "
        int[6] a;
        bit[5] reg;
        a = reg;
    ";

    check_classical_decls(input, &expect![[r#""#]]);
}

#[test]
#[ignore = "not yet implemented"]
fn to_int_with_higher_width_decl_implicitly_fails() {
    let input = "
        bit[5] reg;
        int[6] a = reg;
    ";
    check_classical_decls(input, &expect![[r#""#]]);
}

#[test]
#[ignore = "not yet implemented"]
fn to_int_with_lower_width_implicitly_fails() {
    let input = "
        input int[4] a;
        bit[5] reg;
        a = reg;
    ";

    check_classical_decls(input, &expect![[r#""#]]);
}

#[test]
#[ignore = "not yet implemented"]
fn to_int_with_lower_width_decl_implicitly_fails() {
    let input = "
        bit[5] reg;
        int[4] a = reg;
    ";

    check_classical_decls(input, &expect![[r#""#]]);
}
