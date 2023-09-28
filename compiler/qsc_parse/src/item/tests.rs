// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{parse, parse_attr, parse_namespaces, parse_spec_decl};
use crate::tests::{check, check_vec};
use expect_test::expect;

#[test]
fn body_intrinsic() {
    check(
        parse_spec_decl,
        "body intrinsic;",
        &expect!["SpecDecl _id_ [0-15] (Body): Gen: Intrinsic"],
    );
}

#[test]
fn adjoint_self() {
    check(
        parse_spec_decl,
        "adjoint self;",
        &expect!["SpecDecl _id_ [0-13] (Adj): Gen: Slf"],
    );
}

#[test]
fn adjoint_invert() {
    check(
        parse_spec_decl,
        "adjoint invert;",
        &expect!["SpecDecl _id_ [0-15] (Adj): Gen: Invert"],
    );
}

#[test]
fn controlled_distribute() {
    check(
        parse_spec_decl,
        "controlled distribute;",
        &expect!["SpecDecl _id_ [0-22] (Ctl): Gen: Distribute"],
    );
}

#[test]
fn controlled_adjoint_auto() {
    check(
        parse_spec_decl,
        "controlled adjoint auto;",
        &expect!["SpecDecl _id_ [0-24] (CtlAdj): Gen: Auto"],
    );
}

