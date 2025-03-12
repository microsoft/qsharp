// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_stmt_kinds;

#[test]
#[ignore = "not yet implemented"]
fn subtraction() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a - b);
    ";

    check_stmt_kinds(input, &expect![[r#""#]]);
}

#[test]
#[ignore = "not yet implemented"]
fn addition() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a + b);
    ";

    check_stmt_kinds(input, &expect![[r#""#]]);
}

#[test]
#[ignore = "not yet implemented"]
fn multiplication() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a * b);
    ";

    check_stmt_kinds(input, &expect![[r#""#]]);
}

#[test]
#[ignore = "not yet implemented"]
fn division() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a / b);
    ";

    check_stmt_kinds(input, &expect![[r#""#]]);
}

#[test]
#[ignore = "not yet implemented"]
fn power() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a ** b);
    ";

    check_stmt_kinds(input, &expect![[r#""#]]);
}
