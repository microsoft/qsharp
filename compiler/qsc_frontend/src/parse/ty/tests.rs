// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::ty;
use crate::parse::tests::check;
use expect_test::expect;

#[test]
fn ty_big_int() {
    check(
        ty,
        "BigInt",
        &expect!["Type 4294967295 [0-6]: Prim (BigInt)"],
    );
}

#[test]
fn ty_bool() {
    check(ty, "Bool", &expect!["Type 4294967295 [0-4]: Prim (Bool)"]);
}

#[test]
fn ty_double() {
    check(
        ty,
        "Double",
        &expect!["Type 4294967295 [0-6]: Prim (Double)"],
    );
}

#[test]
fn ty_int() {
    check(ty, "Int", &expect!["Type 4294967295 [0-3]: Prim (Int)"]);
}

#[test]
fn ty_pauli() {
    check(ty, "Pauli", &expect!["Type 4294967295 [0-5]: Prim (Pauli)"]);
}

#[test]
fn ty_qubit() {
    check(ty, "Qubit", &expect!["Type 4294967295 [0-5]: Prim (Qubit)"]);
}

#[test]
fn ty_range() {
    check(ty, "Range", &expect!["Type 4294967295 [0-5]: Prim (Range)"]);
}

#[test]
fn ty_result() {
    check(
        ty,
        "Result",
        &expect!["Type 4294967295 [0-6]: Prim (Result)"],
    );
}

#[test]
fn ty_string() {
    check(
        ty,
        "String",
        &expect!["Type 4294967295 [0-6]: Prim (String)"],
    );
}

#[test]
fn ty_unit() {
    check(ty, "Unit", &expect!["Type 4294967295 [0-4]: Unit"]);
}

#[test]
fn ty_var() {
    check(ty, "'T", &expect!["Type 4294967295 [0-2]: Type Var T"]);
}

#[test]
fn ty_hole() {
    check(ty, "_", &expect!["Type 4294967295 [0-1]: Hole"]);
}

#[test]
fn ty_path() {
    check(
        ty,
        "Foo",
        &expect![[
            r#"Type 4294967295 [0-3]: Path: Path 4294967295 [0-3] (Ident 4294967295 [0-3] "Foo")"#
        ]],
    );
}

#[test]
fn ty_path2() {
    check(
        ty,
        "Foo.Bar",
        &expect![[
            r#"Type 4294967295 [0-7]: Path: Path 4294967295 [0-7] (Ident 4294967295 [0-3] "Foo") (Ident 4294967295 [4-7] "Bar")"#
        ]],
    );
}

#[test]
fn ty_paren() {
    check(
        ty,
        "(Int)",
        &expect!["Type 4294967295 [0-5]: Paren: Type 4294967295 [1-4]: Prim (Int)"],
    );
}

