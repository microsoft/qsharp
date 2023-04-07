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
        &expect!["SpecDecl 4294967295 [0-15] (Body): Gen: Intrinsic"],
    );
}

#[test]
fn adjoint_self() {
    check(
        spec_decl,
        "adjoint self;",
        &expect!["SpecDecl 4294967295 [0-13] (Adj): Gen: Slf"],
    );
}

#[test]
fn adjoint_invert() {
    check(
        spec_decl,
        "adjoint invert;",
        &expect!["SpecDecl 4294967295 [0-15] (Adj): Gen: Invert"],
    );
}

#[test]
fn controlled_distribute() {
    check(
        spec_decl,
        "controlled distribute;",
        &expect!["SpecDecl 4294967295 [0-22] (Ctl): Gen: Distribute"],
    );
}

#[test]
fn controlled_adjoint_auto() {
    check(
        spec_decl,
        "controlled adjoint auto;",
        &expect!["SpecDecl 4294967295 [0-24] (CtlAdj): Gen: Auto"],
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
            Item 4294967295 [0-17]:
                Open (Ident 4294967295 [5-16] "Foo.Bar.Baz")"#]],
    );
}

#[test]
fn open_alias() {
    check(
        item,
        "open Foo.Bar.Baz as Baz;",
        &expect![[r#"
            Item 4294967295 [0-24]:
                Open (Ident 4294967295 [5-16] "Foo.Bar.Baz") (Ident 4294967295 [20-23] "Baz")"#]],
    );
}

#[test]
fn open_alias_dot() {
    check(
        item,
        "open Foo.Bar.Baz as Bar.Baz;",
        &expect![[r#"
            Item 4294967295 [0-28]:
                Open (Ident 4294967295 [5-16] "Foo.Bar.Baz") (Ident 4294967295 [20-27] "Bar.Baz")"#]],
    );
}

#[test]
fn ty_decl() {
    check(
        item,
        "newtype Foo = Unit;",
        &expect![[r#"
            Item 4294967295 [0-19]:
                New Type (Ident 4294967295 [8-11] "Foo"): TyDef 4294967295 [14-18]: Field:
                    Type 4294967295 [14-18]: Unit"#]],
    );
}

#[test]
fn ty_decl_field_name() {
    check(
        item,
        "newtype Foo = Bar : Int;",
        &expect![[r#"
            Item 4294967295 [0-24]:
                New Type (Ident 4294967295 [8-11] "Foo"): TyDef 4294967295 [14-23]: Field:
                    Ident 4294967295 [14-17] "Bar"
                    Type 4294967295 [20-23]: Prim (Int)"#]],
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
            Item 4294967295 [0-25]:
                New Type (Ident 4294967295 [8-11] "Foo"): TyDef 4294967295 [14-24]: Tuple:
                    TyDef 4294967295 [15-18]: Field:
                        Type 4294967295 [15-18]: Prim (Int)
                    TyDef 4294967295 [20-23]: Field:
                        Type 4294967295 [20-23]: Prim (Int)"#]],
    );
}

#[test]
fn ty_def_tuple_one_named() {
    check(
        item,
        "newtype Foo = (X : Int, Int);",
        &expect![[r#"
            Item 4294967295 [0-29]:
                New Type (Ident 4294967295 [8-11] "Foo"): TyDef 4294967295 [14-28]: Tuple:
                    TyDef 4294967295 [15-22]: Field:
                        Ident 4294967295 [15-16] "X"
                        Type 4294967295 [19-22]: Prim (Int)
                    TyDef 4294967295 [24-27]: Field:
                        Type 4294967295 [24-27]: Prim (Int)"#]],
    );
}

#[test]
fn ty_def_tuple_both_named() {
    check(
        item,
        "newtype Foo = (X : Int, Y : Int);",
        &expect![[r#"
            Item 4294967295 [0-33]:
                New Type (Ident 4294967295 [8-11] "Foo"): TyDef 4294967295 [14-32]: Tuple:
                    TyDef 4294967295 [15-22]: Field:
                        Ident 4294967295 [15-16] "X"
                        Type 4294967295 [19-22]: Prim (Int)
                    TyDef 4294967295 [24-31]: Field:
                        Ident 4294967295 [24-25] "Y"
                        Type 4294967295 [28-31]: Prim (Int)"#]],
    );
}

#[test]
fn ty_def_nested_tuple() {
    check(
        item,
        "newtype Foo = ((X : Int, Y : Int), Z : Int);",
        &expect![[r#"
            Item 4294967295 [0-44]:
                New Type (Ident 4294967295 [8-11] "Foo"): TyDef 4294967295 [14-43]: Tuple:
                    TyDef 4294967295 [15-33]: Tuple:
                        TyDef 4294967295 [16-23]: Field:
                            Ident 4294967295 [16-17] "X"
                            Type 4294967295 [20-23]: Prim (Int)
                        TyDef 4294967295 [25-32]: Field:
                            Ident 4294967295 [25-26] "Y"
                            Type 4294967295 [29-32]: Prim (Int)
                    TyDef 4294967295 [35-42]: Field:
                        Ident 4294967295 [35-36] "Z"
                        Type 4294967295 [39-42]: Prim (Int)"#]],
    );
}

#[test]
fn ty_def_tuple_with_name() {
    check(
        item,
        "newtype Foo = Pair : (Int, Int);",
        &expect![[r#"
            Item 4294967295 [0-32]:
                New Type (Ident 4294967295 [8-11] "Foo"): TyDef 4294967295 [14-31]: Field:
                    Ident 4294967295 [14-18] "Pair"
                    Type 4294967295 [21-31]: Tuple:
                        Type 4294967295 [22-25]: Prim (Int)
                        Type 4294967295 [27-30]: Prim (Int)"#]],
    );
}

#[test]
fn function_decl() {
    check(
        item,
        "function Foo() : Unit { body intrinsic; }",
        &expect![[r#"
            Item 4294967295 [0-41]:
                Callable 4294967295 [0-41] (Function):
                    name: Ident 4294967295 [9-12] "Foo"
                    input: Pat 4294967295 [12-14]: Unit
                    output: Type 4294967295 [17-21]: Unit
                    body: Specializations:
                        SpecDecl 4294967295 [24-39] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn operation_decl() {
    check(
        item,
        "operation Foo() : Unit { body intrinsic; }",
        &expect![[r#"
            Item 4294967295 [0-42]:
                Callable 4294967295 [0-42] (Operation):
                    name: Ident 4294967295 [10-13] "Foo"
                    input: Pat 4294967295 [13-15]: Unit
                    output: Type 4294967295 [18-22]: Unit
                    body: Specializations:
                        SpecDecl 4294967295 [25-40] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_one_param() {
    check(
        item,
        "function Foo(x : Int) : Unit { body intrinsic; }",
        &expect![[r#"
            Item 4294967295 [0-48]:
                Callable 4294967295 [0-48] (Function):
                    name: Ident 4294967295 [9-12] "Foo"
                    input: Pat 4294967295 [12-21]: Paren:
                        Pat 4294967295 [13-20]: Bind:
                            Ident 4294967295 [13-14] "x"
                            Type 4294967295 [17-20]: Prim (Int)
                    output: Type 4294967295 [24-28]: Unit
                    body: Specializations:
                        SpecDecl 4294967295 [31-46] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_two_params() {
    check(
        item,
        "function Foo(x : Int, y : Int) : Unit { body intrinsic; }",
        &expect![[r#"
            Item 4294967295 [0-57]:
                Callable 4294967295 [0-57] (Function):
                    name: Ident 4294967295 [9-12] "Foo"
                    input: Pat 4294967295 [12-30]: Tuple:
                        Pat 4294967295 [13-20]: Bind:
                            Ident 4294967295 [13-14] "x"
                            Type 4294967295 [17-20]: Prim (Int)
                        Pat 4294967295 [22-29]: Bind:
                            Ident 4294967295 [22-23] "y"
                            Type 4294967295 [26-29]: Prim (Int)
                    output: Type 4294967295 [33-37]: Unit
                    body: Specializations:
                        SpecDecl 4294967295 [40-55] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_one_ty_param() {
    check(
        item,
        "function Foo<'T>() : Unit { body intrinsic; }",
        &expect![[r#"
            Item 4294967295 [0-45]:
                Callable 4294967295 [0-45] (Function):
                    name: Ident 4294967295 [9-12] "Foo"
                    type params:
                        Ident 4294967295 [14-15] "T"
                    input: Pat 4294967295 [16-18]: Unit
                    output: Type 4294967295 [21-25]: Unit
                    body: Specializations:
                        SpecDecl 4294967295 [28-43] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_two_ty_params() {
    check(
        item,
        "function Foo<'T, 'U>() : Unit { body intrinsic; }",
        &expect![[r#"
            Item 4294967295 [0-49]:
                Callable 4294967295 [0-49] (Function):
                    name: Ident 4294967295 [9-12] "Foo"
                    type params:
                        Ident 4294967295 [14-15] "T"
                        Ident 4294967295 [18-19] "U"
                    input: Pat 4294967295 [20-22]: Unit
                    output: Type 4294967295 [25-29]: Unit
                    body: Specializations:
                        SpecDecl 4294967295 [32-47] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn function_single_impl() {
    check(
        item,
        "function Foo(x : Int) : Int { let y = x; y }",
        &expect![[r#"
            Item 4294967295 [0-44]:
                Callable 4294967295 [0-44] (Function):
                    name: Ident 4294967295 [9-12] "Foo"
                    input: Pat 4294967295 [12-21]: Paren:
                        Pat 4294967295 [13-20]: Bind:
                            Ident 4294967295 [13-14] "x"
                            Type 4294967295 [17-20]: Prim (Int)
                    output: Type 4294967295 [24-27]: Prim (Int)
                    body: Block: Block 4294967295 [28-44]:
                        Stmt 4294967295 [30-40]: Local (Immutable):
                            Pat 4294967295 [34-35]: Bind:
                                Ident 4294967295 [34-35] "y"
                            Expr 4294967295 [38-39]: Path: Path 4294967295 [38-39] (Ident 4294967295 [38-39] "x")
                        Stmt 4294967295 [41-42]: Expr: Expr 4294967295 [41-42]: Path: Path 4294967295 [41-42] (Ident 4294967295 [41-42] "y")"#]],
    );
}

#[test]
fn operation_body_impl() {
    check(
        item,
        "operation Foo() : Unit { body (...) { x } }",
        &expect![[r#"
            Item 4294967295 [0-43]:
                Callable 4294967295 [0-43] (Operation):
                    name: Ident 4294967295 [10-13] "Foo"
                    input: Pat 4294967295 [13-15]: Unit
                    output: Type 4294967295 [18-22]: Unit
                    body: Specializations:
                        SpecDecl 4294967295 [25-41] (Body): Impl:
                            Pat 4294967295 [30-35]: Paren:
                                Pat 4294967295 [31-34]: Elided
                            Block 4294967295 [36-41]:
                                Stmt 4294967295 [38-39]: Expr: Expr 4294967295 [38-39]: Path: Path 4294967295 [38-39] (Ident 4294967295 [38-39] "x")"#]],
    );
}

#[test]
fn operation_body_ctl_impl() {
    check(
        item,
        "operation Foo() : Unit { body (...) { x } controlled (cs, ...) { y } }",
        &expect![[r#"
            Item 4294967295 [0-70]:
                Callable 4294967295 [0-70] (Operation):
                    name: Ident 4294967295 [10-13] "Foo"
                    input: Pat 4294967295 [13-15]: Unit
                    output: Type 4294967295 [18-22]: Unit
                    body: Specializations:
                        SpecDecl 4294967295 [25-41] (Body): Impl:
                            Pat 4294967295 [30-35]: Paren:
                                Pat 4294967295 [31-34]: Elided
                            Block 4294967295 [36-41]:
                                Stmt 4294967295 [38-39]: Expr: Expr 4294967295 [38-39]: Path: Path 4294967295 [38-39] (Ident 4294967295 [38-39] "x")
                        SpecDecl 4294967295 [42-68] (Ctl): Impl:
                            Pat 4294967295 [53-62]: Tuple:
                                Pat 4294967295 [54-56]: Bind:
                                    Ident 4294967295 [54-56] "cs"
                                Pat 4294967295 [58-61]: Elided
                            Block 4294967295 [63-68]:
                                Stmt 4294967295 [65-66]: Expr: Expr 4294967295 [65-66]: Path: Path 4294967295 [65-66] (Ident 4294967295 [65-66] "y")"#]],
    );
}

#[test]
fn operation_impl_and_gen() {
    check(
        item,
        "operation Foo() : Unit { body (...) { x } adjoint self; }",
        &expect![[r#"
            Item 4294967295 [0-57]:
                Callable 4294967295 [0-57] (Operation):
                    name: Ident 4294967295 [10-13] "Foo"
                    input: Pat 4294967295 [13-15]: Unit
                    output: Type 4294967295 [18-22]: Unit
                    body: Specializations:
                        SpecDecl 4294967295 [25-41] (Body): Impl:
                            Pat 4294967295 [30-35]: Paren:
                                Pat 4294967295 [31-34]: Elided
                            Block 4294967295 [36-41]:
                                Stmt 4294967295 [38-39]: Expr: Expr 4294967295 [38-39]: Path: Path 4294967295 [38-39] (Ident 4294967295 [38-39] "x")
                        SpecDecl 4294967295 [42-55] (Adj): Gen: Slf"#]],
    );
}

#[test]
fn operation_is_adj() {
    check(
        item,
        "operation Foo() : Unit is Adj {}",
        &expect![[r#"
            Item 4294967295 [0-32]:
                Callable 4294967295 [0-32] (Operation):
                    name: Ident 4294967295 [10-13] "Foo"
                    input: Pat 4294967295 [13-15]: Unit
                    output: Type 4294967295 [18-22]: Unit
                    functors: Functor Expr 4294967295 [26-29]: Adj
                    body: Block: Block 4294967295 [30-32]: <empty>"#]],
    );
}

#[test]
fn operation_is_adj_ctl() {
    check(
        item,
        "operation Foo() : Unit is Adj + Ctl {}",
        &expect![[r#"
            Item 4294967295 [0-38]:
                Callable 4294967295 [0-38] (Operation):
                    name: Ident 4294967295 [10-13] "Foo"
                    input: Pat 4294967295 [13-15]: Unit
                    output: Type 4294967295 [18-22]: Unit
                    functors: Functor Expr 4294967295 [26-35]: BinOp Union: (Functor Expr 4294967295 [26-29]: Adj) (Functor Expr 4294967295 [32-35]: Ctl)
                    body: Block: Block 4294967295 [36-38]: <empty>"#]],
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
            Item 4294967295 [0-28]:
                meta:
                    Visibility 4294967295 [0-8] (Internal)
                New Type (Ident 4294967295 [17-20] "Foo"): TyDef 4294967295 [23-27]: Field:
                    Type 4294967295 [23-27]: Unit"#]],
    );
}

#[test]
fn internal_function() {
    check(
        item,
        "internal function Foo() : Unit {}",
        &expect![[r#"
            Item 4294967295 [0-33]:
                meta:
                    Visibility 4294967295 [0-8] (Internal)
                Callable 4294967295 [9-33] (Function):
                    name: Ident 4294967295 [18-21] "Foo"
                    input: Pat 4294967295 [21-23]: Unit
                    output: Type 4294967295 [26-30]: Unit
                    body: Block: Block 4294967295 [31-33]: <empty>"#]],
    );
}

#[test]
fn internal_operation() {
    check(
        item,
        "internal operation Foo() : Unit {}",
        &expect![[r#"
            Item 4294967295 [0-34]:
                meta:
                    Visibility 4294967295 [0-8] (Internal)
                Callable 4294967295 [9-34] (Operation):
                    name: Ident 4294967295 [19-22] "Foo"
                    input: Pat 4294967295 [22-24]: Unit
                    output: Type 4294967295 [27-31]: Unit
                    body: Block: Block 4294967295 [32-34]: <empty>"#]],
    );
}

#[test]
fn attr_no_args() {
    check(
        attr,
        "@Foo()",
        &expect![[r#"
            Attr 4294967295 [0-6] (Path 4294967295 [1-4] (Ident 4294967295 [1-4] "Foo")):
                Expr 4294967295 [4-6]: Unit"#]],
    );
}

#[test]
fn attr_single_arg() {
    check(
        attr,
        "@Foo(123)",
        &expect![[r#"
            Attr 4294967295 [0-9] (Path 4294967295 [1-4] (Ident 4294967295 [1-4] "Foo")):
                Expr 4294967295 [4-9]: Paren: Expr 4294967295 [5-8]: Lit: Int(123)"#]],
    );
}

#[test]
fn attr_two_args() {
    check(
        attr,
        "@Foo(123, \"bar\")",
        &expect![[r#"
            Attr 4294967295 [0-16] (Path 4294967295 [1-4] (Ident 4294967295 [1-4] "Foo")):
                Expr 4294967295 [4-16]: Tuple:
                    Expr 4294967295 [5-8]: Lit: Int(123)
                    Expr 4294967295 [10-15]: Lit: String("bar")"#]],
    );
}

#[test]
fn open_attr() {
    check(
        item,
        "@Foo() open Bar;",
        &expect![[r#"
            Item 4294967295 [0-16]:
                meta:
                    Attr 4294967295 [0-6] (Path 4294967295 [1-4] (Ident 4294967295 [1-4] "Foo")):
                        Expr 4294967295 [4-6]: Unit
                Open (Ident 4294967295 [12-15] "Bar")"#]],
    );
}

#[test]
fn newtype_attr() {
    check(
        item,
        "@Foo() newtype Bar = Unit;",
        &expect![[r#"
            Item 4294967295 [0-26]:
                meta:
                    Attr 4294967295 [0-6] (Path 4294967295 [1-4] (Ident 4294967295 [1-4] "Foo")):
                        Expr 4294967295 [4-6]: Unit
                New Type (Ident 4294967295 [15-18] "Bar"): TyDef 4294967295 [21-25]: Field:
                    Type 4294967295 [21-25]: Unit"#]],
    );
}

#[test]
fn operation_one_attr() {
    check(
        item,
        "@Foo() operation Bar() : Unit {}",
        &expect![[r#"
            Item 4294967295 [0-32]:
                meta:
                    Attr 4294967295 [0-6] (Path 4294967295 [1-4] (Ident 4294967295 [1-4] "Foo")):
                        Expr 4294967295 [4-6]: Unit
                Callable 4294967295 [7-32] (Operation):
                    name: Ident 4294967295 [17-20] "Bar"
                    input: Pat 4294967295 [20-22]: Unit
                    output: Type 4294967295 [25-29]: Unit
                    body: Block: Block 4294967295 [30-32]: <empty>"#]],
    );
}

#[test]
fn operation_two_attrs() {
    check(
        item,
        "@Foo() @Bar() operation Baz() : Unit {}",
        &expect![[r#"
            Item 4294967295 [0-39]:
                meta:
                    Attr 4294967295 [0-6] (Path 4294967295 [1-4] (Ident 4294967295 [1-4] "Foo")):
                        Expr 4294967295 [4-6]: Unit
                    Attr 4294967295 [7-13] (Path 4294967295 [8-11] (Ident 4294967295 [8-11] "Bar")):
                        Expr 4294967295 [11-13]: Unit
                Callable 4294967295 [14-39] (Operation):
                    name: Ident 4294967295 [24-27] "Baz"
                    input: Pat 4294967295 [27-29]: Unit
                    output: Type 4294967295 [32-36]: Unit
                    body: Block: Block 4294967295 [37-39]: <empty>"#]],
    );
}

#[test]
fn namespace_function() {
    check_vec(
        namespaces,
        "namespace A { function Foo() : Unit { body intrinsic; } }",
        &expect![[r#"
            Namespace 4294967295 [0-57] (Ident 4294967295 [10-11] "A"):
                Item 4294967295 [14-55]:
                    Callable 4294967295 [14-55] (Function):
                        name: Ident 4294967295 [23-26] "Foo"
                        input: Pat 4294967295 [26-28]: Unit
                        output: Type 4294967295 [31-35]: Unit
                        body: Specializations:
                            SpecDecl 4294967295 [38-53] (Body): Gen: Intrinsic"#]],
    );
}

#[test]
fn two_namespaces() {
    check_vec(
        namespaces,
        "namespace A {} namespace B {}",
        &expect![[r#"
            Namespace 4294967295 [0-14] (Ident 4294967295 [10-11] "A"):,
            Namespace 4294967295 [15-29] (Ident 4294967295 [25-26] "B"):"#]],
    );
}

#[test]
fn two_open_items() {
    check_vec(
        namespaces,
        "namespace A { open B; open C; }",
        &expect![[r#"
            Namespace 4294967295 [0-31] (Ident 4294967295 [10-11] "A"):
                Item 4294967295 [14-21]:
                    Open (Ident 4294967295 [19-20] "B")
                Item 4294967295 [22-29]:
                    Open (Ident 4294967295 [27-28] "C")"#]],
    );
}

#[test]
fn two_ty_items() {
    check_vec(
        namespaces,
        "namespace A { newtype B = Unit; newtype C = Unit; }",
        &expect![[r#"
            Namespace 4294967295 [0-51] (Ident 4294967295 [10-11] "A"):
                Item 4294967295 [14-31]:
                    New Type (Ident 4294967295 [22-23] "B"): TyDef 4294967295 [26-30]: Field:
                        Type 4294967295 [26-30]: Unit
                Item 4294967295 [32-49]:
                    New Type (Ident 4294967295 [40-41] "C"): TyDef 4294967295 [44-48]: Field:
                        Type 4294967295 [44-48]: Unit"#]],
    );
}

#[test]
fn two_callable_items() {
    check_vec(
        namespaces,
        "namespace A { operation B() : Unit {} function C() : Unit {} }",
        &expect![[r#"
            Namespace 4294967295 [0-62] (Ident 4294967295 [10-11] "A"):
                Item 4294967295 [14-37]:
                    Callable 4294967295 [14-37] (Operation):
                        name: Ident 4294967295 [24-25] "B"
                        input: Pat 4294967295 [25-27]: Unit
                        output: Type 4294967295 [30-34]: Unit
                        body: Block: Block 4294967295 [35-37]: <empty>
                Item 4294967295 [38-60]:
                    Callable 4294967295 [38-60] (Function):
                        name: Ident 4294967295 [47-48] "C"
                        input: Pat 4294967295 [48-50]: Unit
                        output: Type 4294967295 [53-57]: Unit
                        body: Block: Block 4294967295 [58-60]: <empty>"#]],
    );
}
