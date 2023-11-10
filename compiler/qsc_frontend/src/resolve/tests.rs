// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Error, Names, Res};
use crate::{compile, resolve::Resolver};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::{
    assigner::Assigner as AstAssigner,
    ast::{Ident, NodeId, Package, Path, TopLevelNode},
    mut_visit::MutVisitor,
    visit::{self, Visitor},
};
use qsc_data_structures::span::Span;
use qsc_hir::assigner::Assigner as HirAssigner;
use std::fmt::Write;

struct Renamer<'a> {
    names: &'a Names,
    changes: Vec<(Span, Res)>,
}

impl<'a> Renamer<'a> {
    fn new(names: &'a Names) -> Self {
        Self {
            names,
            changes: Vec::new(),
        }
    }

    fn rename(&self, input: &mut String) {
        for (span, res) in self.changes.iter().rev() {
            let name = match res {
                Res::Item(item, _) => match item.package {
                    None => format!("item{}", item.item),
                    Some(package) => format!("package{package}_item{}", item.item),
                },
                Res::Local(node) => format!("local{node}"),
                Res::PrimTy(prim) => format!("{prim:?}"),
                Res::UnitTy => "Unit".to_string(),
                Res::Param(id) => format!("param{id}"),
            };
            input.replace_range((span.lo as usize)..(span.hi as usize), &name);
        }
    }
}

impl Visitor<'_> for Renamer<'_> {
    fn visit_path(&mut self, path: &Path) {
        if let Some(&id) = self.names.get(path.id) {
            self.changes.push((path.span, id));
        } else {
            visit::walk_path(self, path);
        }
    }

    fn visit_ident(&mut self, ident: &Ident) {
        if let Some(&id) = self.names.get(ident.id) {
            self.changes.push((ident.span, id));
        }
    }
}

fn check(input: &str, expect: &Expect) {
    expect.assert_eq(&resolve_names(input));
}

fn resolve_names(input: &str) -> String {
    let (package, names, errors) = compile(input);
    let mut renamer = Renamer::new(&names);
    renamer.visit_package(&package);
    let mut output = input.to_string();
    renamer.rename(&mut output);
    if !errors.is_empty() {
        output += "\n";
    }
    for error in &errors {
        writeln!(output, "// {error:?}").expect("string should be writable");
    }
    output
}

fn compile(input: &str) -> (Package, Names, Vec<Error>) {
    let (namespaces, parse_errors) = qsc_parse::namespaces(input);
    assert!(parse_errors.is_empty(), "parse failed: {parse_errors:#?}");
    let mut package = Package {
        id: NodeId::default(),
        nodes: namespaces
            .into_iter()
            .map(TopLevelNode::Namespace)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        entry: None,
    };

    AstAssigner::new().visit_package(&mut package);

    let mut cond_compile = compile::preprocess::Conditional::new(compile::TargetProfile::Full);
    cond_compile.visit_package(&mut package);
    let dropped_names = cond_compile.into_names();

    let mut assigner = HirAssigner::new();
    let mut globals = super::GlobalTable::new();
    let mut errors = globals.add_local_package(&mut assigner, &package);
    let mut resolver = Resolver::new(globals, dropped_names);
    resolver.with(&mut assigner).visit_package(&package);
    let (names, mut resolve_errors) = resolver.into_names();
    errors.append(&mut resolve_errors);
    (package, names, errors)
}

#[test]
fn global_callable() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {}

                function B() : Unit {
                    A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}

                function item2() : Unit {
                    item1();
                }
            }
        "#]],
    );
}

#[test]
fn global_callable_recursive() {
    check(
        indoc! {
            "namespace Foo {
                function A() : Unit {
                    A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {
                    item1();
                }
            }
        "#]],
    );
}

#[test]
fn global_callable_internal() {
    check(
        indoc! {"
            namespace Foo {
                internal function A() : Unit {}

                function B() : Unit {
                    A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                internal function item1() : Unit {}

                function item2() : Unit {
                    item1();
                }
            }
        "#]],
    );
}

#[test]
fn global_callable_duplicate_error() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {}
                operation A() : Unit {}
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
                operation item2() : Unit {}
            }

            // Duplicate("A", "Foo", Span { lo: 57, hi: 58 })
        "#]],
    );
}

#[test]
fn global_path() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {}
            }

            namespace Bar {
                function B() : Unit {
                    Foo.A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
            }

            namespace item2 {
                function item3() : Unit {
                    item1();
                }
            }
        "#]],
    );
}