#[test]
fn ty_singleton_tuple() {
    check(
        ty,
        "(Int,)",
        &expect![[r#"
            Type 4294967295 [0-6]: Tuple:
                Type 4294967295 [1-4]: Prim (Int)"#]],
    );
}

#[test]
fn ty_tuple() {
    check(
        ty,
        "(Int, Bool)",
        &expect![[r#"
            Type 4294967295 [0-11]: Tuple:
                Type 4294967295 [1-4]: Prim (Int)
                Type 4294967295 [6-10]: Prim (Bool)"#]],
    );
}

#[test]
fn ty_tuple2() {
    check(
        ty,
        "((Int, Bool), Double)",
        &expect![[r#"
            Type 4294967295 [0-21]: Tuple:
                Type 4294967295 [1-12]: Tuple:
                    Type 4294967295 [2-5]: Prim (Int)
                    Type 4294967295 [7-11]: Prim (Bool)
                Type 4294967295 [14-20]: Prim (Double)"#]],
    );
}

#[test]
fn ty_array() {
    check(
        ty,
        "Int[]",
        &expect![[r#"
            Type 4294967295 [0-5]: App:
                base type: Type 4294967295 [3-5]: Prim (Array)
                arg types:
                    Type 4294967295 [0-3]: Prim (Int)"#]],
    );
}

#[test]
fn ty_array2() {
    check(
        ty,
        "Int[][]",
        &expect![[r#"
            Type 4294967295 [0-7]: App:
                base type: Type 4294967295 [5-7]: Prim (Array)
                arg types:
                    Type 4294967295 [0-5]: App:
                        base type: Type 4294967295 [3-5]: Prim (Array)
                        arg types:
                            Type 4294967295 [0-3]: Prim (Int)"#]],
    );
}

#[test]
fn ty_tuple_array() {
    check(
        ty,
        "(Int, Bool)[]",
        &expect![[r#"
            Type 4294967295 [0-13]: App:
                base type: Type 4294967295 [11-13]: Prim (Array)
                arg types:
                    Type 4294967295 [0-11]: Tuple:
                        Type 4294967295 [1-4]: Prim (Int)
                        Type 4294967295 [6-10]: Prim (Bool)"#]],
    );
}

#[test]
fn ty_function() {
    check(
        ty,
        "Int -> Int",
        &expect![[r#"
            Type 4294967295 [0-10]: Arrow (Function):
                param: Type 4294967295 [0-3]: Prim (Int)
                return: Type 4294967295 [7-10]: Prim (Int)"#]],
    );
}

#[test]
fn ty_operation() {
    check(
        ty,
        "Int => Int",
        &expect![[r#"
            Type 4294967295 [0-10]: Arrow (Operation):
                param: Type 4294967295 [0-3]: Prim (Int)
                return: Type 4294967295 [7-10]: Prim (Int)"#]],
    );
}

#[test]
fn ty_curried_function() {
    check(
        ty,
        "Int -> Int -> Int",
        &expect![[r#"
            Type 4294967295 [0-17]: Arrow (Function):
                param: Type 4294967295 [0-3]: Prim (Int)
                return: Type 4294967295 [7-17]: Arrow (Function):
                    param: Type 4294967295 [7-10]: Prim (Int)
                    return: Type 4294967295 [14-17]: Prim (Int)"#]],
    );
}

#[test]
fn ty_higher_order_function() {
    check(
        ty,
        "(Int -> Int) -> Int",
        &expect![[r#"
            Type 4294967295 [0-19]: Arrow (Function):
                param: Type 4294967295 [0-12]: Paren: Type 4294967295 [1-11]: Arrow (Function):
                    param: Type 4294967295 [1-4]: Prim (Int)
                    return: Type 4294967295 [8-11]: Prim (Int)
                return: Type 4294967295 [16-19]: Prim (Int)"#]],
    );
}

#[test]
fn op_ty_is_adj() {
    check(
        ty,
        "Qubit => Unit is Adj",
        &expect![[r#"
            Type 4294967295 [0-20]: Arrow (Operation):
                param: Type 4294967295 [0-5]: Prim (Qubit)
                return: Type 4294967295 [9-13]: Unit
                functors: Functor Expr 4294967295 [17-20]: Adj"#]],
    );
}

#[test]
fn op_ty_is_adj_ctl() {
    check(
        ty,
        "Qubit => Unit is Adj + Ctl",
        &expect![[r#"
            Type 4294967295 [0-26]: Arrow (Operation):
                param: Type 4294967295 [0-5]: Prim (Qubit)
                return: Type 4294967295 [9-13]: Unit
                functors: Functor Expr 4294967295 [17-26]: BinOp Union: (Functor Expr 4294967295 [17-20]: Adj) (Functor Expr 4294967295 [23-26]: Ctl)"#]],
    );
}

#[test]
fn op_ty_is_nested() {
    check(
        ty,
        "Qubit => Qubit => Unit is Adj is Ctl",
        &expect![[r#"
            Type 4294967295 [0-36]: Arrow (Operation):
                param: Type 4294967295 [0-5]: Prim (Qubit)
                return: Type 4294967295 [9-29]: Arrow (Operation):
                    param: Type 4294967295 [9-14]: Prim (Qubit)
                    return: Type 4294967295 [18-22]: Unit
                    functors: Functor Expr 4294967295 [26-29]: Adj
                functors: Functor Expr 4294967295 [33-36]: Ctl"#]],
    );
}

#[test]
fn op_ty_is_nested_paren() {
    check(
        ty,
        "Qubit => (Qubit => Unit) is Ctl",
        &expect![[r#"
            Type 4294967295 [0-31]: Arrow (Operation):
                param: Type 4294967295 [0-5]: Prim (Qubit)
                return: Type 4294967295 [9-24]: Paren: Type 4294967295 [10-23]: Arrow (Operation):
                    param: Type 4294967295 [10-15]: Prim (Qubit)
                    return: Type 4294967295 [19-23]: Unit
                functors: Functor Expr 4294967295 [28-31]: Ctl"#]],
    );
}

#[test]
fn op_ty_is_paren() {
    check(
        ty,
        "Qubit => Unit is (Adj)",
        &expect![[r#"
            Type 4294967295 [0-22]: Arrow (Operation):
                param: Type 4294967295 [0-5]: Prim (Qubit)
                return: Type 4294967295 [9-13]: Unit
                functors: Functor Expr 4294967295 [17-22]: Paren: Functor Expr 4294967295 [18-21]: Adj"#]],
    );
}

#[test]
fn op_ty_union_assoc() {
    check(
        ty,
        "Qubit => Unit is Adj + Adj + Adj",
        &expect![[r#"
            Type 4294967295 [0-32]: Arrow (Operation):
                param: Type 4294967295 [0-5]: Prim (Qubit)
                return: Type 4294967295 [9-13]: Unit
                functors: Functor Expr 4294967295 [17-32]: BinOp Union: (Functor Expr 4294967295 [17-26]: BinOp Union: (Functor Expr 4294967295 [17-20]: Adj) (Functor Expr 4294967295 [23-26]: Adj)) (Functor Expr 4294967295 [29-32]: Adj)"#]],
    );
}

#[test]
fn op_ty_intersect_assoc() {
    check(
        ty,
        "Qubit => Unit is Adj * Adj * Adj",
        &expect![[r#"
            Type 4294967295 [0-32]: Arrow (Operation):
                param: Type 4294967295 [0-5]: Prim (Qubit)
                return: Type 4294967295 [9-13]: Unit
                functors: Functor Expr 4294967295 [17-32]: BinOp Intersect: (Functor Expr 4294967295 [17-26]: BinOp Intersect: (Functor Expr 4294967295 [17-20]: Adj) (Functor Expr 4294967295 [23-26]: Adj)) (Functor Expr 4294967295 [29-32]: Adj)"#]],
    );
}

#[test]
fn op_ty_is_prec() {
    check(
        ty,
        "Qubit => Unit is Adj + Adj * Ctl",
        &expect![[r#"
            Type 4294967295 [0-32]: Arrow (Operation):
                param: Type 4294967295 [0-5]: Prim (Qubit)
                return: Type 4294967295 [9-13]: Unit
                functors: Functor Expr 4294967295 [17-32]: BinOp Union: (Functor Expr 4294967295 [17-20]: Adj) (Functor Expr 4294967295 [23-32]: BinOp Intersect: (Functor Expr 4294967295 [23-26]: Adj) (Functor Expr 4294967295 [29-32]: Ctl))"#]],
    );
}
