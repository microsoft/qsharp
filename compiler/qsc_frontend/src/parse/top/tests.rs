// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{attr, item, namespaces, spec_decl};
use crate::parse::tests::{check, check_vec};
use expect_test::expect;

#[test]
fn body_intrinsic() {
    check(
        spec_decl,
        "body intrinsic;",
        &expect!["SpecDecl _id_ [0-15] (Body): Gen: Intrinsic"],
    );
}

#[test]
fn adjoint_self() {
    check(
        spec_decl,
        "adjoint self;",
        &expect!["SpecDecl _id_ [0-13] (Adj): Gen: Slf"],
    );
}

#[test]
fn adjoint_invert() {
    check(
        spec_decl,
        "adjoint invert;",
        &expect!["SpecDecl _id_ [0-15] (Adj): Gen: Invert"],
    );
}

#[test]
fn controlled_distribute() {
    check(
        spec_decl,
        "controlled distribute;",
        &expect!["SpecDecl _id_ [0-22] (Ctl): Gen: Distribute"],
    );
}

#[test]
fn controlled_adjoint_auto() {
    check(
        spec_decl,
        "controlled adjoint auto;",
        &expect!["SpecDecl _id_ [0-24] (CtlAdj): Gen: Auto"],
    );
}

#[test]
fn spec_gen_missing_semi() {
    check(
        spec_decl,
        "body intrinsic",
        &expect![[r#"
            Err(
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
        spec_decl,
        "adjoint foo;",
        &expect![[r#"
            Err(
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
        item,
        "open Foo.Bar.Baz;",
        &expect![[r#"
            Item _id_ [0-17]:
                Open (Ident _id_ [5-16] "Foo.Bar.Baz")"#]],
    );
}

#[test]
fn open_alias() {
    check(
        item,
        "open Foo.Bar.Baz as Baz;",
        &expect![[r#"
            Item _id_ [0-24]:
                Open (Ident _id_ [5-16] "Foo.Bar.Baz") (Ident _id_ [20-23] "Baz")"#]],
    );
}

#[test]
fn open_alias_dot() {
    check(
        item,
        "open Foo.Bar.Baz as Bar.Baz;",
        &expect![[r#"
            Item _id_ [0-28]:
                Open (Ident _id_ [5-16] "Foo.Bar.Baz") (Ident _id_ [20-27] "Bar.Baz")"#]],
    );
}

#[test]
fn ty_decl() {
    check(
        item,
        "newtype Foo = Unit;",
        &expect![[r#"
            Item _id_ [0-19]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-18]: Field:
                    Type _id_ [14-18]: Unit"#]],
    );
}

#[test]
fn ty_decl_field_name() {
    check(
        item,
        "newtype Foo = Bar : Int;",
        &expect![[r#"
            Item _id_ [0-24]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-23]: Field:
                    Ident _id_ [14-17] "Bar"
                    Type _id_ [20-23]: Prim (Int)"#]],
    );
}

#[test]
fn ty_def_invalid_field_name() {
    check(
        item,
        "newtype Foo = Bar.Baz : Int[];",
        &expect![[r#"
            Err(
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
        item,
        "newtype Foo = (Int, Int);",
        &expect![[r#"
            Item _id_ [0-25]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-24]: Tuple:
                    TyDef _id_ [15-18]: Field:
                        Type _id_ [15-18]: Prim (Int)
                    TyDef _id_ [20-23]: Field:
                        Type _id_ [20-23]: Prim (Int)"#]],
    );
}

#[test]
fn ty_def_tuple_one_named() {
    check(
        item,
        "newtype Foo = (X : Int, Int);",
        &expect![[r#"
            Item _id_ [0-29]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-28]: Tuple:
                    TyDef _id_ [15-22]: Field:
                        Ident _id_ [15-16] "X"
                        Type _id_ [19-22]: Prim (Int)
                    TyDef _id_ [24-27]: Field:
                        Type _id_ [24-27]: Prim (Int)"#]],
    );
}

#[test]
fn ty_def_tuple_both_named() {
    check(
        item,
        "newtype Foo = (X : Int, Y : Int);",
        &expect![[r#"
            Item _id_ [0-33]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-32]: Tuple:
                    TyDef _id_ [15-22]: Field:
                        Ident _id_ [15-16] "X"
                        Type _id_ [19-22]: Prim (Int)
                    TyDef _id_ [24-31]: Field:
                        Ident _id_ [24-25] "Y"
                        Type _id_ [28-31]: Prim (Int)"#]],
    );
}

#[test]
fn ty_def_nested_tuple() {
    check(
        item,
        "newtype Foo = ((X : Int, Y : Int), Z : Int);",
        &expect![[r#"
            Item _id_ [0-44]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-43]: Tuple:
                    TyDef _id_ [15-33]: Tuple:
                        TyDef _id_ [16-23]: Field:
                            Ident _id_ [16-17] "X"
                            Type _id_ [20-23]: Prim (Int)
                        TyDef _id_ [25-32]: Field:
                            Ident _id_ [25-26] "Y"
                            Type _id_ [29-32]: Prim (Int)
                    TyDef _id_ [35-42]: Field:
                        Ident _id_ [35-36] "Z"
                        Type _id_ [39-42]: Prim (Int)"#]],
    );
}

#[test]
fn ty_def_tuple_with_name() {
    check(
        item,
        "newtype Foo = Pair : (Int, Int);",
        &expect![[r#"
            Item _id_ [0-32]:
                New Type (Ident _id_ [8-11] "Foo"): TyDef _id_ [14-31]: Field:
                    Ident _id_ [14-18] "Pair"
                    Type _id_ [21-31]: Tuple:
                        Type _id_ [22-25]: Prim (Int)
                        Type _id_ [27-30]: Prim (Int)"#]],
    );
}

#[test]
fn function_decl() {
    check(
        item,
        "function Foo() : Unit { body intrinsic; }",
        &expect![[r#"
            Item _id_ [0-41]:
                Callable _id_ [0-41] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    input: Pat _id_ [12-14]: Unit
                    output: Type _id_ [17-21]: Unit
                    body: Specializations:
                        SpecDecl _id_ [24-39] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn operation_decl() {
    check(
        item,
        "operation Foo() : Unit { body intrinsic; }",
        &expect![[r#"
            Item _id_ [0-42]:
                Callable _id_ [0-42] (Operation):
                    name: Ident _id_ [10-13] "Foo"
                    input: Pat _id_ [13-15]: Unit
                    output: Type _id_ [18-22]: Unit
                    body: Specializations:
                        SpecDecl _id_ [25-40] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_one_param() {
    check(
        item,
        "function Foo(x : Int) : Unit { body intrinsic; }",
        &expect![[r#"
            Item _id_ [0-48]:
                Callable _id_ [0-48] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    input: Pat _id_ [12-21]: Paren:
                        Pat _id_ [13-20]: Bind:
                            Ident _id_ [13-14] "x"
                            Type _id_ [17-20]: Prim (Int)
                    output: Type _id_ [24-28]: Unit
                    body: Specializations:
                        SpecDecl _id_ [31-46] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_two_params() {
    check(
        item,
        "function Foo(x : Int, y : Int) : Unit { body intrinsic; }",
        &expect![[r#"
            Item _id_ [0-57]:
                Callable _id_ [0-57] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    input: Pat _id_ [12-30]: Tuple:
                        Pat _id_ [13-20]: Bind:
                            Ident _id_ [13-14] "x"
                            Type _id_ [17-20]: Prim (Int)
                        Pat _id_ [22-29]: Bind:
                            Ident _id_ [22-23] "y"
                            Type _id_ [26-29]: Prim (Int)
                    output: Type _id_ [33-37]: Unit
                    body: Specializations:
                        SpecDecl _id_ [40-55] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_one_ty_param() {
    check(
        item,
        "function Foo<'T>() : Unit { body intrinsic; }",
        &expect![[r#"
            Item _id_ [0-45]:
                Callable _id_ [0-45] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    type params:
                        Ident _id_ [14-15] "T"
                    input: Pat _id_ [16-18]: Unit
                    output: Type _id_ [21-25]: Unit
                    body: Specializations:
                        SpecDecl _id_ [28-43] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_two_ty_params() {
    check(
        item,
        "function Foo<'T, 'U>() : Unit { body intrinsic; }",
        &expect![[r#"
            Item _id_ [0-49]:
                Callable _id_ [0-49] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    type params:
                        Ident _id_ [14-15] "T"
                        Ident _id_ [18-19] "U"
                    input: Pat _id_ [20-22]: Unit
                    output: Type _id_ [25-29]: Unit
                    body: Specializations:
                        SpecDecl _id_ [32-47] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_single_impl() {
    check(
        item,
        "function Foo(x : Int) : Int { let y = x; y }",
        &expect![[r#"
            Item _id_ [0-44]:
                Callable _id_ [0-44] (Function):
                    name: Ident _id_ [9-12] "Foo"
                    input: Pat _id_ [12-21]: Paren:
                        Pat _id_ [13-20]: Bind:
                            Ident _id_ [13-14] "x"
                            Type _id_ [17-20]: Prim (Int)
                    output: Type _id_ [24-27]: Prim (Int)
                    body: Block: Block _id_ [28-44]:
                        Stmt _id_ [30-40]: Local (Immutable):
                            Pat _id_ [34-35]: Bind:
                                Ident _id_ [34-35] "y"
                            Expr _id_ [38-39]: Path: Path _id_ [38-39] (Ident _id_ [38-39] "x")
                        Stmt _id_ [41-42]: Expr: Expr _id_ [41-42]: Path: Path _id_ [41-42] (Ident _id_ [41-42] "y")"#]],
    );
}

#[test]
fn operation_body_impl() {
    check(
        item,
        "operation Foo() : Unit { body (...) { x } }",
        &expect![[r#"
            Item _id_ [0-43]:
                Callable _id_ [0-43] (Operation):
                    name: Ident _id_ [10-13] "Foo"
                    input: Pat _id_ [13-15]: Unit
                    output: Type _id_ [18-22]: Unit
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
        item,
        "operation Foo() : Unit { body (...) { x } controlled (cs, ...) { y } }",
        &expect![[r#"
            Item _id_ [0-70]:
                Callable _id_ [0-70] (Operation):
                    name: Ident _id_ [10-13] "Foo"
                    input: Pat _id_ [13-15]: Unit
                    output: Type _id_ [18-22]: Unit
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
        item,
        "operation Foo() : Unit { body (...) { x } adjoint self; }",
        &expect![[r#"
            Item _id_ [0-57]:
                Callable _id_ [0-57] (Operation):
                    name: Ident _id_ [10-13] "Foo"
                    input: Pat _id_ [13-15]: Unit
                    output: Type _id_ [18-22]: Unit
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
        item,
        "operation Foo() : Unit is Adj {}",
        &expect![[r#"
            Item _id_ [0-32]:
                Callable _id_ [0-32] (Operation):
                    name: Ident _id_ [10-13] "Foo"
                    input: Pat _id_ [13-15]: Unit
                    output: Type _id_ [18-22]: Unit
                    functors: Functor Expr _id_ [26-29]: Adj
                    body: Block: Block _id_ [30-32]: <empty>"#]],
    );
}

#[test]
fn operation_is_adj_ctl() {
    check(
        item,
        "operation Foo() : Unit is Adj + Ctl {}",
        &expect![[r#"
            Item _id_ [0-38]:
                Callable _id_ [0-38] (Operation):
                    name: Ident _id_ [10-13] "Foo"
                    input: Pat _id_ [13-15]: Unit
                    output: Type _id_ [18-22]: Unit
                    functors: Functor Expr _id_ [26-35]: BinOp Union: (Functor Expr _id_ [26-29]: Adj) (Functor Expr _id_ [32-35]: Ctl)
                    body: Block: Block _id_ [36-38]: <empty>"#]],
    );
}

#[test]
fn function_missing_output_ty() {
    check(
        item,
        "function Foo() { body intrinsic; }",
        &expect![[r#"
            Err(
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
        item,
        "internal newtype Foo = Unit;",
        &expect![[r#"
            Item _id_ [0-28]:
                Visibility _id_ [0-8] (Internal)
                New Type (Ident _id_ [17-20] "Foo"): TyDef _id_ [23-27]: Field:
                    Type _id_ [23-27]: Unit"#]],
    );
}

#[test]
fn internal_function() {
    check(
        item,
        "internal function Foo() : Unit {}",
        &expect![[r#"
            Item _id_ [0-33]:
                Visibility _id_ [0-8] (Internal)
                Callable _id_ [9-33] (Function):
                    name: Ident _id_ [18-21] "Foo"
                    input: Pat _id_ [21-23]: Unit
                    output: Type _id_ [26-30]: Unit
                    body: Block: Block _id_ [31-33]: <empty>"#]],
    );
}

#[test]
fn internal_operation() {
    check(
        item,
        "internal operation Foo() : Unit {}",
        &expect![[r#"
            Item _id_ [0-34]:
                Visibility _id_ [0-8] (Internal)
                Callable _id_ [9-34] (Operation):
                    name: Ident _id_ [19-22] "Foo"
                    input: Pat _id_ [22-24]: Unit
                    output: Type _id_ [27-31]: Unit
                    body: Block: Block _id_ [32-34]: <empty>"#]],
    );
}

#[test]
fn attr_no_args() {
    check(
        attr,
        "@Foo()",
        &expect![[r#"
            Attr _id_ [0-6] (Ident _id_ [1-4] "Foo"):
                Expr _id_ [4-6]: Unit"#]],
    );
}

#[test]
fn attr_single_arg() {
    check(
        attr,
        "@Foo(123)",
        &expect![[r#"
            Attr _id_ [0-9] (Ident _id_ [1-4] "Foo"):
                Expr _id_ [4-9]: Paren: Expr _id_ [5-8]: Lit: Int(123)"#]],
    );
}

#[test]
fn attr_two_args() {
    check(
        attr,
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
        item,
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
        item,
        "@Foo() newtype Bar = Unit;",
        &expect![[r#"
            Item _id_ [0-26]:
                Attr _id_ [0-6] (Ident _id_ [1-4] "Foo"):
                    Expr _id_ [4-6]: Unit
                New Type (Ident _id_ [15-18] "Bar"): TyDef _id_ [21-25]: Field:
                    Type _id_ [21-25]: Unit"#]],
    );
}

#[test]
fn operation_one_attr() {
    check(
        item,
        "@Foo() operation Bar() : Unit {}",
        &expect![[r#"
            Item _id_ [0-32]:
                Attr _id_ [0-6] (Ident _id_ [1-4] "Foo"):
                    Expr _id_ [4-6]: Unit
                Callable _id_ [7-32] (Operation):
                    name: Ident _id_ [17-20] "Bar"
                    input: Pat _id_ [20-22]: Unit
                    output: Type _id_ [25-29]: Unit
                    body: Block: Block _id_ [30-32]: <empty>"#]],
    );
}

#[test]
fn operation_two_attrs() {
    check(
        item,
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
                    output: Type _id_ [32-36]: Unit
                    body: Block: Block _id_ [37-39]: <empty>"#]],
    );
}

#[test]
fn namespace_function() {
    check_vec(
        namespaces,
        "namespace A { function Foo() : Unit { body intrinsic; } }",
        &expect![[r#"
            Namespace _id_ [0-57] (Ident _id_ [10-11] "A"):
                Item _id_ [14-55]:
                    Callable _id_ [14-55] (Function):
                        name: Ident _id_ [23-26] "Foo"
                        input: Pat _id_ [26-28]: Unit
                        output: Type _id_ [31-35]: Unit
                        body: Specializations:
                            SpecDecl _id_ [38-53] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn two_namespaces() {
    check_vec(
        namespaces,
        "namespace A {} namespace B {}",
        &expect![[r#"
            Namespace _id_ [0-14] (Ident _id_ [10-11] "A"):,
            Namespace _id_ [15-29] (Ident _id_ [25-26] "B"):"#]],
    );
}

#[test]
fn two_open_items() {
    check_vec(
        namespaces,
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
        namespaces,
        "namespace A { newtype B = Unit; newtype C = Unit; }",
        &expect![[r#"
            Namespace _id_ [0-51] (Ident _id_ [10-11] "A"):
                Item _id_ [14-31]:
                    New Type (Ident _id_ [22-23] "B"): TyDef _id_ [26-30]: Field:
                        Type _id_ [26-30]: Unit
                Item _id_ [32-49]:
                    New Type (Ident _id_ [40-41] "C"): TyDef _id_ [44-48]: Field:
                        Type _id_ [44-48]: Unit"#]],
    );
}

#[test]
fn two_callable_items() {
    check_vec(
        namespaces,
        "namespace A { operation B() : Unit {} function C() : Unit {} }",
        &expect![[r#"
            Namespace _id_ [0-62] (Ident _id_ [10-11] "A"):
                Item _id_ [14-37]:
                    Callable _id_ [14-37] (Operation):
                        name: Ident _id_ [24-25] "B"
                        input: Pat _id_ [25-27]: Unit
                        output: Type _id_ [30-34]: Unit
                        body: Block: Block _id_ [35-37]: <empty>
                Item _id_ [38-60]:
                    Callable _id_ [38-60] (Function):
                        name: Ident _id_ [47-48] "C"
                        input: Pat _id_ [48-50]: Unit
                        output: Type _id_ [53-57]: Unit
                        body: Block: Block _id_ [58-60]: <empty>"#]],
    );
}
