// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::ty;
use crate::tests::check;
use expect_test::expect;

#[test]
fn ty_big_int() {
    check(
        ty,
        "BigInt",
        &expect![[r#"Type _id_ [0-6]: Path: Path _id_ [0-6] (Ident _id_ [0-6] "BigInt")"#]],
    );
}

#[test]
fn ty_bool() {
    check(
        ty,
        "Bool",
        &expect![[r#"Type _id_ [0-4]: Path: Path _id_ [0-4] (Ident _id_ [0-4] "Bool")"#]],
    );
}

#[test]
fn ty_double() {
    check(
        ty,
        "Double",
        &expect![[r#"Type _id_ [0-6]: Path: Path _id_ [0-6] (Ident _id_ [0-6] "Double")"#]],
    );
}

#[test]
fn ty_int() {
    check(
        ty,
        "Int",
        &expect![[r#"Type _id_ [0-3]: Path: Path _id_ [0-3] (Ident _id_ [0-3] "Int")"#]],
    );
}

#[test]
fn ty_pauli() {
    check(
        ty,
        "Pauli",
        &expect![[r#"Type _id_ [0-5]: Path: Path _id_ [0-5] (Ident _id_ [0-5] "Pauli")"#]],
    );
}

#[test]
fn ty_qubit() {
    check(
        ty,
        "Qubit",
        &expect![[r#"Type _id_ [0-5]: Path: Path _id_ [0-5] (Ident _id_ [0-5] "Qubit")"#]],
    );
}

#[test]
fn ty_range() {
    check(
        ty,
        "Range",
        &expect![[r#"Type _id_ [0-5]: Path: Path _id_ [0-5] (Ident _id_ [0-5] "Range")"#]],
    );
}

#[test]
fn ty_result() {
    check(
        ty,
        "Result",
        &expect![[r#"Type _id_ [0-6]: Path: Path _id_ [0-6] (Ident _id_ [0-6] "Result")"#]],
    );
}

#[test]
fn ty_string() {
    check(
        ty,
        "String",
        &expect![[r#"Type _id_ [0-6]: Path: Path _id_ [0-6] (Ident _id_ [0-6] "String")"#]],
    );
}

#[test]
fn ty_unit() {
    check(
        ty,
        "Unit",
        &expect![[r#"Type _id_ [0-4]: Path: Path _id_ [0-4] (Ident _id_ [0-4] "Unit")"#]],
    );
}

#[test]
fn ty_param() {
    check(
        ty,
        "'T",
        &expect![[r#"Type _id_ [0-2]: Type Param: Ident _id_ [0-2] "'T""#]],
    );
}

#[test]
fn ty_hole() {
    check(ty, "_", &expect!["Type _id_ [0-1]: Hole"]);
}

#[test]
fn ty_path() {
    check(
        ty,
        "Foo",
        &expect![[r#"Type _id_ [0-3]: Path: Path _id_ [0-3] (Ident _id_ [0-3] "Foo")"#]],
    );
}

#[test]
fn ty_path2() {
    check(
        ty,
        "Foo.Bar",
        &expect![[
            r#"Type _id_ [0-7]: Path: Path _id_ [0-7] (Ident _id_ [0-3] "Foo") (Ident _id_ [4-7] "Bar")"#
        ]],
    );
}

#[test]
fn ty_paren() {
    check(
        ty,
        "(Int)",
        &expect![[
            r#"Type _id_ [0-5]: Paren: Type _id_ [1-4]: Path: Path _id_ [1-4] (Ident _id_ [1-4] "Int")"#
        ]],
    );
}

#[test]
fn ty_singleton_tuple() {
    check(
        ty,
        "(Int,)",
        &expect![[r#"
            Type _id_ [0-6]: Tuple:
                Type _id_ [1-4]: Path: Path _id_ [1-4] (Ident _id_ [1-4] "Int")"#]],
    );
}

#[test]
fn ty_tuple() {
    check(
        ty,
        "(Int, Bool)",
        &expect![[r#"
            Type _id_ [0-11]: Tuple:
                Type _id_ [1-4]: Path: Path _id_ [1-4] (Ident _id_ [1-4] "Int")
                Type _id_ [6-10]: Path: Path _id_ [6-10] (Ident _id_ [6-10] "Bool")"#]],
    );
}

#[test]
fn ty_tuple2() {
    check(
        ty,
        "((Int, Bool), Double)",
        &expect![[r#"
            Type _id_ [0-21]: Tuple:
                Type _id_ [1-12]: Tuple:
                    Type _id_ [2-5]: Path: Path _id_ [2-5] (Ident _id_ [2-5] "Int")
                    Type _id_ [7-11]: Path: Path _id_ [7-11] (Ident _id_ [7-11] "Bool")
                Type _id_ [14-20]: Path: Path _id_ [14-20] (Ident _id_ [14-20] "Double")"#]],
    );
}

#[test]
fn ty_array() {
    check(
        ty,
        "Int[]",
        &expect![[
            r#"Type _id_ [0-5]: Array: Type _id_ [0-3]: Path: Path _id_ [0-3] (Ident _id_ [0-3] "Int")"#
        ]],
    );
}

#[test]
fn ty_array2() {
    check(
        ty,
        "Int[][]",
        &expect![[
            r#"Type _id_ [0-7]: Array: Type _id_ [0-5]: Array: Type _id_ [0-3]: Path: Path _id_ [0-3] (Ident _id_ [0-3] "Int")"#
        ]],
    );
}

#[test]
fn ty_tuple_array() {
    check(
        ty,
        "(Int, Bool)[]",
        &expect![[r#"
            Type _id_ [0-13]: Array: Type _id_ [0-11]: Tuple:
                Type _id_ [1-4]: Path: Path _id_ [1-4] (Ident _id_ [1-4] "Int")
                Type _id_ [6-10]: Path: Path _id_ [6-10] (Ident _id_ [6-10] "Bool")"#]],
    );
}

#[test]
fn ty_function() {
    check(
        ty,
        "Int -> Int",
        &expect![[r#"
            Type _id_ [0-10]: Arrow (Function):
                param: Type _id_ [0-3]: Path: Path _id_ [0-3] (Ident _id_ [0-3] "Int")
                return: Type _id_ [7-10]: Path: Path _id_ [7-10] (Ident _id_ [7-10] "Int")"#]],
    );
}

#[test]
fn ty_operation() {
    check(
        ty,
        "Int => Int",
        &expect![[r#"
            Type _id_ [0-10]: Arrow (Operation):
                param: Type _id_ [0-3]: Path: Path _id_ [0-3] (Ident _id_ [0-3] "Int")
                return: Type _id_ [7-10]: Path: Path _id_ [7-10] (Ident _id_ [7-10] "Int")"#]],
    );
}

#[test]
fn ty_curried_function() {
    check(
        ty,
        "Int -> Int -> Int",
        &expect![[r#"
            Type _id_ [0-17]: Arrow (Function):
                param: Type _id_ [0-3]: Path: Path _id_ [0-3] (Ident _id_ [0-3] "Int")
                return: Type _id_ [7-17]: Arrow (Function):
                    param: Type _id_ [7-10]: Path: Path _id_ [7-10] (Ident _id_ [7-10] "Int")
                    return: Type _id_ [14-17]: Path: Path _id_ [14-17] (Ident _id_ [14-17] "Int")"#]],
    );
}

#[test]
fn ty_higher_order_function() {
    check(
        ty,
        "(Int -> Int) -> Int",
        &expect![[r#"
            Type _id_ [0-19]: Arrow (Function):
                param: Type _id_ [0-12]: Paren: Type _id_ [1-11]: Arrow (Function):
                    param: Type _id_ [1-4]: Path: Path _id_ [1-4] (Ident _id_ [1-4] "Int")
                    return: Type _id_ [8-11]: Path: Path _id_ [8-11] (Ident _id_ [8-11] "Int")
                return: Type _id_ [16-19]: Path: Path _id_ [16-19] (Ident _id_ [16-19] "Int")"#]],
    );
}

#[test]
fn op_ty_is_adj() {
    check(
        ty,
        "Qubit => Unit is Adj",
        &expect![[r#"
            Type _id_ [0-20]: Arrow (Operation):
                param: Type _id_ [0-5]: Path: Path _id_ [0-5] (Ident _id_ [0-5] "Qubit")
                return: Type _id_ [9-13]: Path: Path _id_ [9-13] (Ident _id_ [9-13] "Unit")
                functors: Functor Expr _id_ [17-20]: Adj"#]],
    );
}

#[test]
fn op_ty_is_adj_ctl() {
    check(
        ty,
        "Qubit => Unit is Adj + Ctl",
        &expect![[r#"
            Type _id_ [0-26]: Arrow (Operation):
                param: Type _id_ [0-5]: Path: Path _id_ [0-5] (Ident _id_ [0-5] "Qubit")
                return: Type _id_ [9-13]: Path: Path _id_ [9-13] (Ident _id_ [9-13] "Unit")
                functors: Functor Expr _id_ [17-26]: BinOp Union: (Functor Expr _id_ [17-20]: Adj) (Functor Expr _id_ [23-26]: Ctl)"#]],
    );
}

#[test]
fn op_ty_is_nested() {
    check(
        ty,
        "Qubit => Qubit => Unit is Adj is Ctl",
        &expect![[r#"
            Type _id_ [0-36]: Arrow (Operation):
                param: Type _id_ [0-5]: Path: Path _id_ [0-5] (Ident _id_ [0-5] "Qubit")
                return: Type _id_ [9-29]: Arrow (Operation):
                    param: Type _id_ [9-14]: Path: Path _id_ [9-14] (Ident _id_ [9-14] "Qubit")
                    return: Type _id_ [18-22]: Path: Path _id_ [18-22] (Ident _id_ [18-22] "Unit")
                    functors: Functor Expr _id_ [26-29]: Adj
                functors: Functor Expr _id_ [33-36]: Ctl"#]],
    );
}

#[test]
fn op_ty_is_nested_paren() {
    check(
        ty,
        "Qubit => (Qubit => Unit) is Ctl",
        &expect![[r#"
            Type _id_ [0-31]: Arrow (Operation):
                param: Type _id_ [0-5]: Path: Path _id_ [0-5] (Ident _id_ [0-5] "Qubit")
                return: Type _id_ [9-24]: Paren: Type _id_ [10-23]: Arrow (Operation):
                    param: Type _id_ [10-15]: Path: Path _id_ [10-15] (Ident _id_ [10-15] "Qubit")
                    return: Type _id_ [19-23]: Path: Path _id_ [19-23] (Ident _id_ [19-23] "Unit")
                functors: Functor Expr _id_ [28-31]: Ctl"#]],
    );
}

#[test]
fn op_ty_is_paren() {
    check(
        ty,
        "Qubit => Unit is (Adj)",
        &expect![[r#"
            Type _id_ [0-22]: Arrow (Operation):
                param: Type _id_ [0-5]: Path: Path _id_ [0-5] (Ident _id_ [0-5] "Qubit")
                return: Type _id_ [9-13]: Path: Path _id_ [9-13] (Ident _id_ [9-13] "Unit")
                functors: Functor Expr _id_ [17-22]: Paren: Functor Expr _id_ [18-21]: Adj"#]],
    );
}

#[test]
fn op_ty_union_assoc() {
    check(
        ty,
        "Qubit => Unit is Adj + Adj + Adj",
        &expect![[r#"
            Type _id_ [0-32]: Arrow (Operation):
                param: Type _id_ [0-5]: Path: Path _id_ [0-5] (Ident _id_ [0-5] "Qubit")
                return: Type _id_ [9-13]: Path: Path _id_ [9-13] (Ident _id_ [9-13] "Unit")
                functors: Functor Expr _id_ [17-32]: BinOp Union: (Functor Expr _id_ [17-26]: BinOp Union: (Functor Expr _id_ [17-20]: Adj) (Functor Expr _id_ [23-26]: Adj)) (Functor Expr _id_ [29-32]: Adj)"#]],
    );
}

#[test]
fn op_ty_intersect_assoc() {
    check(
        ty,
        "Qubit => Unit is Adj * Adj * Adj",
        &expect![[r#"
            Type _id_ [0-32]: Arrow (Operation):
                param: Type _id_ [0-5]: Path: Path _id_ [0-5] (Ident _id_ [0-5] "Qubit")
                return: Type _id_ [9-13]: Path: Path _id_ [9-13] (Ident _id_ [9-13] "Unit")
                functors: Functor Expr _id_ [17-32]: BinOp Intersect: (Functor Expr _id_ [17-26]: BinOp Intersect: (Functor Expr _id_ [17-20]: Adj) (Functor Expr _id_ [23-26]: Adj)) (Functor Expr _id_ [29-32]: Adj)"#]],
    );
}

#[test]
fn op_ty_is_prec() {
    check(
        ty,
        "Qubit => Unit is Adj + Adj * Ctl",
        &expect![[r#"
            Type _id_ [0-32]: Arrow (Operation):
                param: Type _id_ [0-5]: Path: Path _id_ [0-5] (Ident _id_ [0-5] "Qubit")
                return: Type _id_ [9-13]: Path: Path _id_ [9-13] (Ident _id_ [9-13] "Unit")
                functors: Functor Expr _id_ [17-32]: BinOp Union: (Functor Expr _id_ [17-20]: Adj) (Functor Expr _id_ [23-32]: BinOp Intersect: (Functor Expr _id_ [23-26]: Adj) (Functor Expr _id_ [29-32]: Ctl))"#]],
    );
}
