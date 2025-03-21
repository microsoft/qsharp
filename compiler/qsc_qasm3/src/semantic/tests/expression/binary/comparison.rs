// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod bit_to_bit;
mod bool_to_bool;
mod float_to_float;
mod int_to_int;

mod uint_to_uint;

use expect_test::expect;

use crate::semantic::tests::check_stmt_kinds;

#[test]
#[ignore = "not yet implemented"]
fn bitarray_var_comparisons_can_be_translated() {
    let input = r#"
        bit[1] x = "1";
        bit[1] y = "0";
        bool f = x > y;
        bool e = x >= y;
        bool a = x < y;
        bool c = x <= y;
        bool b = x == y;
        bool d = x != y;
    "#;

    check_stmt_kinds(input, &expect![[r#""#]]);
}

#[test]
#[ignore = "not yet implemented"]
fn bitarray_var_comparison_to_int_can_be_translated() {
    let input = r#"
        bit[1] x = "1";
        input int y;
        bool a = x > y;
        bool b = x >= y;
        bool c = x < y;
        bool d = x <= y;
        bool e = x == y;
        bool f = x != y;
        bool g = y > x;
        bool h = y >= x;
        bool i = y < x;
        bool j = y <= x;
        bool k = y == x;
        bool l = y != x;
    "#;

    check_stmt_kinds(input, &expect![[r#""#]]);
}