#[test]
fn open_namespace() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {}
            }

            namespace Bar {
                open Foo;

                function B() : Unit {
                    A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
            }

            namespace item2 {
                open Foo;

                function item3() : Unit {
                    item1();
                }
            }
        "#]],
    );
}

#[test]
fn open_alias() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {}
            }

            namespace Bar {
                open Foo as F;

                function B() : Unit {
                    F.A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
            }

            namespace item2 {
                open Foo as F;

                function item3() : Unit {
                    item1();
                }
            }
        "#]],
    );
}

#[test]
fn prelude_callable() {
    check(
        indoc! {"
            namespace Microsoft.Quantum.Core {
                function A() : Unit {}
            }

            namespace Foo {
                function B() : Unit {
                    A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
            }

            namespace item2 {
                function item3() : Unit {
                    item1();
                }
            }
        "#]],
    );
}

#[test]
fn parent_namespace_shadows_prelude() {
    check(
        indoc! {"
            namespace Microsoft.Quantum.Core {
                function A() : Unit {}
            }

            namespace Foo {
                function A() : Unit {}

                function B() : Unit {
                    A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
            }

            namespace item2 {
                function item3() : Unit {}

                function item4() : Unit {
                    item3();
                }
            }
        "#]],
    );
}

#[test]
fn open_shadows_prelude() {
    check(
        indoc! {"
            namespace Microsoft.Quantum.Core {
                function A() : Unit {}
            }

            namespace Foo {
                function A() : Unit {}
            }

            namespace Bar {
                open Foo;

                function B() : Unit {
                    A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
            }

            namespace item2 {
                function item3() : Unit {}
            }

            namespace item4 {
                open Foo;

                function item5() : Unit {
                    item3();
                }
            }
        "#]],
    );
}

#[test]
fn ambiguous_prelude() {
    check(
        indoc! {"
        namespace Microsoft.Quantum.Canon {
            function A() : Unit {}
        }

        namespace Microsoft.Quantum.Core {
            function A() : Unit {}
        }

        namespace Foo {
            function B() : Unit {
                A();
            }
        }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
            }

            namespace item2 {
                function item3() : Unit {}
            }

            namespace item4 {
                function item5() : Unit {
                    A();
                }
            }

            // AmbiguousPrelude { name: "A", candidate_a: "Microsoft.Quantum.Canon", candidate_b: "Microsoft.Quantum.Core", span: Span { lo: 181, hi: 182 } }
        "#]],
    );
}

#[test]
fn local_var() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Int {
                    let x = 0;
                    x
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Int {
                    let local13 = 0;
                    local13
                }
            }
        "#]],
    );
}

#[test]
fn shadow_local() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Int {
                    let x = 0;
                    let y = {
                        let x = 1;
                        x
                    };
                    x + y
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Int {
                    let local13 = 0;
                    let local17 = {
                        let local22 = 1;
                        local22
                    };
                    local13 + local17
                }
            }
        "#]],
    );
}

#[test]
fn callable_param() {
    check(
        indoc! {"
            namespace Foo {
                function A(x : Int) : Int {
                    x
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1(local8 : Int) : Int {
                    local8
                }
            }
        "#]],
    );
}

#[test]
fn spec_param() {
    check(
        indoc! {"
            namespace Foo {
                operation A(q : Qubit) : (Qubit[], Qubit) {
                    controlled (cs, ...) {
                        (cs, q)
                    }
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1(local8 : Qubit) : (Qubit[], Qubit) {
                    controlled (local23, ...) {
                        (local23, local8)
                    }
                }
            }
        "#]],
    );
}

#[test]
fn spec_param_shadow_disallowed() {
    check(
        indoc! {"
            namespace Foo {
                operation A(qs : Qubit[]) : Qubit[] {
                    controlled (qs, ...) {
                        qs
                    }
                    body ... {
                        qs
                    }
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1(local8 : Qubit[]) : Qubit[] {
                    controlled (local20, ...) {
                        local20
                    }
                    body ... {
                        local8
                    }
                }
            }

            // DuplicateBinding("qs", Span { lo: 78, hi: 80 })
        "#]],
    );
}

#[test]
fn local_shadows_global() {
    check(
        indoc! {"
            namespace Foo {
                function x() : Unit {}

                function y() : Int {
                    x();
                    let x = 1;
                    x
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}

                function item2() : Int {
                    item1();
                    let local27 = 1;
                    local27
                }
            }
        "#]],
    );
}

#[test]
fn shadow_same_block() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Int {
                    let x = 0;
                    let x = x + 1;
                    x
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Int {
                    let local13 = 0;
                    let local17 = local13 + 1;
                    local17
                }
            }
        "#]],
    );
}

#[test]
fn parent_namespace_shadows_open() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {}
            }

            namespace Bar {
                open Foo;

                function A() : Unit {}

                function B() : Unit {
                    A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
            }

            namespace item2 {
                open Foo;

                function item3() : Unit {}

                function item4() : Unit {
                    item3();
                }
            }
        "#]],
    );
}

#[test]
fn open_alias_shadows_global() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {}
            }

            namespace Bar {
                function A() : Unit {}
            }

            namespace Baz {
                open Foo as Bar;

                function B() : Unit {
                    Bar.A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
            }

            namespace item2 {
                function item3() : Unit {}
            }

            namespace item4 {
                open Foo as Bar;

                function item5() : Unit {
                    item1();
                }
            }
        "#]],
    );
}

#[test]
fn shadowing_disallowed_within_parameters() {
    check(
        indoc! {"
            namespace Test {
                operation Foo(x: Int, y: Double, x: Bool) : Unit {}
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1(local8: Int, local13: Double, local18: Bool) : Unit {}
            }

            // DuplicateBinding("x", Span { lo: 54, hi: 55 })
        "#]],
    );
}

#[test]
fn shadowing_disallowed_within_local_binding() {
    check(
        indoc! {"
            namespace Test {
                operation Foo() : Unit {
                    let (first, second, first) = (1, 2, 3);
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1() : Unit {
                    let (local14, local16, local18) = (1, 2, 3);
                }
            }

            // DuplicateBinding("first", Span { lo: 74, hi: 79 })
        "#]],
    );
}

#[test]
fn shadowing_disallowed_within_for_loop() {
    check(
        indoc! {"
            namespace Test {
                operation Foo() : Unit {
                    for (key, val, key) in [(1, 1, 1)] {}
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1() : Unit {
                    for (local15, local17, local19) in [(1, 1, 1)] {}
                }
            }

            // DuplicateBinding("key", Span { lo: 69, hi: 72 })
        "#]],
    );
}

#[test]
fn shadowing_disallowed_within_lambda_param() {
    check(
        indoc! {"
            namespace Test {
                operation Foo() : Unit {
                    let f = (x, y, x) -> x + y + 1;
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1() : Unit {
                    let local13 = (local17, local19, local21) -> local21 + local19 + 1;
                }
            }

            // DuplicateBinding("x", Span { lo: 69, hi: 70 })
        "#]],
    );
}

#[test]
fn merged_aliases() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {}
            }

            namespace Bar {
                function B() : Unit {}
            }

            namespace Baz {
                open Foo as Alias;
                open Bar as Alias;

                function C() : Unit {
                    Alias.A();
                    Alias.B();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
            }

            namespace item2 {
                function item3() : Unit {}
            }

            namespace item4 {
                open Foo as Alias;
                open Bar as Alias;

                function item5() : Unit {
                    item1();
                    item3();
                }
            }
        "#]],
    );
}

#[test]
fn ty_decl() {
    check(
        indoc! {"
            namespace Foo {
                newtype A = Unit;
                function B(a : A) : Unit {}
            }
        "},
        &expect![[r#"
            namespace item0 {
                newtype item1 = Unit;
                function item2(local14 : item1) : Unit {}
            }
        "#]],
    );
}

#[test]
fn ty_decl_duplicate_error() {
    check(
        indoc! {"
            namespace Foo {
                newtype A = Unit;
                newtype A = Bool;
            }
        "},
        &expect![[r#"
            namespace item0 {
                newtype item1 = Unit;
                newtype item2 = Bool;
            }

            // Duplicate("A", "Foo", Span { lo: 50, hi: 51 })
        "#]],
    );
}

#[test]
fn ty_decl_duplicate_error_on_built_in_ty() {
    check(
        indoc! {"
            namespace Microsoft.Quantum.Core {
                newtype Pauli = Unit;
            }
        "},
        &expect![[r#"
            namespace item0 {
                newtype item1 = Unit;
            }

            // Duplicate("Pauli", "Microsoft.Quantum.Core", Span { lo: 47, hi: 52 })
        "#]],
    );
}

#[test]
fn ty_decl_in_ty_decl() {
    check(
        indoc! {"
            namespace Foo {
                newtype A = Unit;
                newtype B = A;
            }
        "},
        &expect![[r#"
            namespace item0 {
                newtype item1 = Unit;
                newtype item2 = item1;
            }
        "#]],
    );
}

#[test]
fn ty_decl_recursive() {
    check(
        indoc! {"
            namespace Foo {
                newtype A = A;
            }
        "},
        &expect![[r#"
            namespace item0 {
                newtype item1 = item1;
            }
        "#]],
    );
}

#[test]
fn ty_decl_cons() {
    check(
        indoc! {"
            namespace Foo {
                newtype A = Unit;

                function B() : A {
                    A()
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                newtype item1 = Unit;

                function item2() : item1 {
                    item1()
                }
            }
        "#]],
    );
}

#[test]
fn unknown_term() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {
                    B();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {
                    B();
                }
            }

            // NotFound("B", Span { lo: 50, hi: 51 })
        "#]],
    );
}

#[test]
fn unknown_ty() {
    check(
        indoc! {"
            namespace Foo {
                function A(b : B) : Unit {}
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1(local8 : B) : Unit {}
            }

            // NotFound("B", Span { lo: 35, hi: 36 })
        "#]],
    );
}

#[test]
fn open_ambiguous_terms() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {}
            }

            namespace Bar {
                function A() : Unit {}
            }

            namespace Baz {
                open Foo;
                open Bar;

                function C() : Unit {
                    A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
            }

            namespace item2 {
                function item3() : Unit {}
            }

            namespace item4 {
                open Foo;
                open Bar;

                function item5() : Unit {
                    A();
                }
            }

            // Ambiguous { name: "A", first_open: "Foo", second_open: "Bar", name_span: Span { lo: 171, hi: 172 }, first_open_span: Span { lo: 117, hi: 120 }, second_open_span: Span { lo: 131, hi: 134 } }
        "#]],
    );
}

#[test]
fn open_ambiguous_tys() {
    check(
        indoc! {"
            namespace Foo {
                newtype A = Unit;
            }

            namespace Bar {
                newtype A = Unit;
            }

            namespace Baz {
                open Foo;
                open Bar;

                function C(a : A) : Unit {}
            }
        "},
        &expect![[r#"
            namespace item0 {
                newtype item1 = Unit;
            }

            namespace item2 {
                newtype item3 = Unit;
            }

            namespace item4 {
                open Foo;
                open Bar;

                function item5(local28 : A) : Unit {}
            }

            // Ambiguous { name: "A", first_open: "Foo", second_open: "Bar", name_span: Span { lo: 146, hi: 147 }, first_open_span: Span { lo: 107, hi: 110 }, second_open_span: Span { lo: 121, hi: 124 } }
        "#]],
    );
}

#[test]
fn merged_aliases_ambiguous_terms() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {}
            }

            namespace Bar {
                function A() : Unit {}
            }

            namespace Baz {
                open Foo as Alias;
                open Bar as Alias;

                function C() : Unit {
                    Alias.A();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {}
            }

            namespace item2 {
                function item3() : Unit {}
            }

            namespace item4 {
                open Foo as Alias;
                open Bar as Alias;

                function item5() : Unit {
                    Alias.A();
                }
            }

            // Ambiguous { name: "A", first_open: "Foo", second_open: "Bar", name_span: Span { lo: 195, hi: 196 }, first_open_span: Span { lo: 117, hi: 120 }, second_open_span: Span { lo: 140, hi: 143 } }
        "#]],
    );
}

#[test]
fn merged_aliases_ambiguous_tys() {
    check(
        indoc! {"
            namespace Foo {
                newtype A = Unit;
            }

            namespace Bar {
                newtype A = Unit;
            }

            namespace Baz {
                open Foo as Alias;
                open Bar as Alias;

                function C(a : Alias.A) : Unit {}
            }
        "},
        &expect![[r#"
            namespace item0 {
                newtype item1 = Unit;
            }

            namespace item2 {
                newtype item3 = Unit;
            }

            namespace item4 {
                open Foo as Alias;
                open Bar as Alias;

                function item5(local30 : Alias.A) : Unit {}
            }

            // Ambiguous { name: "A", first_open: "Foo", second_open: "Bar", name_span: Span { lo: 170, hi: 171 }, first_open_span: Span { lo: 107, hi: 110 }, second_open_span: Span { lo: 130, hi: 133 } }
        "#]],
    );
}

#[test]
fn lambda_param() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {
                    let f = x -> x + 1;
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {
                    let local13 = local16 -> local16 + 1;
                }
            }
        "#]],
    );
}

#[test]
fn lambda_shadows_local() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Int {
                    let x = 1;
                    let f = x -> x + 1;
                    x
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Int {
                    let local13 = 1;
                    let local17 = local20 -> local20 + 1;
                    local13
                }
            }
        "#]],
    );
}

#[test]
fn for_loop_range() {
    check(
        indoc! {"
            namespace Foo {
                function A() : Unit {
                    for i in 0..9 {
                        let _ = i;
                    }
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {
                    for local14 in 0..9 {
                        let _ = local14;
                    }
                }
            }
        "#]],
    );
}

#[test]
fn for_loop_var() {
    check(
        indoc! {"
            namespace Foo {
                function A(xs : Int[]) : Unit {
                    for x in xs {
                        let _ = x;
                    }
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1(local8 : Int[]) : Unit {
                    for local20 in local8 {
                        let _ = local20;
                    }
                }
            }
        "#]],
    );
}

#[test]
fn repeat_until() {
    check(
        indoc! {"
            namespace Foo {
                operation A() : Unit {
                    mutable cond = false;
                    repeat {
                        set cond = true;
                    } until cond;
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1() : Unit {
                    mutable local13 = false;
                    repeat {
                        set local13 = true;
                    } until local13;
                }
            }
        "#]],
    );
}

#[test]
fn repeat_until_fixup() {
    check(
        indoc! {"
            namespace Foo {
                operation A() : Unit {
                    mutable cond = false;
                    repeat {
                        set cond = false;
                    } until cond
                    fixup {
                        set cond = true;
                    }
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1() : Unit {
                    mutable local13 = false;
                    repeat {
                        set local13 = false;
                    } until local13
                    fixup {
                        set local13 = true;
                    }
                }
            }
        "#]],
    );
}

#[test]
fn repeat_until_fixup_scoping() {
    check(
        indoc! {"
        namespace Foo {
            operation A() : Unit {
                repeat {
                    mutable cond = false;
                }
                until cond
                fixup {
                    set cond = true;
                }
            }
        }"},
        &expect![[r#"
            namespace item0 {
                operation item1() : Unit {
                    repeat {
                        mutable local16 = false;
                    }
                    until cond
                    fixup {
                        set cond = true;
                    }
                }
            }
            // NotFound("cond", Span { lo: 118, hi: 122 })
            // NotFound("cond", Span { lo: 155, hi: 159 })
        "#]],
    );
}

#[test]
fn use_qubit() {
    check(
        indoc! {"
            namespace Foo {
                operation X(q : Qubit) : Unit {
                    body intrinsic;
                }
                operation A() : Unit {
                    use q = Qubit();
                    X(q);
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1(local8 : Qubit) : Unit {
                    body intrinsic;
                }
                operation item2() : Unit {
                    use local26 = Qubit();
                    item1(local26);
                }
            }
        "#]],
    );
}

#[test]
fn use_qubit_block() {
    check(
        indoc! {"
            namespace Foo {
                operation X(q : Qubit) : Unit {
                    body intrinsic;
                }
                operation A() : Unit {
                    use q = Qubit() {
                        X(q);
                    }
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1(local8 : Qubit) : Unit {
                    body intrinsic;
                }
                operation item2() : Unit {
                    use local26 = Qubit() {
                        item1(local26);
                    }
                }
            }
        "#]],
    );
}

#[test]
fn local_function() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Int {
                    function Bar() : Int { 2 }
                    Bar() + 1
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Int {
                    function item2() : Int { 2 }
                    item2() + 1
                }
            }
        "#]],
    );
}

#[test]
fn local_function_use_before_declare() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {
                    Bar();
                    function Bar() : () {}
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : () {
                    item2();
                    function item2() : () {}
                }
            }
        "#]],
    );
}

#[test]
fn local_function_is_really_local() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {
                    function Bar() : () {}
                    Bar();
                }

                function Baz() : () { Bar(); }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : () {
                    function item3() : () {}
                    item3();
                }

                function item2() : () { Bar(); }
            }

            // NotFound("Bar", Span { lo: 119, hi: 122 })
        "#]],
    );
}

#[test]
fn local_function_is_not_closure() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {
                    let x = 2;
                    function Bar() : Int { x }
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : () {
                    let local11 = 2;
                    function item2() : Int { x }
                }
            }

            // NotFound("x", Span { lo: 90, hi: 91 })
        "#]],
    );
}

#[test]
fn local_type() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {
                    newtype Bar = Int;
                    let x = Bar(5);
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : () {
                    newtype item2 = Int;
                    let local18 = item2(5);
                }
            }
        "#]],
    );
}

#[test]
fn local_open() {
    check(
        indoc! {"
            namespace A { function Foo() : () { open B; Bar(); } }
            namespace B { function Bar() : () {} }
        "},
        &expect![[r#"
            namespace item0 { function item1() : () { open B; item3(); } }
            namespace item2 { function item3() : () {} }
        "#]],
    );
}

#[test]
fn local_open_shadows_parent_item() {
    check(
        indoc! {"
            namespace A {
                function Bar() : () {}
                function Foo() : () { open B; Bar(); }
            }

            namespace B { function Bar() : () {} }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : () {}
                function item2() : () { open B; item4(); }
            }

            namespace item3 { function item4() : () {} }
        "#]],
    );
}

#[test]
fn local_open_shadows_parent_open() {
    check(
        indoc! {"
            namespace A {
                open B;
                function Foo() : () { open C; Bar(); }
            }

            namespace B { function Bar() : () {} }
            namespace C { function Bar() : () {} }
        "},
        &expect![[r#"
            namespace item0 {
                open B;
                function item1() : () { open C; item5(); }
            }

            namespace item2 { function item3() : () {} }
            namespace item4 { function item5() : () {} }
        "#]],
    );
}

#[test]
fn update_array_index_var() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {
                    let xs = [2];
                    let i = 0;
                    let ys = xs w/ i <- 3;
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : () {
                    let local11 = [2];
                    let local16 = 0;
                    let local20 = local11 w/ local16 <- 3;
                }
            }
        "#]],
    );
}

#[test]
fn update_array_index_expr() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {
                    let xs = [2];
                    let i = 0;
                    let ys = xs w/ i + 1 <- 3;
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : () {
                    let local11 = [2];
                    let local16 = 0;
                    let local20 = local11 w/ local16 + 1 <- 3;
                }
            }
        "#]],
    );
}

#[test]
fn update_udt_known_field_name() {
    check(
        indoc! {"
            namespace A {
                newtype Pair = (First : Int, Second : Int);

                function Foo() : () {
                    let p = Pair(1, 2);
                    let q = p w/ First <- 3;
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                newtype item1 = (First : Int, Second : Int);

                function item2() : () {
                    let local24 = item1(1, 2);
                    let local34 = local24 w/ First <- 3;
                }
            }
        "#]],
    );
}

#[test]
fn update_udt_known_field_name_expr() {
    check(
        indoc! {"
            namespace A {
                newtype Pair = (First : Int, Second : Int);

                function Foo() : () {
                    let p = Pair(1, 2);
                    let q = p w/ First + 1 <- 3;
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                newtype item1 = (First : Int, Second : Int);

                function item2() : () {
                    let local24 = item1(1, 2);
                    let local34 = local24 w/ First + 1 <- 3;
                }
            }

            // NotFound("First", Span { lo: 138, hi: 143 })
        "#]],
    );
}

#[test]
fn update_udt_unknown_field_name() {
    check(
        indoc! {"
            namespace A {
                newtype Pair = (First : Int, Second : Int);

                function Foo() : () {
                    let p = Pair(1, 2);
                    let q = p w/ Third <- 3;
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                newtype item1 = (First : Int, Second : Int);

                function item2() : () {
                    let local24 = item1(1, 2);
                    let local34 = local24 w/ Third <- 3;
                }
            }
        "#]],
    );
}

#[test]
fn update_udt_unknown_field_name_known_global() {
    check(
        indoc! {"
            namespace A {
                newtype Pair = (First : Int, Second : Int);

                function Third() : () {}

                function Foo() : () {
                    let p = Pair(1, 2);
                    let q = p w/ Third <- 3;
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                newtype item1 = (First : Int, Second : Int);

                function item2() : () {}

                function item3() : () {
                    let local30 = item1(1, 2);
                    let local40 = local30 w/ Third <- 3;
                }
            }
        "#]],
    );
}

#[test]
fn unknown_namespace() {
    check(
        indoc! {"
            namespace A {
                open Microsoft.Quantum.Fake;
            }
        "},
        &expect![[r#"
            namespace item0 {
                open Microsoft.Quantum.Fake;
            }

            // NotFound("Microsoft.Quantum.Fake", Span { lo: 23, hi: 45 })
        "#]],
    );
}

#[test]
fn empty_namespace_works() {
    check(
        indoc! {"
            namespace A {
                open B;
                function foo(): Unit{}
            }
            namespace B {}
        "},
        &expect![[r#"
            namespace item0 {
                open B;
                function item1(): Unit{}
            }
            namespace item2 {}
        "#]],
    );
}

#[test]
fn cyclic_namespace_dependency_supported() {
    check(
        indoc! {"
            namespace A {
                open B;
                operation Foo() : Unit {
                    Bar();
                }
            }
            namespace B {
                open A;
                operation Bar() : Unit {
                    Foo();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                open B;
                operation item1() : Unit {
                    item3();
                }
            }
            namespace item2 {
                open A;
                operation item3() : Unit {
                    item1();
                }
            }
        "#]],
    );
}

#[test]
fn bind_items_in_repeat() {
    check(
        indoc! {"
            namespace A {
                operation B() : Unit {
                    repeat {
                        function C() : Unit {}
                    } until false
                    fixup {
                        function D() : Unit {}
                    }
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1() : Unit {
                    repeat {
                        function item2() : Unit {}
                    } until false
                    fixup {
                        function item3() : Unit {}
                    }
                }
            }
        "#]],
    );
}

#[test]
fn bind_items_in_qubit_use_block() {
    check(
        indoc! {"
            namespace A {
                operation B() : Unit {
                    use q = Qubit() {
                        function C() : Unit {}
                    }
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1() : Unit {
                    use local13 = Qubit() {
                        function item2() : Unit {}
                    }
                }
            }
        "#]],
    );
}

#[test]
fn use_bound_item_in_another_bound_item() {
    check(
        indoc! {"
            namespace A {
                function B() : Unit {
                    function C() : Unit {
                        D();
                    }
                    function D() : Unit {}
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1() : Unit {
                    function item2() : Unit {
                        item3();
                    }
                    function item3() : Unit {}
                }
            }
        "#]],
    );
}

#[test]
fn use_unbound_generic() {
    check(
        indoc! {"
            namespace A {
                function B<'T>(x: 'U) : 'U {
                    x
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1<param0>(local9: 'U) : 'U {
                    local9
                }
            }

            // NotFound("'U", Span { lo: 36, hi: 38 })
            // NotFound("'U", Span { lo: 42, hi: 44 })
        "#]],
    );
}
#[test]
fn resolve_local_generic() {
    check(
        indoc! {"
            namespace A {
                function B<'T>(x: 'T) : 'T {
                    x
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                function item1<param0>(local9: param0) : param0 {
                    local9
                }
            }
        "#]],
    );
}

#[test]
fn dropped_callable() {
    check(
        indoc! {"
            namespace A {
                @Config(Base)
                function Dropped() : Unit {}

                function B() : Unit {
                    Dropped();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                @Config(Base)
                function Dropped() : Unit {}

                function item1() : Unit {
                    Dropped();
                }
            }

            // NotAvailable("Dropped", "A.Dropped", Span { lo: 100, hi: 107 })
        "#]],
    );
}

#[test]
fn multiple_definition_dropped_is_not_found() {
    check(
        indoc! {"
            namespace A {
                @Config(Full)
                operation B() : Unit {}
                @Config(Base)
                operation B() : Unit {}
                @Config(Base)
                operation C() : Unit {}
                @Config(Full)
                operation C() : Unit {}
            }
            namespace D {
                operation E() : Unit {
                    B();
                    C();
                }
                operation F() : Unit {
                    open A;
                    B();
                    C();
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                @Config(Full)
                operation item1() : Unit {}
                @Config(Base)
                operation B() : Unit {}
                @Config(Base)
                operation C() : Unit {}
                @Config(Full)
                operation item2() : Unit {}
            }
            namespace item3 {
                operation item4() : Unit {
                    B();
                    C();
                }
                operation item5() : Unit {
                    open A;
                    item1();
                    item2();
                }
            }

            // NotFound("B", Span { lo: 249, hi: 250 })
            // NotFound("C", Span { lo: 262, hi: 263 })
        "#]],
    );
}
