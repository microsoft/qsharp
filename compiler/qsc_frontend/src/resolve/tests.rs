// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{GlobalTable, Res, Resolutions};
use crate::parse;
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::{
    assigner::Assigner,
    ast::{Ident, NodeId, Package, Path},
    mut_visit::MutVisitor,
    visit::{self, Visitor},
};
use qsc_data_structures::span::Span;
use std::fmt::Write;

struct Renamer<'a> {
    resolutions: &'a Resolutions,
    changes: Vec<(Span, Res)>,
}

impl<'a> Renamer<'a> {
    fn new(resolutions: &'a Resolutions) -> Self {
        Self {
            resolutions,
            changes: Vec::new(),
        }
    }

    fn rename(&self, input: &mut String) {
        for (span, res) in self.changes.iter().rev() {
            let name = match res {
                Res::Item(item) => match item.package {
                    None => format!("item{}", item.item),
                    Some(package) => format!("package{package}_item{}", item.item),
                },
                Res::Local(node) => format!("local{node}"),
            };
            input.replace_range(span, &name);
        }
    }
}

impl Visitor<'_> for Renamer<'_> {
    fn visit_path(&mut self, path: &Path) {
        if let Some(&id) = self.resolutions.get(path.id) {
            self.changes.push((path.span, id));
        } else {
            visit::walk_path(self, path);
        }
    }

    fn visit_ident(&mut self, ident: &Ident) {
        if let Some(&id) = self.resolutions.get(ident.id) {
            self.changes.push((ident.span, id));
        }
    }
}

fn check(input: &str, expect: &Expect) {
    expect.assert_eq(&resolve_names(input));
}

fn resolve_names(input: &str) -> String {
    let (namespaces, errors) = parse::namespaces(input);
    assert!(errors.is_empty(), "Program has syntax errors: {errors:#?}");
    let mut package = Package {
        id: NodeId::default(),
        namespaces,
        entry: None,
    };
    let mut assigner = Assigner::new();
    assigner.visit_package(&mut package);
    let mut globals = GlobalTable::new();
    globals.add_local_package(&package);
    let mut resolver = globals.into_resolver();
    resolver.visit_package(&package);
    let (resolutions, errors) = resolver.into_resolutions();
    let mut renamer = Renamer::new(&resolutions);
    renamer.visit_package(&package);
    let mut output = input.to_string();
    renamer.rename(&mut output);

    if !errors.is_empty() {
        output += "\n";
    }

    for error in &errors {
        writeln!(output, "// {error:?}").expect("error should write to output string");
    }

    output
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
#[should_panic(expected = "ambiguity in prelude resolution")]
fn ambiguous_prelude() {
    resolve_names(indoc! {"
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
    "});
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
                    let local11 = 0;
                    local11
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
                    let local11 = 0;
                    let local15 = {
                        let local20 = 1;
                        local20
                    };
                    local11 + local15
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
                    controlled (local17, ...) {
                        (local17, local8)
                    }
                }
            }
        "#]],
    );
}

#[test]
fn spec_param_shadow() {
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
                    controlled (local16, ...) {
                        local16
                    }
                    body ... {
                        local8
                    }
                }
            }
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
                    let local23 = 1;
                    local23
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
                    let local11 = 0;
                    let local15 = local11 + 1;
                    local15
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
                function item2(local12 : item1) : Unit {}
            }
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

            // Ambiguous("A", "Bar", "Foo", Span { lo: 171, hi: 172 })
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

                function item5(local24 : A) : Unit {}
            }

            // Ambiguous("A", "Bar", "Foo", Span { lo: 146, hi: 147 })
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

            // Ambiguous("A", "Bar", "Foo", Span { lo: 189, hi: 196 })
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

                function item5(local26 : Alias.A) : Unit {}
            }

            // Ambiguous("A", "Bar", "Foo", Span { lo: 164, hi: 171 })
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
                    let local11 = local14 -> local14 + 1;
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
                    let local11 = 1;
                    let local15 = local18 -> local18 + 1;
                    local11
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
                    for local12 in 0..9 {
                        let _ = local12;
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
                    for local16 in local8 {
                        let _ = local16;
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
                    repeat {
                        let cond = true;
                    } until cond;
                }
            }
        "},
        &expect![[r#"
            namespace item0 {
                operation item1() : Unit {
                    repeat {
                        let local14 = true;
                    } until local14;
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
                    repeat {
                        mutable cond = false;
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
                    repeat {
                        mutable local14 = false;
                    } until local14
                    fixup {
                        set local14 = true;
                    }
                }
            }
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
                    use local20 = Qubit();
                    item1(local20);
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
                    use local20 = Qubit() {
                        item1(local20);
                    }
                }
            }
        "#]],
    );
}
