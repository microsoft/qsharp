// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::expr;
use crate::parse::tests::check;
use expect_test::expect;

#[test]
fn lit_int() {
    check(
        expr,
        "123",
        &expect!["Expr 4294967295 [0-3]: Lit: Int(123)"],
    );
}

#[test]
fn lit_int_underscore() {
    check(
        expr,
        "123_456",
        &expect!["Expr 4294967295 [0-7]: Lit: Int(123456)"],
    );
}

#[test]
fn lit_int_leading_zero() {
    check(
        expr,
        "0123",
        &expect!["Expr 4294967295 [0-4]: Lit: Int(123)"],
    );
}

#[test]
fn lit_int_overflow() {
    check(
        expr,
        "9_223_372_036_854_775_808",
        &expect!["Expr 4294967295 [0-25]: Lit: Int(-9223372036854775808)"],
    );
}

#[test]
fn lit_int_min() {
    check(
        expr,
        "-9_223_372_036_854_775_808",
        &expect![[r#"
            Expr 4294967295 [0-26]: UnOp (Neg):
                Expr 4294967295 [1-26]: Lit: Int(-9223372036854775808)"#]],
    );
}

#[test]
fn lit_int_hexadecimal() {
    check(
        expr,
        "0x1a2b3c",
        &expect!["Expr 4294967295 [0-8]: Lit: Int(1715004)"],
    );
}

#[test]
fn lit_int_octal() {
    check(
        expr,
        "0o1234567",
        &expect!["Expr 4294967295 [0-9]: Lit: Int(342391)"],
    );
}

#[test]
fn lit_int_binary() {
    check(
        expr,
        "0b10110",
        &expect!["Expr 4294967295 [0-7]: Lit: Int(22)"],
    );
}

#[test]
fn lit_bigint() {
    check(
        expr,
        "123L",
        &expect!["Expr 4294967295 [0-4]: Lit: BigInt(123)"],
    );
}

#[test]
fn lit_bigint_underscore() {
    check(
        expr,
        "123_456L",
        &expect!["Expr 4294967295 [0-8]: Lit: BigInt(123456)"],
    );
}

#[test]
fn lit_bigint_hexadecimal() {
    check(
        expr,
        "0x1a2b3cL",
        &expect!["Expr 4294967295 [0-9]: Lit: BigInt(1715004)"],
    );
}

#[test]
fn lit_bigint_octal() {
    check(
        expr,
        "0o1234567L",
        &expect!["Expr 4294967295 [0-10]: Lit: BigInt(342391)"],
    );
}

#[test]
fn lit_bigint_binary() {
    check(
        expr,
        "0b10110L",
        &expect!["Expr 4294967295 [0-8]: Lit: BigInt(22)"],
    );
}

#[test]
fn lit_double() {
    check(
        expr,
        "1.23",
        &expect!["Expr 4294967295 [0-4]: Lit: Double(1.23)"],
    );
}

#[test]
fn lit_double_leading_dot() {
    check(
        expr,
        ".23",
        &expect![[r#"
            Err(
                Rule(
                    "expression",
                    Dot,
                    Span {
                        lo: 0,
                        hi: 1,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn lit_double_trailing_dot() {
    check(
        expr,
        "1.",
        &expect!["Expr 4294967295 [0-2]: Lit: Double(1)"],
    );
}

#[test]
fn lit_double_underscore() {
    check(
        expr,
        "123_456.78",
        &expect!["Expr 4294967295 [0-10]: Lit: Double(123456.78)"],
    );
}

#[test]
fn lit_double_leading_zero() {
    check(
        expr,
        "0.23",
        &expect!["Expr 4294967295 [0-4]: Lit: Double(0.23)"],
    );
}

#[test]
fn lit_int_hexadecimal_dot() {
    check(
        expr,
        "0x123.45",
        &expect!["Expr 4294967295 [0-5]: Lit: Int(291)"],
    );
}

#[test]
fn lit_string() {
    check(
        expr,
        r#""foo""#,
        &expect![[r#"Expr 4294967295 [0-5]: Lit: String("foo")"#]],
    );
}

#[test]
fn lit_string_escape_quote() {
    check(
        expr,
        r#""foo\"bar""#,
        &expect![[r#"Expr 4294967295 [0-10]: Lit: String("foo\"bar")"#]],
    );
}

#[test]
fn lit_string_unmatched_quote() {
    check(
        expr,
        r#""Uh oh.."#,
        &expect![[r#"
        Err(
            Rule(
                "expression",
                Eof,
                Span {
                    lo: 8,
                    hi: 8,
                },
            ),
        )
    "#]],
    );
}

#[test]
fn lit_string_empty() {
    check(
        expr,
        r#""""#,
        &expect![[r#"Expr 4294967295 [0-2]: Lit: String("")"#]],
    );
}

#[test]
fn lit_false() {
    check(
        expr,
        "false",
        &expect!["Expr 4294967295 [0-5]: Lit: Bool(false)"],
    );
}

#[test]
fn lit_true() {
    check(
        expr,
        "true",
        &expect!["Expr 4294967295 [0-4]: Lit: Bool(true)"],
    );
}

#[test]
fn lit_zero() {
    check(
        expr,
        "Zero",
        &expect!["Expr 4294967295 [0-4]: Lit: Result(Zero)"],
    );
}

#[test]
fn lit_one() {
    check(
        expr,
        "One",
        &expect!["Expr 4294967295 [0-3]: Lit: Result(One)"],
    );
}

#[test]
fn lit_pauli_i() {
    check(
        expr,
        "PauliI",
        &expect!["Expr 4294967295 [0-6]: Lit: Pauli(I)"],
    );
}

#[test]
fn lit_pauli_x() {
    check(
        expr,
        "PauliX",
        &expect!["Expr 4294967295 [0-6]: Lit: Pauli(X)"],
    );
}

#[test]
fn lit_pauli_y() {
    check(
        expr,
        "PauliY",
        &expect!["Expr 4294967295 [0-6]: Lit: Pauli(Y)"],
    );
}

#[test]
fn lit_pauli_z() {
    check(
        expr,
        "PauliZ",
        &expect!["Expr 4294967295 [0-6]: Lit: Pauli(Z)"],
    );
}

#[test]
fn hole() {
    check(expr, "_", &expect!["Expr 4294967295 [0-1]: Hole"]);
}

#[test]
fn single_path() {
    check(
        expr,
        "foo",
        &expect![[
            r#"Expr 4294967295 [0-3]: Path: Path 4294967295 [0-3] (Ident 4294967295 [0-3] "foo")"#
        ]],
    );
}

#[test]
fn double_path() {
    check(
        expr,
        "foo.bar",
        &expect![[
            r#"Expr 4294967295 [0-7]: Path: Path 4294967295 [0-7] (Ident 4294967295 [0-3] "foo") (Ident 4294967295 [4-7] "bar")"#
        ]],
    );
}

#[test]
fn block() {
    check(
        expr,
        "{ let x = 1; x }",
        &expect![[r#"
            Expr 4294967295 [0-16]: Expr Block: Block 4294967295 [0-16]:
                Stmt 4294967295 [2-12]: Local (Immutable):
                    Pat 4294967295 [6-7]: Bind:
                        Ident 4294967295 [6-7] "x"
                    Expr 4294967295 [10-11]: Lit: Int(1)
                Stmt 4294967295 [13-14]: Expr: Expr 4294967295 [13-14]: Path: Path 4294967295 [13-14] (Ident 4294967295 [13-14] "x")"#]],
    );
}

#[test]
fn fail() {
    check(
        expr,
        r#"fail "message""#,
        &expect![[
            r#"Expr 4294967295 [0-14]: Fail: Expr 4294967295 [5-14]: Lit: String("message")"#
        ]],
    );
}

#[test]
fn for_in() {
    check(
        expr,
        "for x in xs { x }",
        &expect![[r#"
            Expr 4294967295 [0-17]: For:
                Pat 4294967295 [4-5]: Bind:
                    Ident 4294967295 [4-5] "x"
                Expr 4294967295 [9-11]: Path: Path 4294967295 [9-11] (Ident 4294967295 [9-11] "xs")
                Block 4294967295 [12-17]:
                    Stmt 4294967295 [14-15]: Expr: Expr 4294967295 [14-15]: Path: Path 4294967295 [14-15] (Ident 4294967295 [14-15] "x")"#]],
    );
}

#[test]
fn if_then() {
    check(
        expr,
        "if c { e }",
        &expect![[r#"
            Expr 4294967295 [0-10]: If:
                Expr 4294967295 [3-4]: Path: Path 4294967295 [3-4] (Ident 4294967295 [3-4] "c")
                Block 4294967295 [5-10]:
                    Stmt 4294967295 [7-8]: Expr: Expr 4294967295 [7-8]: Path: Path 4294967295 [7-8] (Ident 4294967295 [7-8] "e")"#]],
    );
}

#[test]
fn if_else() {
    check(
        expr,
        "if c { x } else { y }",
        &expect![[r#"
            Expr 4294967295 [0-21]: If:
                Expr 4294967295 [3-4]: Path: Path 4294967295 [3-4] (Ident 4294967295 [3-4] "c")
                Block 4294967295 [5-10]:
                    Stmt 4294967295 [7-8]: Expr: Expr 4294967295 [7-8]: Path: Path 4294967295 [7-8] (Ident 4294967295 [7-8] "x")
                Expr 4294967295 [11-21]: Expr Block: Block 4294967295 [16-21]:
                    Stmt 4294967295 [18-19]: Expr: Expr 4294967295 [18-19]: Path: Path 4294967295 [18-19] (Ident 4294967295 [18-19] "y")"#]],
    );
}

#[test]
fn if_elif() {
    check(
        expr,
        "if c1 { x } elif c2 { y }",
        &expect![[r#"
            Expr 4294967295 [0-25]: If:
                Expr 4294967295 [3-5]: Path: Path 4294967295 [3-5] (Ident 4294967295 [3-5] "c1")
                Block 4294967295 [6-11]:
                    Stmt 4294967295 [8-9]: Expr: Expr 4294967295 [8-9]: Path: Path 4294967295 [8-9] (Ident 4294967295 [8-9] "x")
                Expr 4294967295 [12-25]: If:
                    Expr 4294967295 [17-19]: Path: Path 4294967295 [17-19] (Ident 4294967295 [17-19] "c2")
                    Block 4294967295 [20-25]:
                        Stmt 4294967295 [22-23]: Expr: Expr 4294967295 [22-23]: Path: Path 4294967295 [22-23] (Ident 4294967295 [22-23] "y")"#]],
    );
}

#[test]
fn if_elif_else() {
    check(
        expr,
        "if c1 { x } elif c2 { y } else { z }",
        &expect![[r#"
            Expr 4294967295 [0-36]: If:
                Expr 4294967295 [3-5]: Path: Path 4294967295 [3-5] (Ident 4294967295 [3-5] "c1")
                Block 4294967295 [6-11]:
                    Stmt 4294967295 [8-9]: Expr: Expr 4294967295 [8-9]: Path: Path 4294967295 [8-9] (Ident 4294967295 [8-9] "x")
                Expr 4294967295 [12-36]: If:
                    Expr 4294967295 [17-19]: Path: Path 4294967295 [17-19] (Ident 4294967295 [17-19] "c2")
                    Block 4294967295 [20-25]:
                        Stmt 4294967295 [22-23]: Expr: Expr 4294967295 [22-23]: Path: Path 4294967295 [22-23] (Ident 4294967295 [22-23] "y")
                    Expr 4294967295 [26-36]: Expr Block: Block 4294967295 [31-36]:
                        Stmt 4294967295 [33-34]: Expr: Expr 4294967295 [33-34]: Path: Path 4294967295 [33-34] (Ident 4294967295 [33-34] "z")"#]],
    );
}

#[test]
fn repeat_until() {
    check(
        expr,
        "repeat { x } until c",
        &expect![[r#"
            Expr 4294967295 [0-20]: Repeat:
                Block 4294967295 [7-12]:
                    Stmt 4294967295 [9-10]: Expr: Expr 4294967295 [9-10]: Path: Path 4294967295 [9-10] (Ident 4294967295 [9-10] "x")
                Expr 4294967295 [19-20]: Path: Path 4294967295 [19-20] (Ident 4294967295 [19-20] "c")
                <no fixup>"#]],
    );
}

#[test]
fn repeat_until_fixup() {
    check(
        expr,
        "repeat { x } until c fixup { y }",
        &expect![[r#"
            Expr 4294967295 [0-32]: Repeat:
                Block 4294967295 [7-12]:
                    Stmt 4294967295 [9-10]: Expr: Expr 4294967295 [9-10]: Path: Path 4294967295 [9-10] (Ident 4294967295 [9-10] "x")
                Expr 4294967295 [19-20]: Path: Path 4294967295 [19-20] (Ident 4294967295 [19-20] "c")
                Block 4294967295 [27-32]:
                    Stmt 4294967295 [29-30]: Expr: Expr 4294967295 [29-30]: Path: Path 4294967295 [29-30] (Ident 4294967295 [29-30] "y")"#]],
    );
}

#[test]
fn return_expr() {
    check(
        expr,
        "return x",
        &expect![[
            r#"Expr 4294967295 [0-8]: Return: Expr 4294967295 [7-8]: Path: Path 4294967295 [7-8] (Ident 4294967295 [7-8] "x")"#
        ]],
    );
}

#[test]
fn set() {
    check(
        expr,
        "set x = y",
        &expect![[r#"
            Expr 4294967295 [0-9]: Assign:
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [8-9]: Path: Path 4294967295 [8-9] (Ident 4294967295 [8-9] "y")"#]],
    );
}

#[test]
fn set_hole() {
    check(
        expr,
        "set _ = 1",
        &expect![[r#"
            Expr 4294967295 [0-9]: Assign:
                Expr 4294967295 [4-5]: Hole
                Expr 4294967295 [8-9]: Lit: Int(1)"#]],
    );
}

#[test]
fn set_hole_tuple() {
    check(
        expr,
        "set (x, _) = (1, 2)",
        &expect![[r#"
            Expr 4294967295 [0-19]: Assign:
                Expr 4294967295 [4-10]: Tuple:
                    Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "x")
                    Expr 4294967295 [8-9]: Hole
                Expr 4294967295 [13-19]: Tuple:
                    Expr 4294967295 [14-15]: Lit: Int(1)
                    Expr 4294967295 [17-18]: Lit: Int(2)"#]],
    );
}

#[test]
fn set_hole_tuple_nested() {
    check(
        expr,
        "set (_, (x, _)) = (1, (2, 3))",
        &expect![[r#"
            Expr 4294967295 [0-29]: Assign:
                Expr 4294967295 [4-15]: Tuple:
                    Expr 4294967295 [5-6]: Hole
                    Expr 4294967295 [8-14]: Tuple:
                        Expr 4294967295 [9-10]: Path: Path 4294967295 [9-10] (Ident 4294967295 [9-10] "x")
                        Expr 4294967295 [12-13]: Hole
                Expr 4294967295 [18-29]: Tuple:
                    Expr 4294967295 [19-20]: Lit: Int(1)
                    Expr 4294967295 [22-28]: Tuple:
                        Expr 4294967295 [23-24]: Lit: Int(2)
                        Expr 4294967295 [26-27]: Lit: Int(3)"#]],
    );
}

#[test]
fn set_bitwise_and() {
    check(
        expr,
        "set x &&&= y",
        &expect![[r#"
            Expr 4294967295 [0-12]: AssignOp (AndB):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [11-12]: Path: Path 4294967295 [11-12] (Ident 4294967295 [11-12] "y")"#]],
    );
}

#[test]
fn set_logical_and() {
    check(
        expr,
        "set x and= y",
        &expect![[r#"
            Expr 4294967295 [0-12]: AssignOp (AndL):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [11-12]: Path: Path 4294967295 [11-12] (Ident 4294967295 [11-12] "y")"#]],
    );
}

#[test]
fn set_bitwise_or() {
    check(
        expr,
        "set x |||= y",
        &expect![[r#"
            Expr 4294967295 [0-12]: AssignOp (OrB):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [11-12]: Path: Path 4294967295 [11-12] (Ident 4294967295 [11-12] "y")"#]],
    );
}

#[test]
fn set_exp() {
    check(
        expr,
        "set x ^= y",
        &expect![[r#"
            Expr 4294967295 [0-10]: AssignOp (Exp):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [9-10]: Path: Path 4294967295 [9-10] (Ident 4294967295 [9-10] "y")"#]],
    );
}

#[test]
fn set_bitwise_xor() {
    check(
        expr,
        "set x ^^^= y",
        &expect![[r#"
            Expr 4294967295 [0-12]: AssignOp (XorB):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [11-12]: Path: Path 4294967295 [11-12] (Ident 4294967295 [11-12] "y")"#]],
    );
}

#[test]
fn set_shr() {
    check(
        expr,
        "set x >>>= y",
        &expect![[r#"
            Expr 4294967295 [0-12]: AssignOp (Shr):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [11-12]: Path: Path 4294967295 [11-12] (Ident 4294967295 [11-12] "y")"#]],
    );
}

#[test]
fn set_shl() {
    check(
        expr,
        "set x <<<= y",
        &expect![[r#"
            Expr 4294967295 [0-12]: AssignOp (Shl):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [11-12]: Path: Path 4294967295 [11-12] (Ident 4294967295 [11-12] "y")"#]],
    );
}

#[test]
fn set_sub() {
    check(
        expr,
        "set x -= y",
        &expect![[r#"
            Expr 4294967295 [0-10]: AssignOp (Sub):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [9-10]: Path: Path 4294967295 [9-10] (Ident 4294967295 [9-10] "y")"#]],
    );
}

#[test]
fn set_logical_or() {
    check(
        expr,
        "set x or= y",
        &expect![[r#"
            Expr 4294967295 [0-11]: AssignOp (OrL):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [10-11]: Path: Path 4294967295 [10-11] (Ident 4294967295 [10-11] "y")"#]],
    );
}

#[test]
fn set_mod() {
    check(
        expr,
        "set x %= y",
        &expect![[r#"
            Expr 4294967295 [0-10]: AssignOp (Mod):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [9-10]: Path: Path 4294967295 [9-10] (Ident 4294967295 [9-10] "y")"#]],
    );
}

#[test]
fn set_add() {
    check(
        expr,
        "set x += y",
        &expect![[r#"
            Expr 4294967295 [0-10]: AssignOp (Add):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [9-10]: Path: Path 4294967295 [9-10] (Ident 4294967295 [9-10] "y")"#]],
    );
}

#[test]
fn set_div() {
    check(
        expr,
        "set x /= y",
        &expect![[r#"
            Expr 4294967295 [0-10]: AssignOp (Div):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [9-10]: Path: Path 4294967295 [9-10] (Ident 4294967295 [9-10] "y")"#]],
    );
}

#[test]
fn set_mul() {
    check(
        expr,
        "set x *= y",
        &expect![[r#"
            Expr 4294967295 [0-10]: AssignOp (Mul):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [9-10]: Path: Path 4294967295 [9-10] (Ident 4294967295 [9-10] "y")"#]],
    );
}

#[test]
fn set_with_update() {
    check(
        expr,
        "set x w/= i <- y",
        &expect![[r#"
            Expr 4294967295 [0-16]: AssignUpdate:
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                Expr 4294967295 [10-11]: Path: Path 4294967295 [10-11] (Ident 4294967295 [10-11] "i")
                Expr 4294967295 [15-16]: Path: Path 4294967295 [15-16] (Ident 4294967295 [15-16] "y")"#]],
    );
}

#[test]
fn while_expr() {
    check(
        expr,
        "while c { x }",
        &expect![[r#"
            Expr 4294967295 [0-13]: While:
                Expr 4294967295 [6-7]: Path: Path 4294967295 [6-7] (Ident 4294967295 [6-7] "c")
                Block 4294967295 [8-13]:
                    Stmt 4294967295 [10-11]: Expr: Expr 4294967295 [10-11]: Path: Path 4294967295 [10-11] (Ident 4294967295 [10-11] "x")"#]],
    );
}

#[test]
fn within_apply() {
    check(
        expr,
        "within { x } apply { y }",
        &expect![[r#"
            Expr 4294967295 [0-24]: Conjugate:
                Block 4294967295 [7-12]:
                    Stmt 4294967295 [9-10]: Expr: Expr 4294967295 [9-10]: Path: Path 4294967295 [9-10] (Ident 4294967295 [9-10] "x")
                Block 4294967295 [19-24]:
                    Stmt 4294967295 [21-22]: Expr: Expr 4294967295 [21-22]: Path: Path 4294967295 [21-22] (Ident 4294967295 [21-22] "y")"#]],
    );
}

#[test]
fn unit() {
    check(expr, "()", &expect!["Expr 4294967295 [0-2]: Unit"]);
}

#[test]
fn paren() {
    check(
        expr,
        "(x)",
        &expect![[
            r#"Expr 4294967295 [0-3]: Paren: Expr 4294967295 [1-2]: Path: Path 4294967295 [1-2] (Ident 4294967295 [1-2] "x")"#
        ]],
    );
}

#[test]
fn singleton_tuple() {
    check(
        expr,
        "(x,)",
        &expect![[r#"
            Expr 4294967295 [0-4]: Tuple:
                Expr 4294967295 [1-2]: Path: Path 4294967295 [1-2] (Ident 4294967295 [1-2] "x")"#]],
    );
}

#[test]
fn pair() {
    check(
        expr,
        "(x, y)",
        &expect![[r#"
            Expr 4294967295 [0-6]: Tuple:
                Expr 4294967295 [1-2]: Path: Path 4294967295 [1-2] (Ident 4294967295 [1-2] "x")
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "y")"#]],
    );
}

#[test]
fn array_empty() {
    check(expr, "[]", &expect!["Expr 4294967295 [0-2]: Array:"]);
}

#[test]
fn array_single() {
    check(
        expr,
        "[x]",
        &expect![[r#"
            Expr 4294967295 [0-3]: Array:
                Expr 4294967295 [1-2]: Path: Path 4294967295 [1-2] (Ident 4294967295 [1-2] "x")"#]],
    );
}

#[test]
fn array_pair() {
    check(
        expr,
        "[x, y]",
        &expect![[r#"
            Expr 4294967295 [0-6]: Array:
                Expr 4294967295 [1-2]: Path: Path 4294967295 [1-2] (Ident 4294967295 [1-2] "x")
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "y")"#]],
    );
}

#[test]
fn array_repeat() {
    check(
        expr,
        "[0, size = 3]",
        &expect![[r#"
            Expr 4294967295 [0-13]: ArrayRepeat:
                Expr 4294967295 [1-2]: Lit: Int(0)
                Expr 4294967295 [11-12]: Lit: Int(3)"#]],
    );
}

#[test]
fn array_repeat_complex() {
    check(
        expr,
        "[Foo(), size = Count() + 1]",
        &expect![[r#"
            Expr 4294967295 [0-27]: ArrayRepeat:
                Expr 4294967295 [1-6]: Call:
                    Expr 4294967295 [1-4]: Path: Path 4294967295 [1-4] (Ident 4294967295 [1-4] "Foo")
                    Expr 4294967295 [4-6]: Unit
                Expr 4294967295 [15-26]: BinOp (Add):
                    Expr 4294967295 [15-22]: Call:
                        Expr 4294967295 [15-20]: Path: Path 4294967295 [15-20] (Ident 4294967295 [15-20] "Count")
                        Expr 4294967295 [20-22]: Unit
                    Expr 4294967295 [25-26]: Lit: Int(1)"#]],
    );
}

#[test]
fn array_size_last_item() {
    check(
        expr,
        "[foo, size]",
        &expect![[r#"
            Expr 4294967295 [0-11]: Array:
                Expr 4294967295 [1-4]: Path: Path 4294967295 [1-4] (Ident 4294967295 [1-4] "foo")
                Expr 4294967295 [6-10]: Path: Path 4294967295 [6-10] (Ident 4294967295 [6-10] "size")"#]],
    );
}

#[test]
fn array_size_middle_item() {
    check(
        expr,
        "[foo, size, bar]",
        &expect![[r#"
            Expr 4294967295 [0-16]: Array:
                Expr 4294967295 [1-4]: Path: Path 4294967295 [1-4] (Ident 4294967295 [1-4] "foo")
                Expr 4294967295 [6-10]: Path: Path 4294967295 [6-10] (Ident 4294967295 [6-10] "size")
                Expr 4294967295 [12-15]: Path: Path 4294967295 [12-15] (Ident 4294967295 [12-15] "bar")"#]],
    );
}

#[test]
fn array_repeat_no_items() {
    check(
        expr,
        "[size = 3]",
        &expect![[r#"
            Err(
                Token(
                    Close(
                        Bracket,
                    ),
                    Eq,
                    Span {
                        lo: 6,
                        hi: 7,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn array_repeat_two_items() {
    check(
        expr,
        "[1, 2, size = 3]",
        &expect![[r#"
            Err(
                Token(
                    Close(
                        Bracket,
                    ),
                    Eq,
                    Span {
                        lo: 12,
                        hi: 13,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn array_concat() {
    check(
        expr,
        "[1, 2] + [3, 4]",
        &expect![[r#"
            Expr 4294967295 [0-15]: BinOp (Add):
                Expr 4294967295 [0-6]: Array:
                    Expr 4294967295 [1-2]: Lit: Int(1)
                    Expr 4294967295 [4-5]: Lit: Int(2)
                Expr 4294967295 [9-15]: Array:
                    Expr 4294967295 [10-11]: Lit: Int(3)
                    Expr 4294967295 [13-14]: Lit: Int(4)"#]],
    );
}

#[test]
fn and_op() {
    check(
        expr,
        "x and y",
        &expect![[r#"
            Expr 4294967295 [0-7]: BinOp (AndL):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [6-7]: Path: Path 4294967295 [6-7] (Ident 4294967295 [6-7] "y")"#]],
    );
}

#[test]
fn or_op() {
    check(
        expr,
        "x or y",
        &expect![[r#"
            Expr 4294967295 [0-6]: BinOp (OrL):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "y")"#]],
    );
}

#[test]
fn and_or_ops() {
    check(
        expr,
        "x or y and z",
        &expect![[r#"
            Expr 4294967295 [0-12]: BinOp (OrL):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [5-12]: BinOp (AndL):
                    Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "y")
                    Expr 4294967295 [11-12]: Path: Path 4294967295 [11-12] (Ident 4294967295 [11-12] "z")"#]],
    );
}

#[test]
fn eq_op() {
    check(
        expr,
        "x == y",
        &expect![[r#"
            Expr 4294967295 [0-6]: BinOp (Eq):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "y")"#]],
    );
}

#[test]
fn ne_op() {
    check(
        expr,
        "x != y",
        &expect![[r#"
            Expr 4294967295 [0-6]: BinOp (Neq):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "y")"#]],
    );
}

#[test]
fn gt_op() {
    check(
        expr,
        "x > y",
        &expect![[r#"
            Expr 4294967295 [0-5]: BinOp (Gt):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "y")"#]],
    );
}

#[test]
fn gte_op() {
    check(
        expr,
        "x >= y",
        &expect![[r#"
            Expr 4294967295 [0-6]: BinOp (Gte):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "y")"#]],
    );
}

#[test]
fn lt_op() {
    check(
        expr,
        "x < y",
        &expect![[r#"
            Expr 4294967295 [0-5]: BinOp (Lt):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "y")"#]],
    );
}

#[test]
fn lte_op() {
    check(
        expr,
        "x <= y",
        &expect![[r#"
            Expr 4294967295 [0-6]: BinOp (Lte):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "y")"#]],
    );
}

#[test]
fn bitwise_and_op() {
    check(
        expr,
        "x &&& y",
        &expect![[r#"
            Expr 4294967295 [0-7]: BinOp (AndB):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [6-7]: Path: Path 4294967295 [6-7] (Ident 4294967295 [6-7] "y")"#]],
    );
}

#[test]
fn bitwise_or_op() {
    check(
        expr,
        "x ||| y",
        &expect![[r#"
            Expr 4294967295 [0-7]: BinOp (OrB):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [6-7]: Path: Path 4294967295 [6-7] (Ident 4294967295 [6-7] "y")"#]],
    );
}

#[test]
fn bitwise_and_or_op() {
    check(
        expr,
        "x ||| y &&& z",
        &expect![[r#"
            Expr 4294967295 [0-13]: BinOp (OrB):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [6-13]: BinOp (AndB):
                    Expr 4294967295 [6-7]: Path: Path 4294967295 [6-7] (Ident 4294967295 [6-7] "y")
                    Expr 4294967295 [12-13]: Path: Path 4294967295 [12-13] (Ident 4294967295 [12-13] "z")"#]],
    );
}

#[test]
fn bitwise_xor_op() {
    check(
        expr,
        "x ^^^ y",
        &expect![[r#"
            Expr 4294967295 [0-7]: BinOp (XorB):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [6-7]: Path: Path 4294967295 [6-7] (Ident 4294967295 [6-7] "y")"#]],
    );
}

#[test]
fn bitwise_or_xor_ops() {
    check(
        expr,
        "x ||| y ^^^ z",
        &expect![[r#"
            Expr 4294967295 [0-13]: BinOp (OrB):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [6-13]: BinOp (XorB):
                    Expr 4294967295 [6-7]: Path: Path 4294967295 [6-7] (Ident 4294967295 [6-7] "y")
                    Expr 4294967295 [12-13]: Path: Path 4294967295 [12-13] (Ident 4294967295 [12-13] "z")"#]],
    );
}

#[test]
fn shl_op() {
    check(
        expr,
        "x <<< y",
        &expect![[r#"
            Expr 4294967295 [0-7]: BinOp (Shl):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [6-7]: Path: Path 4294967295 [6-7] (Ident 4294967295 [6-7] "y")"#]],
    );
}

#[test]
fn shr_op() {
    check(
        expr,
        "x >>> y",
        &expect![[r#"
            Expr 4294967295 [0-7]: BinOp (Shr):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [6-7]: Path: Path 4294967295 [6-7] (Ident 4294967295 [6-7] "y")"#]],
    );
}

#[test]
fn add_op() {
    check(
        expr,
        "x + y",
        &expect![[r#"
            Expr 4294967295 [0-5]: BinOp (Add):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "y")"#]],
    );
}

#[test]
fn add_left_assoc() {
    check(
        expr,
        "x + y + z",
        &expect![[r#"
            Expr 4294967295 [0-9]: BinOp (Add):
                Expr 4294967295 [0-5]: BinOp (Add):
                    Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                    Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "y")
                Expr 4294967295 [8-9]: Path: Path 4294967295 [8-9] (Ident 4294967295 [8-9] "z")"#]],
    );
}

#[test]
fn sub_op() {
    check(
        expr,
        "x - y",
        &expect![[r#"
            Expr 4294967295 [0-5]: BinOp (Sub):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "y")"#]],
    );
}

#[test]
fn mul_op() {
    check(
        expr,
        "x * y",
        &expect![[r#"
            Expr 4294967295 [0-5]: BinOp (Mul):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "y")"#]],
    );
}

#[test]
fn add_mul_ops() {
    check(
        expr,
        "x + y * z",
        &expect![[r#"
            Expr 4294967295 [0-9]: BinOp (Add):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [4-9]: BinOp (Mul):
                    Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "y")
                    Expr 4294967295 [8-9]: Path: Path 4294967295 [8-9] (Ident 4294967295 [8-9] "z")"#]],
    );
}

#[test]
fn div_op() {
    check(
        expr,
        "x / y",
        &expect![[r#"
            Expr 4294967295 [0-5]: BinOp (Div):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "y")"#]],
    );
}

#[test]
fn mod_op() {
    check(
        expr,
        "x % y",
        &expect![[r#"
            Expr 4294967295 [0-5]: BinOp (Mod):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "y")"#]],
    );
}

#[test]
fn two_plus_two_is_four() {
    check(
        expr,
        "2 + 2 == 4",
        &expect![[r#"
            Expr 4294967295 [0-10]: BinOp (Eq):
                Expr 4294967295 [0-5]: BinOp (Add):
                    Expr 4294967295 [0-1]: Lit: Int(2)
                    Expr 4294967295 [4-5]: Lit: Int(2)
                Expr 4294967295 [9-10]: Lit: Int(4)"#]],
    );
}

#[test]
fn exp_op() {
    check(
        expr,
        "x ^ y",
        &expect![[r#"
            Expr 4294967295 [0-5]: BinOp (Exp):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "y")"#]],
    );
}

#[test]
fn exp_right_assoc() {
    check(
        expr,
        "2 ^ 3 ^ 4",
        &expect![[r#"
            Expr 4294967295 [0-9]: BinOp (Exp):
                Expr 4294967295 [0-1]: Lit: Int(2)
                Expr 4294967295 [4-9]: BinOp (Exp):
                    Expr 4294967295 [4-5]: Lit: Int(3)
                    Expr 4294967295 [8-9]: Lit: Int(4)"#]],
    );
}

#[test]
fn negate_exp() {
    check(
        expr,
        "-2^3",
        &expect![[r#"
            Expr 4294967295 [0-4]: UnOp (Neg):
                Expr 4294967295 [1-4]: BinOp (Exp):
                    Expr 4294967295 [1-2]: Lit: Int(2)
                    Expr 4294967295 [3-4]: Lit: Int(3)"#]],
    );
}

#[test]
fn unwrap_op() {
    check(
        expr,
        "x!",
        &expect![[r#"
            Expr 4294967295 [0-2]: UnOp (Unwrap):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")"#]],
    );
}

#[test]
fn logical_not_op() {
    check(
        expr,
        "not x",
        &expect![[r#"
            Expr 4294967295 [0-5]: UnOp (NotL):
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")"#]],
    );
}

#[test]
fn bitwise_not_op() {
    check(
        expr,
        "~~~x",
        &expect![[r#"
            Expr 4294967295 [0-4]: UnOp (NotB):
                Expr 4294967295 [3-4]: Path: Path 4294967295 [3-4] (Ident 4294967295 [3-4] "x")"#]],
    );
}

#[test]
fn pos_op() {
    check(
        expr,
        "+x",
        &expect![[r#"
            Expr 4294967295 [0-2]: UnOp (Pos):
                Expr 4294967295 [1-2]: Path: Path 4294967295 [1-2] (Ident 4294967295 [1-2] "x")"#]],
    );
}

#[test]
fn neg_op() {
    check(
        expr,
        "-x",
        &expect![[r#"
            Expr 4294967295 [0-2]: UnOp (Neg):
                Expr 4294967295 [1-2]: Path: Path 4294967295 [1-2] (Ident 4294967295 [1-2] "x")"#]],
    );
}

#[test]
fn neg_minus_ops() {
    check(
        expr,
        "-x - y",
        &expect![[r#"
            Expr 4294967295 [0-6]: BinOp (Sub):
                Expr 4294967295 [0-2]: UnOp (Neg):
                    Expr 4294967295 [1-2]: Path: Path 4294967295 [1-2] (Ident 4294967295 [1-2] "x")
                Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "y")"#]],
    );
}

#[test]
fn adjoint_op() {
    check(
        expr,
        "Adjoint x",
        &expect![[r#"
            Expr 4294967295 [0-9]: UnOp (Functor Adj):
                Expr 4294967295 [8-9]: Path: Path 4294967295 [8-9] (Ident 4294967295 [8-9] "x")"#]],
    );
}

#[test]
fn adjoint_call_ops() {
    check(
        expr,
        "Adjoint X(q)",
        &expect![[r#"
            Expr 4294967295 [0-12]: Call:
                Expr 4294967295 [0-9]: UnOp (Functor Adj):
                    Expr 4294967295 [8-9]: Path: Path 4294967295 [8-9] (Ident 4294967295 [8-9] "X")
                Expr 4294967295 [9-12]: Paren: Expr 4294967295 [10-11]: Path: Path 4294967295 [10-11] (Ident 4294967295 [10-11] "q")"#]],
    );
}

#[test]
fn adjoint_index_call_ops() {
    check(
        expr,
        "Adjoint ops[i](q)",
        &expect![[r#"
            Expr 4294967295 [0-17]: Call:
                Expr 4294967295 [0-14]: UnOp (Functor Adj):
                    Expr 4294967295 [8-14]: Index:
                        Expr 4294967295 [8-11]: Path: Path 4294967295 [8-11] (Ident 4294967295 [8-11] "ops")
                        Expr 4294967295 [12-13]: Path: Path 4294967295 [12-13] (Ident 4294967295 [12-13] "i")
                Expr 4294967295 [14-17]: Paren: Expr 4294967295 [15-16]: Path: Path 4294967295 [15-16] (Ident 4294967295 [15-16] "q")"#]],
    );
}

#[test]
fn controlled_op() {
    check(
        expr,
        "Controlled x",
        &expect![[r#"
            Expr 4294967295 [0-12]: UnOp (Functor Ctl):
                Expr 4294967295 [11-12]: Path: Path 4294967295 [11-12] (Ident 4294967295 [11-12] "x")"#]],
    );
}

#[test]
fn controlled_call_ops() {
    check(
        expr,
        "Controlled X([q1], q2)",
        &expect![[r#"
            Expr 4294967295 [0-22]: Call:
                Expr 4294967295 [0-12]: UnOp (Functor Ctl):
                    Expr 4294967295 [11-12]: Path: Path 4294967295 [11-12] (Ident 4294967295 [11-12] "X")
                Expr 4294967295 [12-22]: Tuple:
                    Expr 4294967295 [13-17]: Array:
                        Expr 4294967295 [14-16]: Path: Path 4294967295 [14-16] (Ident 4294967295 [14-16] "q1")
                    Expr 4294967295 [19-21]: Path: Path 4294967295 [19-21] (Ident 4294967295 [19-21] "q2")"#]],
    );
}

#[test]
fn controlled_index_call_ops() {
    check(
        expr,
        "Controlled ops[i]([q1], q2)",
        &expect![[r#"
            Expr 4294967295 [0-27]: Call:
                Expr 4294967295 [0-17]: UnOp (Functor Ctl):
                    Expr 4294967295 [11-17]: Index:
                        Expr 4294967295 [11-14]: Path: Path 4294967295 [11-14] (Ident 4294967295 [11-14] "ops")
                        Expr 4294967295 [15-16]: Path: Path 4294967295 [15-16] (Ident 4294967295 [15-16] "i")
                Expr 4294967295 [17-27]: Tuple:
                    Expr 4294967295 [18-22]: Array:
                        Expr 4294967295 [19-21]: Path: Path 4294967295 [19-21] (Ident 4294967295 [19-21] "q1")
                    Expr 4294967295 [24-26]: Path: Path 4294967295 [24-26] (Ident 4294967295 [24-26] "q2")"#]],
    );
}

#[test]
fn update_op() {
    check(
        expr,
        "x w/ i <- v",
        &expect![[r#"
            Expr 4294967295 [0-11]: TernOp (Update):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "i")
                Expr 4294967295 [10-11]: Path: Path 4294967295 [10-11] (Ident 4294967295 [10-11] "v")"#]],
    );
}

#[test]
fn update_op_left_assoc() {
    check(
        expr,
        "x w/ i1 <- v1 w/ i2 <- v2",
        &expect![[r#"
            Expr 4294967295 [0-25]: TernOp (Update):
                Expr 4294967295 [0-13]: TernOp (Update):
                    Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                    Expr 4294967295 [5-7]: Path: Path 4294967295 [5-7] (Ident 4294967295 [5-7] "i1")
                    Expr 4294967295 [11-13]: Path: Path 4294967295 [11-13] (Ident 4294967295 [11-13] "v1")
                Expr 4294967295 [17-19]: Path: Path 4294967295 [17-19] (Ident 4294967295 [17-19] "i2")
                Expr 4294967295 [23-25]: Path: Path 4294967295 [23-25] (Ident 4294967295 [23-25] "v2")"#]],
    );
}

#[test]
fn cond_op() {
    check(
        expr,
        "c ? a | b",
        &expect![[r#"
            Expr 4294967295 [0-9]: TernOp (Cond):
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "c")
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "a")
                Expr 4294967295 [8-9]: Path: Path 4294967295 [8-9] (Ident 4294967295 [8-9] "b")"#]],
    );
}

#[test]
fn cond_op_right_assoc() {
    check(
        expr,
        "c1 ? a | c2 ? b | c",
        &expect![[r#"
            Expr 4294967295 [0-19]: TernOp (Cond):
                Expr 4294967295 [0-2]: Path: Path 4294967295 [0-2] (Ident 4294967295 [0-2] "c1")
                Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "a")
                Expr 4294967295 [9-19]: TernOp (Cond):
                    Expr 4294967295 [9-11]: Path: Path 4294967295 [9-11] (Ident 4294967295 [9-11] "c2")
                    Expr 4294967295 [14-15]: Path: Path 4294967295 [14-15] (Ident 4294967295 [14-15] "b")
                    Expr 4294967295 [18-19]: Path: Path 4294967295 [18-19] (Ident 4294967295 [18-19] "c")"#]],
    );
}

#[test]
fn field_op() {
    check(
        expr,
        "x::foo",
        &expect![[r#"
            Expr 4294967295 [0-6]: Field:
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Ident 4294967295 [3-6] "foo""#]],
    );
}

#[test]
fn index_op() {
    check(
        expr,
        "x[i]",
        &expect![[r#"
            Expr 4294967295 [0-4]: Index:
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [2-3]: Path: Path 4294967295 [2-3] (Ident 4294967295 [2-3] "i")"#]],
    );
}

#[test]
fn call_op_unit() {
    check(
        expr,
        "Foo()",
        &expect![[r#"
            Expr 4294967295 [0-5]: Call:
                Expr 4294967295 [0-3]: Path: Path 4294967295 [0-3] (Ident 4294967295 [0-3] "Foo")
                Expr 4294967295 [3-5]: Unit"#]],
    );
}

#[test]
fn call_op_one() {
    check(
        expr,
        "Foo(x)",
        &expect![[r#"
            Expr 4294967295 [0-6]: Call:
                Expr 4294967295 [0-3]: Path: Path 4294967295 [0-3] (Ident 4294967295 [0-3] "Foo")
                Expr 4294967295 [3-6]: Paren: Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")"#]],
    );
}

#[test]
fn call_op_singleton_tuple() {
    check(
        expr,
        "Foo(x,)",
        &expect![[r#"
            Expr 4294967295 [0-7]: Call:
                Expr 4294967295 [0-3]: Path: Path 4294967295 [0-3] (Ident 4294967295 [0-3] "Foo")
                Expr 4294967295 [3-7]: Tuple:
                    Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")"#]],
    );
}

#[test]
fn call_op_pair() {
    check(
        expr,
        "Foo(x, y)",
        &expect![[r#"
            Expr 4294967295 [0-9]: Call:
                Expr 4294967295 [0-3]: Path: Path 4294967295 [0-3] (Ident 4294967295 [0-3] "Foo")
                Expr 4294967295 [3-9]: Tuple:
                    Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "x")
                    Expr 4294967295 [7-8]: Path: Path 4294967295 [7-8] (Ident 4294967295 [7-8] "y")"#]],
    );
}

#[test]
fn call_with_array() {
    check(
        expr,
        "f([1, 2])",
        &expect![[r#"
            Expr 4294967295 [0-9]: Call:
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "f")
                Expr 4294967295 [1-9]: Paren: Expr 4294967295 [2-8]: Array:
                    Expr 4294967295 [3-4]: Lit: Int(1)
                    Expr 4294967295 [6-7]: Lit: Int(2)"#]],
    );
}

#[test]
fn call_partial_app() {
    check(
        expr,
        "Foo(1, _, 3)",
        &expect![[r#"
            Expr 4294967295 [0-12]: Call:
                Expr 4294967295 [0-3]: Path: Path 4294967295 [0-3] (Ident 4294967295 [0-3] "Foo")
                Expr 4294967295 [3-12]: Tuple:
                    Expr 4294967295 [4-5]: Lit: Int(1)
                    Expr 4294967295 [7-8]: Hole
                    Expr 4294967295 [10-11]: Lit: Int(3)"#]],
    );
}

#[test]
fn call_partial_app_nested() {
    check(
        expr,
        "Foo(1, _, (_, 4))",
        &expect![[r#"
            Expr 4294967295 [0-17]: Call:
                Expr 4294967295 [0-3]: Path: Path 4294967295 [0-3] (Ident 4294967295 [0-3] "Foo")
                Expr 4294967295 [3-17]: Tuple:
                    Expr 4294967295 [4-5]: Lit: Int(1)
                    Expr 4294967295 [7-8]: Hole
                    Expr 4294967295 [10-16]: Tuple:
                        Expr 4294967295 [11-12]: Hole
                        Expr 4294967295 [14-15]: Lit: Int(4)"#]],
    );
}

#[test]
fn call_index_ops() {
    check(
        expr,
        "f()[i]",
        &expect![[r#"
            Expr 4294967295 [0-6]: Index:
                Expr 4294967295 [0-3]: Call:
                    Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "f")
                    Expr 4294967295 [1-3]: Unit
                Expr 4294967295 [4-5]: Path: Path 4294967295 [4-5] (Ident 4294967295 [4-5] "i")"#]],
    );
}

#[test]
fn index_call_ops() {
    check(
        expr,
        "fs[i]()",
        &expect![[r#"
            Expr 4294967295 [0-7]: Call:
                Expr 4294967295 [0-5]: Index:
                    Expr 4294967295 [0-2]: Path: Path 4294967295 [0-2] (Ident 4294967295 [0-2] "fs")
                    Expr 4294967295 [3-4]: Path: Path 4294967295 [3-4] (Ident 4294967295 [3-4] "i")
                Expr 4294967295 [5-7]: Unit"#]],
    );
}

#[test]
fn range_op() {
    check(
        expr,
        "x..y",
        &expect![[r#"
            Expr 4294967295 [0-4]: Range:
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")<no step>
                Expr 4294967295 [3-4]: Path: Path 4294967295 [3-4] (Ident 4294967295 [3-4] "y")"#]],
    );
}

#[test]
fn range_op_with_step() {
    check(
        expr,
        "x..y..z",
        &expect![[r#"
            Expr 4294967295 [0-7]: Range:
                Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "x")
                Expr 4294967295 [3-4]: Path: Path 4294967295 [3-4] (Ident 4294967295 [3-4] "y")
                Expr 4294967295 [6-7]: Path: Path 4294967295 [6-7] (Ident 4294967295 [6-7] "z")"#]],
    );
}

#[test]
fn range_complex_stop() {
    check(
        expr,
        "0..Length(xs) - 1",
        &expect![[r#"
            Expr 4294967295 [0-17]: Range:
                Expr 4294967295 [0-1]: Lit: Int(0)<no step>
                Expr 4294967295 [3-17]: BinOp (Sub):
                    Expr 4294967295 [3-13]: Call:
                        Expr 4294967295 [3-9]: Path: Path 4294967295 [3-9] (Ident 4294967295 [3-9] "Length")
                        Expr 4294967295 [9-13]: Paren: Expr 4294967295 [10-12]: Path: Path 4294967295 [10-12] (Ident 4294967295 [10-12] "xs")
                    Expr 4294967295 [16-17]: Lit: Int(1)"#]],
    );
}

#[test]
fn range_complex_start() {
    check(
        expr,
        "i + 1..n",
        &expect![[r#"
            Expr 4294967295 [0-8]: Range:
                Expr 4294967295 [0-5]: BinOp (Add):
                    Expr 4294967295 [0-1]: Path: Path 4294967295 [0-1] (Ident 4294967295 [0-1] "i")
                    Expr 4294967295 [4-5]: Lit: Int(1)<no step>
                Expr 4294967295 [7-8]: Path: Path 4294967295 [7-8] (Ident 4294967295 [7-8] "n")"#]],
    );
}

#[test]
fn range_complex_step() {
    check(
        expr,
        "0..s + 1..n",
        &expect![[r#"
            Expr 4294967295 [0-11]: Range:
                Expr 4294967295 [0-1]: Lit: Int(0)
                Expr 4294967295 [3-8]: BinOp (Add):
                    Expr 4294967295 [3-4]: Path: Path 4294967295 [3-4] (Ident 4294967295 [3-4] "s")
                    Expr 4294967295 [7-8]: Lit: Int(1)
                Expr 4294967295 [10-11]: Path: Path 4294967295 [10-11] (Ident 4294967295 [10-11] "n")"#]],
    );
}

#[test]
fn range_start_open() {
    check(
        expr,
        "2...",
        &expect![[r#"
            Expr 4294967295 [0-4]: Range:
                Expr 4294967295 [0-1]: Lit: Int(2)<no step><no stop>"#]],
    );
}

#[test]
fn range_start_step_open() {
    check(
        expr,
        "3..2...",
        &expect![[r#"
            Expr 4294967295 [0-7]: Range:
                Expr 4294967295 [0-1]: Lit: Int(3)
                Expr 4294967295 [3-4]: Lit: Int(2)<no stop>"#]],
    );
}

#[test]
fn range_open_stop() {
    check(
        expr,
        "...2",
        &expect![[r#"
            Expr 4294967295 [0-4]: Range:<no start><no step>
                Expr 4294967295 [3-4]: Lit: Int(2)"#]],
    );
}

#[test]
fn range_open_step_stop() {
    check(
        expr,
        "...2..3",
        &expect![[r#"
            Expr 4294967295 [0-7]: Range:<no start>
                Expr 4294967295 [3-4]: Lit: Int(2)
                Expr 4294967295 [6-7]: Lit: Int(3)"#]],
    );
}

#[test]
fn range_open_step_open() {
    check(
        expr,
        "...2...",
        &expect![[r#"
            Expr 4294967295 [0-7]: Range:<no start>
                Expr 4294967295 [3-4]: Lit: Int(2)<no stop>"#]],
    );
}

#[test]
fn function_lambda() {
    check(
        expr,
        "x -> x + 1",
        &expect![[r#"
            Expr 4294967295 [0-10]: Lambda (Function):
                Pat 4294967295 [0-1]: Bind:
                    Ident 4294967295 [0-1] "x"
                Expr 4294967295 [5-10]: BinOp (Add):
                    Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "x")
                    Expr 4294967295 [9-10]: Lit: Int(1)"#]],
    );
}

#[test]
fn operation_lambda() {
    check(
        expr,
        "q => X(q)",
        &expect![[r#"
            Expr 4294967295 [0-9]: Lambda (Operation):
                Pat 4294967295 [0-1]: Bind:
                    Ident 4294967295 [0-1] "q"
                Expr 4294967295 [5-9]: Call:
                    Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "X")
                    Expr 4294967295 [6-9]: Paren: Expr 4294967295 [7-8]: Path: Path 4294967295 [7-8] (Ident 4294967295 [7-8] "q")"#]],
    );
}

#[test]
fn lambda_tuple_input() {
    check(
        expr,
        "(x, y) -> x + y",
        &expect![[r#"
            Expr 4294967295 [0-15]: Lambda (Function):
                Pat 4294967295 [0-6]: Tuple:
                    Pat 4294967295 [1-2]: Bind:
                        Ident 4294967295 [1-2] "x"
                    Pat 4294967295 [4-5]: Bind:
                        Ident 4294967295 [4-5] "y"
                Expr 4294967295 [10-15]: BinOp (Add):
                    Expr 4294967295 [10-11]: Path: Path 4294967295 [10-11] (Ident 4294967295 [10-11] "x")
                    Expr 4294967295 [14-15]: Path: Path 4294967295 [14-15] (Ident 4294967295 [14-15] "y")"#]],
    );
}

#[test]
fn lambda_invalid_input() {
    check(
        expr,
        "x + 1 -> x",
        &expect![[r#"
            Err(
                Convert(
                    "pattern",
                    "expression",
                    Span {
                        lo: 0,
                        hi: 5,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn lambda_invalid_tuple_input() {
    check(
        expr,
        "(x, y + 1) -> x + y",
        &expect![[r#"
            Err(
                Convert(
                    "pattern",
                    "expression",
                    Span {
                        lo: 4,
                        hi: 9,
                    },
                ),
            )
        "#]],
    );
}