#[test]
fn spec_gen_missing_semi() {
    check(
        parse_spec_decl,
        "body intrinsic",
        &expect![[r#"
            Error(
                Token(
                    Semi,
                    Eof,
                    Span {
                        lo: 14,
                        hi: 14,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn spec_invalid_gen() {
    check(
        parse_spec_decl,
        "adjoint foo;",
        &expect![[r#"
            Error(
                Token(
                    Open(
                        Brace,
                    ),
                    Semi,
                    Span {
                        lo: 11,
                        hi: 12,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn open_no_alias() {
    check(
        parse,
        "open Foo.Bar.Baz;",
        &expect![[r#"
            Item _id_ [0-17]:
                Open (Ident _id_ [5-16] "Foo.Bar.Baz")"#]],
    );
}

#[test]
fn open_alias() {
    check(
        parse,
        "open Foo.Bar.Baz as Baz;",
        &expect![[r#"
            Item _id_ [0-24]:
                Open (Ident _id_ [5-16] "Foo.Bar.Baz") (Ident _id_ [20-23] "Baz")"#]],
    );
}

#[test]
fn open_alias_dot() {
    check(
        parse,
        "open Foo.Bar.Baz as Bar.Baz;",
        &expect![[r#"
            Item _id_ [0-28]:
                Open (Ident _id_ [5-16] "Foo.Bar.Baz") (Ident _id_ [20-27] "Bar.Baz")"#]],
    );
}

#[test]
fn ty_decl() {
    check(
        parse,
        "newtype Foo = Unit;",
        &expect![[r#"
            Item _id_ [0-19]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-18]: Field:
                    Type _id_ [14-18]: Path: Path _id_ [14-18] (Ident _id_ [14-18] "Unit")"#]],
    );
}

#[test]
fn ty_decl_field_name() {
    check(
        parse,
        "newtype Foo = Bar : Int;",
        &expect![[r#"
            Item _id_ [0-24]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-23]: Field:
                    Ident _id_ [14-17] "Bar"
                    Type _id_ [20-23]: Path: Path _id_ [20-23] (Ident _id_ [20-23] "Int")"#]],
    );
}

#[test]
fn ty_decl_doc() {
    check(
        parse,
        "/// This is a
        /// doc comment.
        newtype Foo = Int;",
        &expect![[r#"
            Item _id_ [0-65]:
                doc:
                    This is a
                    doc comment.
                New Type (Ident _id_ [55-58] "Foo"): TyDef _id_ [61-64]: Field:
                    Type _id_ [61-64]: Path: Path _id_ [61-64] (Ident _id_ [61-64] "Int")"#]],
    );
}

#[test]
fn ty_def_invalid_field_name() {
    check(
        parse,
        "newtype Foo = Bar.Baz : Int[];",
        &expect![[r#"
            Error(
                Convert(
                    "identifier",
                    "type",
                    Span {
                        lo: 14,
                        hi: 21,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn ty_def_tuple() {
    check(
        parse,
        "newtype Foo = (Int, Int);",
        &expect![[r#"
            Item _id_ [0-25]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-24]: Field:
                    Type _id_ [14-24]: Tuple:
                        Type _id_ [15-18]: Path: Path _id_ [15-18] (Ident _id_ [15-18] "Int")
                        Type _id_ [20-23]: Path: Path _id_ [20-23] (Ident _id_ [20-23] "Int")"#]],
    );
}

#[test]
fn ty_def_tuple_one_named() {
    check(
        parse,
        "newtype Foo = (X : Int, Int);",
        &expect![[r#"
            Item _id_ [0-29]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-28]: Tuple:
                    TyDef _id_ [15-22]: Field:
                        Ident _id_ [15-16] "X"
                        Type _id_ [19-22]: Path: Path _id_ [19-22] (Ident _id_ [19-22] "Int")
                    TyDef _id_ [24-27]: Field:
                        Type _id_ [24-27]: Path: Path _id_ [24-27] (Ident _id_ [24-27] "Int")"#]],
    );
}

#[test]
fn ty_def_tuple_both_named() {
    check(
        parse,
        "newtype Foo = (X : Int, Y : Int);",
        &expect![[r#"
            Item _id_ [0-33]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-32]: Tuple:
                    TyDef _id_ [15-22]: Field:
                        Ident _id_ [15-16] "X"
                        Type _id_ [19-22]: Path: Path _id_ [19-22] (Ident _id_ [19-22] "Int")
                    TyDef _id_ [24-31]: Field:
                        Ident _id_ [24-25] "Y"
                        Type _id_ [28-31]: Path: Path _id_ [28-31] (Ident _id_ [28-31] "Int")"#]],
    );
}

#[test]
fn ty_def_nested_tuple() {
    check(
        parse,
        "newtype Foo = ((X : Int, Y : Int), Z : Int);",
        &expect![[r#"
            Item _id_ [0-44]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-43]: Tuple:
                    TyDef _id_ [15-33]: Tuple:
                        TyDef _id_ [16-23]: Field:
                            Ident _id_ [16-17] "X"
                            Type _id_ [20-23]: Path: Path _id_ [20-23] (Ident _id_ [20-23] "Int")
                        TyDef _id_ [25-32]: Field:
                            Ident _id_ [25-26] "Y"
                            Type _id_ [29-32]: Path: Path _id_ [29-32] (Ident _id_ [29-32] "Int")
                    TyDef _id_ [35-42]: Field:
                        Ident _id_ [35-36] "Z"
                        Type _id_ [39-42]: Path: Path _id_ [39-42] (Ident _id_ [39-42] "Int")"#]],
    );
}

#[test]
fn ty_def_tuple_with_name() {
    check(
        parse,
        "newtype Foo = Pair : (Int, Int);",
        &expect![[r#"
            Item _id_ [0-32]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-31]: Field:
                    Ident _id_ [14-18] "Pair"
                    Type _id_ [21-31]: Tuple:
                        Type _id_ [22-25]: Path: Path _id_ [22-25] (Ident _id_ [22-25] "Int")
                        Type _id_ [27-30]: Path: Path _id_ [27-30] (Ident _id_ [27-30] "Int")"#]],
    );
}

#[test]
fn ty_def_tuple_array() {
    check(
        parse,
        "newtype Foo = (Int, Int)[];",
        &expect![[r#"
        Item _id_ [0-27]:
            New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-26]: Field:
                Type _id_ [14-26]: Array: Type _id_ [14-24]: Tuple:
                    Type _id_ [15-18]: Path: Path _id_ [15-18] (Ident _id_ [15-18] "Int")
                    Type _id_ [20-23]: Path: Path _id_ [20-23] (Ident _id_ [20-23] "Int")"#]],
    );
}

#[test]
fn ty_def_tuple_lambda_args() {
    check(
        parse,
        "newtype Foo = (Int, Int) -> Int;",
        &expect![[r#"
            Item _id_ [0-32]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-31]: Field:
                    Type _id_ [14-31]: Arrow (Function):
                        param: Type _id_ [14-24]: Tuple:
                            Type _id_ [15-18]: Path: Path _id_ [15-18] (Ident _id_ [15-18] "Int")
                            Type _id_ [20-23]: Path: Path _id_ [20-23] (Ident _id_ [20-23] "Int")
                        return: Type _id_ [28-31]: Path: Path _id_ [28-31] (Ident _id_ [28-31] "Int")"#]],
    );
}

#[test]
fn function_decl() {
    check(
        parse,
        "function Foo() : Unit { body intrinsic; }",
        &expect![[r#"
            Item _id_ [0-41]:
                Callable _id_ [0-41] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    input: Pat _id_ [12-14]: Unit
                    output: Type _id_ [17-21]: Path: Path _id_ [17-21] (Ident _id_ [17-21] "Unit")
                    body: Specializations:
                        SpecDecl _id_ [24-39] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_decl_doc() {
    check(
        parse,
        "/// This is a
        /// doc comment.
        function Foo() : () {}",
        &expect![[r#"
            Item _id_ [0-69]:
                doc:
                    This is a
                    doc comment.
                Callable _id_ [47-69] (Function):
                    name: Ident _id_ [56-59] "Foo"
                    input: Pat _id_ [59-61]: Unit
                    output: Type _id_ [64-66]: Unit
                    body: Block: Block _id_ [67-69]: <empty>"#]],
    );
}

#[test]
fn operation_decl() {
    check(
        parse,
        "operation Foo() : Unit { body intrinsic; }",
        &expect![[r#"
            Item _id_ [0-42]:
                Callable _id_ [0-42] (Operation):
                    name: Ident _id_ [10-13] "Foo"
                    input: Pat _id_ [13-15]: Unit
                    output: Type _id_ [18-22]: Path: Path _id_ [18-22] (Ident _id_ [18-22] "Unit")
                    body: Specializations:
                        SpecDecl _id_ [25-40] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn operation_decl_doc() {
    check(
        parse,
        "/// This is a
        /// doc comment.
        operation Foo() : () {}",
        &expect![[r#"
            Item _id_ [0-70]:
                doc:
                    This is a
                    doc comment.
                Callable _id_ [47-70] (Operation):
                    name: Ident _id_ [57-60] "Foo"
                    input: Pat _id_ [60-62]: Unit
                    output: Type _id_ [65-67]: Unit
                    body: Block: Block _id_ [68-70]: <empty>"#]],
    );
}

#[test]
fn function_one_param() {
    check(
        parse,
        "function Foo(x : Int) : Unit { body intrinsic; }",
        &expect![[r#"
            Item _id_ [0-48]:
                Callable _id_ [0-48] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    input: Pat _id_ [12-21]: Paren:
                        Pat _id_ [13-20]: Bind:
                            Ident _id_ [13-14] "x"
                            Type _id_ [17-20]: Path: Path _id_ [17-20] (Ident _id_ [17-20] "Int")
                    output: Type _id_ [24-28]: Path: Path _id_ [24-28] (Ident _id_ [24-28] "Unit")
                    body: Specializations:
                        SpecDecl _id_ [31-46] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_two_params() {
    check(
        parse,
        "function Foo(x : Int, y : Int) : Unit { body intrinsic; }",
        &expect![[r#"
            Item _id_ [0-57]:
                Callable _id_ [0-57] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    input: Pat _id_ [12-30]: Tuple:
                        Pat _id_ [13-20]: Bind:
                            Ident _id_ [13-14] "x"
                            Type _id_ [17-20]: Path: Path _id_ [17-20] (Ident _id_ [17-20] "Int")
                        Pat _id_ [22-29]: Bind:
                            Ident _id_ [22-23] "y"
                            Type _id_ [26-29]: Path: Path _id_ [26-29] (Ident _id_ [26-29] "Int")
                    output: Type _id_ [33-37]: Path: Path _id_ [33-37] (Ident _id_ [33-37] "Unit")
                    body: Specializations:
                        SpecDecl _id_ [40-55] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_one_ty_param() {
    check(
        parse,
        "function Foo<'T>() : Unit { body intrinsic; }",
        &expect![[r#"
            Item _id_ [0-45]:
                Callable _id_ [0-45] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    generics:
                        Ident _id_ [13-15] "'T"
                    input: Pat _id_ [16-18]: Unit
                    output: Type _id_ [21-25]: Path: Path _id_ [21-25] (Ident _id_ [21-25] "Unit")
                    body: Specializations:
                        SpecDecl _id_ [28-43] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_two_ty_params() {
    check(
        parse,
        "function Foo<'T, 'U>() : Unit { body intrinsic; }",
        &expect![[r#"
            Item _id_ [0-49]:
                Callable _id_ [0-49] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    generics:
                        Ident _id_ [13-15] "'T"
                        Ident _id_ [17-19] "'U"
                    input: Pat _id_ [20-22]: Unit
                    output: Type _id_ [25-29]: Path: Path _id_ [25-29] (Ident _id_ [25-29] "Unit")
                    body: Specializations:
                        SpecDecl _id_ [32-47] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_single_impl() {
    check(
        parse,
        "function Foo(x : Int) : Int { let y = x; y }",
        &expect![[r#"
            Item _id_ [0-44]:
                Callable _id_ [0-44] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    input: Pat _id_ [12-21]: Paren:
                        Pat _id_ [13-20]: Bind:
                            Ident _id_ [13-14] "x"
                            Type _id_ [17-20]: Path: Path _id_ [17-20] (Ident _id_ [17-20] "Int")
                    output: Type _id_ [24-27]: Path: Path _id_ [24-27] (Ident _id_ [24-27] "Int")
                    body: Block: Block _id_ [28-44]:
                        Stmt _id_ [30-40]: Local (Immutable):
                            Pat _id_ [34-35]: Bind:
                                Ident _id_ [34-35] "y"
                            Expr _id_ [38-39]: Path: Path _id_ [38-39] (Ident _id_ [38-39] "x")
                        Stmt _id_ [41-42]: Expr: Expr _id_ [41-42]: Path: Path _id_ [41-42] (Ident _id_ [41-42] "y")"#]],
    );
}

#[test]
fn function_body_missing_semi_between_stmts() {
    check(
        parse,
        "function Foo() : () { f(x) g(y) }",
        &expect![[r#"
            Item _id_ [0-33]:
                Callable _id_ [0-33] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    input: Pat _id_ [12-14]: Unit
                    output: Type _id_ [17-19]: Unit
                    body: Block: Block _id_ [20-33]:
                        Stmt _id_ [22-26]: Expr: Expr _id_ [22-26]: Call:
                            Expr _id_ [22-23]: Path: Path _id_ [22-23] (Ident _id_ [22-23] "f")
                            Expr _id_ [23-26]: Paren: Expr _id_ [24-25]: Path: Path _id_ [24-25] (Ident _id_ [24-25] "x")
                        Stmt _id_ [27-31]: Expr: Expr _id_ [27-31]: Call:
                            Expr _id_ [27-28]: Path: Path _id_ [27-28] (Ident _id_ [27-28] "g")
                            Expr _id_ [28-31]: Paren: Expr _id_ [29-30]: Path: Path _id_ [29-30] (Ident _id_ [29-30] "y")

            [
                Error(
                    MissingSemi(
                        Span {
                            lo: 26,
                            hi: 26,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn operation_body_impl() {
    check(
        parse,
        "operation Foo() : Unit { body (...) { x } }",
        &expect![[r#"
            Item _id_ [0-43]:
                Callable _id_ [0-43] (Operation):
                    name: Ident _id_ [10-13] "Foo"
                    input: Pat _id_ [13-15]: Unit
                    output: Type _id_ [18-22]: Path: Path _id_ [18-22] (Ident _id_ [18-22] "Unit")
                    body: Specializations:
                        SpecDecl _id_ [25-41] (Body): Impl:
                            Pat _id_ [30-35]: Paren:
                                Pat _id_ [31-34]: Elided
                            Block _id_ [36-41]:
                                Stmt _id_ [38-39]: Expr: Expr _id_ [38-39]: Path: Path _id_ [38-39] (Ident _id_ [38-39] "x")"#]],
    );
}

#[test]
fn operation_body_ctl_impl() {
    check(
        parse,
        "operation Foo() : Unit { body (...) { x } controlled (cs, ...) { y } }",
        &expect![[r#"
            Item _id_ [0-70]:
                Callable _id_ [0-70] (Operation):
                    name: Ident _id_ [10-13] "Foo"
                    input: Pat _id_ [13-15]: Unit
                    output: Type _id_ [18-22]: Path: Path _id_ [18-22] (Ident _id_ [18-22] "Unit")
                    body: Specializations:
                        SpecDecl _id_ [25-41] (Body): Impl:
                            Pat _id_ [30-35]: Paren:
                                Pat _id_ [31-34]: Elided
                            Block _id_ [36-41]:
                                Stmt _id_ [38-39]: Expr: Expr _id_ [38-39]: Path: Path _id_ [38-39] (Ident _id_ [38-39] "x")
                        SpecDecl _id_ [42-68] (Ctl): Impl:
                            Pat _id_ [53-62]: Tuple:
                                Pat _id_ [54-56]: Bind:
                                    Ident _id_ [54-56] "cs"
                                Pat _id_ [58-61]: Elided
                            Block _id_ [63-68]:
                                Stmt _id_ [65-66]: Expr: Expr _id_ [65-66]: Path: Path _id_ [65-66] (Ident _id_ [65-66] "y")"#]],
    );
}

#[test]
fn operation_impl_and_gen() {
    check(
        parse,
        "operation Foo() : Unit { body (...) { x } adjoint self; }",
        &expect![[r#"
            Item _id_ [0-57]:
                Callable _id_ [0-57] (Operation):
                    name: Ident _id_ [10-13] "Foo"
                    input: Pat _id_ [13-15]: Unit
                    output: Type _id_ [18-22]: Path: Path _id_ [18-22] (Ident _id_ [18-22] "Unit")
                    body: Specializations:
                        SpecDecl _id_ [25-41] (Body): Impl:
                            Pat _id_ [30-35]: Paren:
                                Pat _id_ [31-34]: Elided
                            Block _id_ [36-41]:
                                Stmt _id_ [38-39]: Expr: Expr _id_ [38-39]: Path: Path _id_ [38-39] (Ident _id_ [38-39] "x")
                        SpecDecl _id_ [42-55] (Adj): Gen: Slf"#]],
    );
}

#[test]
fn operation_is_adj() {
    check(
        parse,
        "operation Foo() : Unit is Adj {}",
        &expect![[r#"
            Item _id_ [0-32]:
                Callable _id_ [0-32] (Operation):
                    name: Ident _id_ [10-13] "Foo"
                    input: Pat _id_ [13-15]: Unit
                    output: Type _id_ [18-22]: Path: Path _id_ [18-22] (Ident _id_ [18-22] "Unit")
                    functors: Functor Expr _id_ [26-29]: Adj
                    body: Block: Block _id_ [30-32]: <empty>"#]],
    );
}

#[test]
fn operation_is_adj_ctl() {
    check(
        parse,
        "operation Foo() : Unit is Adj + Ctl {}",
        &expect![[r#"
            Item _id_ [0-38]:
                Callable _id_ [0-38] (Operation):
                    name: Ident _id_ [10-13] "Foo"
                    input: Pat _id_ [13-15]: Unit
                    output: Type _id_ [18-22]: Path: Path _id_ [18-22] (Ident _id_ [18-22] "Unit")
                    functors: Functor Expr _id_ [26-35]: BinOp Union: (Functor Expr _id_ [26-29]: Adj) (Functor Expr _id_ [32-35]: Ctl)
                    body: Block: Block _id_ [36-38]: <empty>"#]],
    );
}

#[test]
fn function_missing_output_ty() {
    check(
        parse,
        "function Foo() { body intrinsic; }",
        &expect![[r#"
            Error(
                Token(
                    Colon,
                    Open(
                        Brace,
                    ),
                    Span {
                        lo: 15,
                        hi: 16,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn internal_ty() {
    check(
        parse,
        "internal newtype Foo = Unit;",
        &expect![[r#"
            Item _id_ [0-28]:
                Visibility _id_ [0-8] (Internal)
                New Type (Ident _id_ [17-20] "Foo"): TyDef _id_ [23-27]: Field:
                    Type _id_ [23-27]: Path: Path _id_ [23-27] (Ident _id_ [23-27] "Unit")"#]],
    );
}

#[test]
fn internal_function() {
    check(
        parse,
        "internal function Foo() : Unit {}",
        &expect![[r#"
            Item _id_ [0-33]:
                Visibility _id_ [0-8] (Internal)
                Callable _id_ [9-33] (Function):
                    name: Ident _id_ [18-21] "Foo"
                    input: Pat _id_ [21-23]: Unit
                    output: Type _id_ [26-30]: Path: Path _id_ [26-30] (Ident _id_ [26-30] "Unit")
                    body: Block: Block _id_ [31-33]: <empty>"#]],
    );
}

#[test]
fn internal_function_doc() {
    check(
        parse,
        "/// This is a
        /// doc comment.
        internal function Foo() : () {}",
        &expect![[r#"
            Item _id_ [0-78]:
                doc:
                    This is a
                    doc comment.
                Visibility _id_ [47-55] (Internal)
                Callable _id_ [56-78] (Function):
                    name: Ident _id_ [65-68] "Foo"
                    input: Pat _id_ [68-70]: Unit
                    output: Type _id_ [73-75]: Unit
                    body: Block: Block _id_ [76-78]: <empty>"#]],
    );
}

#[test]
fn internal_operation() {
    check(
        parse,
        "internal operation Foo() : Unit {}",
        &expect![[r#"
            Item _id_ [0-34]:
                Visibility _id_ [0-8] (Internal)
                Callable _id_ [9-34] (Operation):
                    name: Ident _id_ [19-22] "Foo"
                    input: Pat _id_ [22-24]: Unit
                    output: Type _id_ [27-31]: Path: Path _id_ [27-31] (Ident _id_ [27-31] "Unit")
                    body: Block: Block _id_ [32-34]: <empty>"#]],
    );
}

#[test]
fn attr_no_args() {
    check(
        parse_attr,
        "@Foo()",
        &expect![[r#"
            Attr _id_ [0-6] (Ident _id_ [1-4] "Foo"):
                Expr _id_ [4-6]: Unit"#]],
    );
}

#[test]
fn attr_single_arg() {
    check(
        parse_attr,
        "@Foo(123)",
        &expect![[r#"
            Attr _id_ [0-9] (Ident _id_ [1-4] "Foo"):
                Expr _id_ [4-9]: Paren: Expr _id_ [5-8]: Lit: Int(123)"#]],
    );
}

#[test]
fn attr_two_args() {
    check(
        parse_attr,
        "@Foo(123, \"bar\")",
        &expect![[r#"
            Attr _id_ [0-16] (Ident _id_ [1-4] "Foo"):
                Expr _id_ [4-16]: Tuple:
                    Expr _id_ [5-8]: Lit: Int(123)
                    Expr _id_ [10-15]: Lit: String("bar")"#]],
    );
}

#[test]
fn open_attr() {
    check(
        parse,
        "@Foo() open Bar;",
        &expect![[r#"
            Item _id_ [0-16]:
                Attr _id_ [0-6] (Ident _id_ [1-4] "Foo"):
                    Expr _id_ [4-6]: Unit
                Open (Ident _id_ [12-15] "Bar")"#]],
    );
}

#[test]
fn newtype_attr() {
    check(
        parse,
        "@Foo() newtype Bar = Unit;",
        &expect![[r#"
            Item _id_ [0-26]:
                Attr _id_ [0-6] (Ident _id_ [1-4] "Foo"):
                    Expr _id_ [4-6]: Unit
                New Type (Ident _id_ [15-18] "Bar"): TyDef _id_ [21-25]: Field:
                    Type _id_ [21-25]: Path: Path _id_ [21-25] (Ident _id_ [21-25] "Unit")"#]],
    );
}

#[test]
fn operation_one_attr() {
    check(
        parse,
        "@Foo() operation Bar() : Unit {}",
        &expect![[r#"
            Item _id_ [0-32]:
                Attr _id_ [0-6] (Ident _id_ [1-4] "Foo"):
                    Expr _id_ [4-6]: Unit
                Callable _id_ [7-32] (Operation):
                    name: Ident _id_ [17-20] "Bar"
                    input: Pat _id_ [20-22]: Unit
                    output: Type _id_ [25-29]: Path: Path _id_ [25-29] (Ident _id_ [25-29] "Unit")
                    body: Block: Block _id_ [30-32]: <empty>"#]],
    );
}

#[test]
fn operation_two_attrs() {
    check(
        parse,
        "@Foo() @Bar() operation Baz() : Unit {}",
        &expect![[r#"
            Item _id_ [0-39]:
                Attr _id_ [0-6] (Ident _id_ [1-4] "Foo"):
                    Expr _id_ [4-6]: Unit
                Attr _id_ [7-13] (Ident _id_ [8-11] "Bar"):
                    Expr _id_ [11-13]: Unit
                Callable _id_ [14-39] (Operation):
                    name: Ident _id_ [24-27] "Baz"
                    input: Pat _id_ [27-29]: Unit
                    output: Type _id_ [32-36]: Path: Path _id_ [32-36] (Ident _id_ [32-36] "Unit")
                    body: Block: Block _id_ [37-39]: <empty>"#]],
    );
}

#[test]
fn operation_attr_doc() {
    check(
        parse,
        "/// This is a
        /// doc comment.
        @Foo()
        operation Bar() : () {}",
        &expect![[r#"
            Item _id_ [0-85]:
                doc:
                    This is a
                    doc comment.
                Attr _id_ [47-53] (Ident _id_ [48-51] "Foo"):
                    Expr _id_ [51-53]: Unit
                Callable _id_ [62-85] (Operation):
                    name: Ident _id_ [72-75] "Bar"
                    input: Pat _id_ [75-77]: Unit
                    output: Type _id_ [80-82]: Unit
                    body: Block: Block _id_ [83-85]: <empty>"#]],
    );
}

#[test]
fn namespace_function() {
    check_vec(
        parse_namespaces,
        "namespace A { function Foo() : Unit { body intrinsic; } }",
        &expect![[r#"
            Namespace _id_ [0-57] (Ident _id_ [10-11] "A"):
                Item _id_ [14-55]:
                    Callable _id_ [14-55] (Function):
                        name: Ident _id_ [23-26] "Foo"
                        input: Pat _id_ [26-28]: Unit
                        output: Type _id_ [31-35]: Path: Path _id_ [31-35] (Ident _id_ [31-35] "Unit")
                        body: Specializations:
                            SpecDecl _id_ [38-53] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn namespace_doc() {
    check_vec(
        parse_namespaces,
        "/// This is a
        /// doc comment.
        namespace A {
            function Foo() : () {}
        }",
        &expect![[r#"
            Namespace _id_ [0-105] (Ident _id_ [57-58] "A"):
                doc:
                    This is a
                    doc comment.
                Item _id_ [73-95]:
                    Callable _id_ [73-95] (Function):
                        name: Ident _id_ [82-85] "Foo"
                        input: Pat _id_ [85-87]: Unit
                        output: Type _id_ [90-92]: Unit
                        body: Block: Block _id_ [93-95]: <empty>"#]],
    );
}

#[test]
fn floating_doc_comments_in_namespace() {
    check_vec(
        parse_namespaces,
        "namespace MyQuantumProgram {
    @EntryPoint()
    function Main() : Unit {}
    /// hi
    /// another doc comment
}
",
        &expect![[r#"
            Namespace _id_ [0-117] (Ident _id_ [10-26] "MyQuantumProgram"):
                Item _id_ [33-76]:
                    Attr _id_ [33-46] (Ident _id_ [34-44] "EntryPoint"):
                        Expr _id_ [44-46]: Unit
                    Callable _id_ [51-76] (Function):
                        name: Ident _id_ [60-64] "Main"
                        input: Pat _id_ [64-66]: Unit
                        output: Type _id_ [69-73]: Path: Path _id_ [69-73] (Ident _id_ [69-73] "Unit")
                        body: Block: Block _id_ [74-76]: <empty>
                Item _id_ [81-115]:
                    Err

            [
                Error(
                    FloatingDocComment(
                        Span {
                            lo: 81,
                            hi: 115,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn two_namespaces() {
    check_vec(
        parse_namespaces,
        "namespace A {} namespace B {}",
        &expect![[r#"
            Namespace _id_ [0-14] (Ident _id_ [10-11] "A"):,
            Namespace _id_ [15-29] (Ident _id_ [25-26] "B"):"#]],
    );
}

#[test]
fn two_namespaces_docs() {
    check_vec(
        parse_namespaces,
        "/// This is the first namespace.
        namespace A {}
        /// This is the second namespace.
        namespace B {}",
        &expect![[r#"
            Namespace _id_ [0-55] (Ident _id_ [51-52] "A"):
                doc:
                    This is the first namespace.,
            Namespace _id_ [64-120] (Ident _id_ [116-117] "B"):
                doc:
                    This is the second namespace."#]],
    );
}

#[test]
fn two_open_items() {
    check_vec(
        parse_namespaces,
        "namespace A { open B; open C; }",
        &expect![[r#"
            Namespace _id_ [0-31] (Ident _id_ [10-11] "A"):
                Item _id_ [14-21]:
                    Open (Ident _id_ [19-20] "B")
                Item _id_ [22-29]:
                    Open (Ident _id_ [27-28] "C")"#]],
    );
}

#[test]
fn two_ty_items() {
    check_vec(
        parse_namespaces,
        "namespace A { newtype B = Unit; newtype C = Unit; }",
        &expect![[r#"
            Namespace _id_ [0-51] (Ident _id_ [10-11] "A"):
                Item _id_ [14-31]:
                    New Type (Ident _id_ [22-23] "B"): TyDef _id_ [26-30]: Field:
                        Type _id_ [26-30]: Path: Path _id_ [26-30] (Ident _id_ [26-30] "Unit")
                Item _id_ [32-49]:
                    New Type (Ident _id_ [40-41] "C"): TyDef _id_ [44-48]: Field:
                        Type _id_ [44-48]: Path: Path _id_ [44-48] (Ident _id_ [44-48] "Unit")"#]],
    );
}

#[test]
fn two_callable_items() {
    check_vec(
        parse_namespaces,
        "namespace A { operation B() : Unit {} function C() : Unit {} }",
        &expect![[r#"
            Namespace _id_ [0-62] (Ident _id_ [10-11] "A"):
                Item _id_ [14-37]:
                    Callable _id_ [14-37] (Operation):
                        name: Ident _id_ [24-25] "B"
                        input: Pat _id_ [25-27]: Unit
                        output: Type _id_ [30-34]: Path: Path _id_ [30-34] (Ident _id_ [30-34] "Unit")
                        body: Block: Block _id_ [35-37]: <empty>
                Item _id_ [38-60]:
                    Callable _id_ [38-60] (Function):
                        name: Ident _id_ [47-48] "C"
                        input: Pat _id_ [48-50]: Unit
                        output: Type _id_ [53-57]: Path: Path _id_ [53-57] (Ident _id_ [53-57] "Unit")
                        body: Block: Block _id_ [58-60]: <empty>"#]],
    );
}

#[test]
fn two_callable_items_docs() {
    check_vec(
        parse_namespaces,
        "namespace A {
            /// This is the first callable.
            function Foo() : () {}
            /// This is the second callable.
            operation Foo() : () {}
        }",
        &expect![[r#"
            Namespace _id_ [0-183] (Ident _id_ [10-11] "A"):
                Item _id_ [26-92]:
                    doc:
                        This is the first callable.
                    Callable _id_ [70-92] (Function):
                        name: Ident _id_ [79-82] "Foo"
                        input: Pat _id_ [82-84]: Unit
                        output: Type _id_ [87-89]: Unit
                        body: Block: Block _id_ [90-92]: <empty>
                Item _id_ [105-173]:
                    doc:
                        This is the second callable.
                    Callable _id_ [150-173] (Operation):
                        name: Ident _id_ [160-163] "Foo"
                        input: Pat _id_ [163-165]: Unit
                        output: Type _id_ [168-170]: Unit
                        body: Block: Block _id_ [171-173]: <empty>"#]],
    );
}

#[test]
fn doc_without_item() {
    check_vec(
        parse_namespaces,
        "namespace A {
            /// This is a doc comment.
        }",
        &expect![[r#"
            Namespace _id_ [0-62] (Ident _id_ [10-11] "A"):
                Item _id_ [26-52]:
                    Err

            [
                Error(
                    FloatingDocComment(
                        Span {
                            lo: 26,
                            hi: 52,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn recover_callable_item() {
    check_vec(
        parse_namespaces,
        "namespace A {
            function Foo() : Int { 5 }
            function Bar() { 10 }
            operation Baz() : Double { 2.0 }
        }",
        &expect![[r#"
            Namespace _id_ [0-141] (Ident _id_ [10-11] "A"):
                Item _id_ [26-52]:
                    Callable _id_ [26-52] (Function):
                        name: Ident _id_ [35-38] "Foo"
                        input: Pat _id_ [38-40]: Unit
                        output: Type _id_ [43-46]: Path: Path _id_ [43-46] (Ident _id_ [43-46] "Int")
                        body: Block: Block _id_ [47-52]:
                            Stmt _id_ [49-50]: Expr: Expr _id_ [49-50]: Lit: Int(5)
                Item _id_ [65-86]:
                    Err
                Item _id_ [99-131]:
                    Callable _id_ [99-131] (Operation):
                        name: Ident _id_ [109-112] "Baz"
                        input: Pat _id_ [112-114]: Unit
                        output: Type _id_ [117-123]: Path: Path _id_ [117-123] (Ident _id_ [117-123] "Double")
                        body: Block: Block _id_ [124-131]:
                            Stmt _id_ [126-129]: Expr: Expr _id_ [126-129]: Lit: Double(2)

            [
                Error(
                    Token(
                        Colon,
                        Open(
                            Brace,
                        ),
                        Span {
                            lo: 80,
                            hi: 81,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn recover_unclosed_callable_item() {
    check_vec(
        parse_namespaces,
        "namespace A {
            function Foo() : Int {",
        &expect![[r#"
            Namespace _id_ [0-48] (Ident _id_ [10-11] "A"):
                Item _id_ [26-48]:
                    Callable _id_ [26-48] (Function):
                        name: Ident _id_ [35-38] "Foo"
                        input: Pat _id_ [38-40]: Unit
                        output: Type _id_ [43-46]: Path: Path _id_ [43-46] (Ident _id_ [43-46] "Int")
                        body: Block: Block _id_ [47-48]: <empty>

            [
                Error(
                    Token(
                        Close(
                            Brace,
                        ),
                        Eof,
                        Span {
                            lo: 48,
                            hi: 48,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn recover_unclosed_namespace() {
    check_vec(
        parse_namespaces,
        "namespace A {
            function Foo() : Int { 2 }",
        &expect![[r#"
            Namespace _id_ [0-52] (Ident _id_ [10-11] "A"):
                Item _id_ [26-52]:
                    Callable _id_ [26-52] (Function):
                        name: Ident _id_ [35-38] "Foo"
                        input: Pat _id_ [38-40]: Unit
                        output: Type _id_ [43-46]: Path: Path _id_ [43-46] (Ident _id_ [43-46] "Int")
                        body: Block: Block _id_ [47-52]:
                            Stmt _id_ [49-50]: Expr: Expr _id_ [49-50]: Lit: Int(2)

            [
                Error(
                    Token(
                        Close(
                            Brace,
                        ),
                        Eof,
                        Span {
                            lo: 52,
                            hi: 52,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn callable_missing_parens() {
    check_vec(
        parse_namespaces,
        "namespace A {
        function Foo x : Int : Int { x }
        }",
        &expect![[r#"
            Namespace _id_ [0-64] (Ident _id_ [10-11] "A"):
                Item _id_ [22-54]:
                    Err

            [
                Error(
                    MissingParens(
                        Span {
                            lo: 35,
                            hi: 42,
                        },
                    ),
                ),
            ]"#]],
    )
}
#[test]
fn callable_missing_close_parens() {
    check_vec(
        parse_namespaces,
        "namespace A {
        function Foo (x : Int : Int { x }
        }",
        &expect![[r#"
            Namespace _id_ [0-65] (Ident _id_ [10-11] "A"):
                Item _id_ [22-55]:
                    Err

            [
                Error(
                    Token(
                        Close(
                            Paren,
                        ),
                        Colon,
                        Span {
                            lo: 44,
                            hi: 45,
                        },
                    ),
                ),
            ]"#]],
    )
}
#[test]
fn callable_missing_open_parens() {
    check_vec(
        parse_namespaces,
        "namespace A {
        function Foo x : Int) : Int { x }
        }",
        &expect![[r#"
            Namespace _id_ [0-65] (Ident _id_ [10-11] "A"):
                Item _id_ [22-55]:
                    Err

            [
                Error(
                    MissingParens(
                        Span {
                            lo: 35,
                            hi: 42,
                        },
                    ),
                ),
            ]"#]],
    )
}
