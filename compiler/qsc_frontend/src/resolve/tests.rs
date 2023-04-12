// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{DefId, GlobalTable, PackageSrc, Resolutions};
use crate::{id, parse};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::{
    ast::{Ident, Package, Path, Span},
    mut_visit::MutVisitor,
    visit::{self, Visitor},
};
use std::fmt::Write;

struct Renamer<'a> {
    resolutions: &'a Resolutions,
    changes: Vec<(Span, DefId)>,
}

impl<'a> Renamer<'a> {
    fn new(resolutions: &'a Resolutions) -> Self {
        Self {
            resolutions,
            changes: Vec::new(),
        }
    }

    fn rename(&self, input: &mut String) {
        for (span, id) in self.changes.iter().rev() {
            let name = match id.package {
                PackageSrc::Local => format!("_{}", id.node),
                PackageSrc::Extern(package) => format!("_{package}_{}", id.node),
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
    let mut package = Package::new(namespaces, None);
    let mut assigner = id::Assigner::new();
    assigner.visit_package(&mut package);
    let mut globals = GlobalTable::new();
    globals.visit_package(&package);
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
            namespace Foo {
                function _5() : Unit {}

                function _11() : Unit {
                    _5();
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
            namespace Foo {
                function _5() : Unit {
                    _5();
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
            namespace Foo {
                internal function _5() : Unit {}

                function _11() : Unit {
                    _5();
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
            namespace Foo {
                function _5() : Unit {}
            }

            namespace Bar {
                function _13() : Unit {
                    _5();
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
            namespace Foo {
                function _5() : Unit {}
            }

            namespace Bar {
                open Foo;

                function _15() : Unit {
                    _5();
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
            namespace Foo {
                function _5() : Unit {}
            }

            namespace Bar {
                open Foo as F;

                function _16() : Unit {
                    _5();
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
            namespace Microsoft.Quantum.Core {
                function _5() : Unit {}
            }

            namespace Foo {
                function _13() : Unit {
                    _5();
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
            namespace Microsoft.Quantum.Core {
                function _5() : Unit {}
            }

            namespace Foo {
                function _13() : Unit {}

                function _19() : Unit {
                    _13();
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
            namespace Microsoft.Quantum.Core {
                function _5() : Unit {}
            }

            namespace Foo {
                function _13() : Unit {}
            }

            namespace Bar {
                open Foo;

                function _23() : Unit {
                    _13();
                }
            }
        "#]],
    );
}

#[test]
#[should_panic(expected = "Ambiguity in prelude resolution.")]
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
            namespace Foo { 
                function _5() : Int {
                    let _11 = 0;
                    _11
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
            namespace Foo {
                function _5() : Int {
                    let _11 = 0;
                    let _15 = {
                        let _20 = 1;
                        _20
                    };
                    _11 + _15
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
            namespace Foo {
                function _5(_8 : Int) : Int {
                    _8
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
            namespace Foo {
                operation _5(_8 : Qubit) : (Qubit[], Qubit) {
                    controlled (_17, ...) {
                        (_17, _8)
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
            namespace Foo {
                operation _5(_8 : Qubit[]) : Qubit[] {
                    controlled (_16, ...) {
                        _16
                    }
                    body ... {
                        _8
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
            namespace Foo {
                function _5() : Unit {}

                function _11() : Int {
                    _5();
                    let _23 = 1;
                    _23
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
            namespace Foo {
                function _5() : Int {
                    let _11 = 0;
                    let _15 = _11 + 1;
                    _15
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
            namespace Foo {
                function _5() : Unit {}
            }

            namespace Bar {
                open Foo;

                function _15() : Unit {}

                function _21() : Unit {
                    _15();
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
            namespace Foo {
                function _5() : Unit {}
            }

            namespace Bar {
                function _13() : Unit {}
            }

            namespace Baz {
                open Foo as Bar;

                function _24() : Unit {
                    _5();
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
            namespace Foo {
                function _5() : Unit {}
            }

            namespace Bar {
                function _13() : Unit {}
            }

            namespace Baz {
                open Foo as Alias;
                open Bar as Alias;

                function _27() : Unit {
                    _5();
                    _13();
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
            namespace Foo {
                newtype _4 = Unit;
                function _9(_12 : _4) : Unit {}
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
            namespace Foo {
                newtype _4 = Unit;
                newtype _8 = _4;
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
            namespace Foo {
                newtype _4 = _4;
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
            namespace Foo {
                newtype _4 = Unit;

                function _9() : _4 {
                    _4()
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
            namespace Foo {
                function _5() : Unit {
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
            namespace Foo {
                function _5(_8 : B) : Unit {}
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
            namespace Foo {
                function _5() : Unit {}
            }

            namespace Bar {
                function _13() : Unit {}
            }

            namespace Baz {
                open Foo;
                open Bar;

                function _25() : Unit {
                    A();
                }
            }

            // Ambiguous("A", Span { lo: 171, hi: 172 }, Span { lo: 117, hi: 120 }, Span { lo: 131, hi: 134 })
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
            namespace Foo {
                newtype _4 = Unit;
            }

            namespace Bar {
                newtype _10 = Unit;
            }

            namespace Baz {
                open Foo;
                open Bar;

                function _21(_24 : A) : Unit {}
            }

            // Ambiguous("A", Span { lo: 146, hi: 147 }, Span { lo: 107, hi: 110 }, Span { lo: 121, hi: 124 })
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
            namespace Foo {
                function _5() : Unit {}
            }

            namespace Bar {
                function _13() : Unit {}
            }

            namespace Baz {
                open Foo as Alias;
                open Bar as Alias;

                function _27() : Unit {
                    Alias.A();
                }
            }

            // Ambiguous("A", Span { lo: 189, hi: 196 }, Span { lo: 117, hi: 120 }, Span { lo: 140, hi: 143 })
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
            namespace Foo {
                newtype _4 = Unit;
            }

            namespace Bar {
                newtype _10 = Unit;
            }

            namespace Baz {
                open Foo as Alias;
                open Bar as Alias;

                function _23(_26 : Alias.A) : Unit {}
            }

            // Ambiguous("A", Span { lo: 164, hi: 171 }, Span { lo: 107, hi: 110 }, Span { lo: 130, hi: 133 })
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
            namespace Foo {
                function _5() : Unit {
                    let _11 = _14 -> _14 + 1;
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
            namespace Foo {
                function _5() : Int {
                    let _11 = 1;
                    let _15 = _18 -> _18 + 1;
                    _11
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
            namespace Foo {
                function _5() : Unit {
                    for _12 in 0..9 {
                        let _ = _12;
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
            namespace Foo {
                function _5(_8 : Int[]) : Unit {
                    for _16 in _8 {
                        let _ = _16;
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
            namespace Foo {
                operation _5() : Unit {
                    repeat {
                        let _14 = true;
                    } until _14;
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
            namespace Foo {
                operation _5() : Unit {
                    repeat {
                        mutable _14 = false;
                    } until _14
                    fixup {
                        set _14 = true;
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
            namespace Foo {
                operation _5(_8 : Qubit) : Unit {
                    body intrinsic;
                }
                operation _14() : Unit {
                    use _20 = Qubit();
                    _5(_20);
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
            namespace Foo {
                operation _5(_8 : Qubit) : Unit {
                    body intrinsic;
                }
                operation _14() : Unit {
                    use _20 = Qubit() {
                        _5(_20);
                    }
                }
            }
        "#]],
    );
}
