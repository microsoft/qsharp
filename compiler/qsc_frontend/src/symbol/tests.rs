// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{DefId, Error, ErrorKind, GlobalTable, Table};
use crate::{id, parse, symbol::PackageIndex};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::{
    ast::{Ident, Package, Path, Span},
    mut_visit::MutVisitor,
    visit::{self, Visitor},
};
use std::fmt::{self, Write};

struct Renamer<'a> {
    symbols: &'a Table,
    changes: Vec<(Span, DefId)>,
}

impl<'a> Renamer<'a> {
    fn new(symbols: &'a Table) -> Self {
        Self {
            symbols,
            changes: Vec::new(),
        }
    }

    fn rename(&self, input: &mut String) {
        for (span, id) in self.changes.iter().rev() {
            assert_eq!(id.package, PackageIndex(0));
            input.replace_range(span, &format!("_{}", id.node));
        }
    }
}

impl Visitor<'_> for Renamer<'_> {
    fn visit_path(&mut self, path: &Path) {
        if let Some(def) = self.symbols.get(path.id) {
            self.changes.push((path.span, def));
        } else {
            visit::walk_path(self, path);
        }
    }

    fn visit_ident(&mut self, ident: &Ident) {
        if let Some(def) = self.symbols.get(ident.id) {
            self.changes.push((ident.span, def));
        }
    }
}

fn check(input: &str, expect: &Expect) {
    let (namespaces, errors) = parse::namespaces(input);
    assert!(errors.is_empty(), "Program has syntax errors: {errors:#?}");
    let mut package = Package::new(namespaces, None);
    let mut assigner = id::Assigner::new();
    assigner.visit_package(&mut package);
    let mut globals = GlobalTable::new();
    globals.visit_package(&package);
    let mut resolver = globals.into_resolver();
    resolver.visit_package(&package);
    let (symbols, errors) = resolver.into_table();
    let mut renamer = Renamer::new(&symbols);
    renamer.visit_package(&package);
    let mut output = input.to_string();
    renamer.rename(&mut output);

    if !errors.is_empty() {
        output += "\n";
    }

    for error in &errors {
        output += "// ";
        write_error(&mut output, error).expect("Error should write to output string.");
        output += "\n";
    }

    expect.assert_eq(&output);
}

fn write_error(mut buffer: impl Write, error: &Error) -> fmt::Result {
    let ErrorKind::Unresolved(candidates) = &error.kind;
    let mut candidates: Vec<_> = candidates.iter().collect();
    candidates.sort();
    write!(
        buffer,
        "Unresolved symbol at {:?} with candidates {:?}.",
        error.span, candidates
    )
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
                    controlled (_18, ...) {
                        (_18, _8)
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
                    controlled (_18, ...) {
                        _18
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

            // Unresolved symbol at Span { lo: 50, hi: 51 } with candidates [].
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

            // Unresolved symbol at Span { lo: 35, hi: 36 } with candidates [].
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

            // Unresolved symbol at Span { lo: 171, hi: 172 } with candidates [DefId { package: PackageIndex(0), node: NodeId(5) }, DefId { package: PackageIndex(0), node: NodeId(13) }].
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

            // Unresolved symbol at Span { lo: 146, hi: 147 } with candidates [DefId { package: PackageIndex(0), node: NodeId(4) }, DefId { package: PackageIndex(0), node: NodeId(10) }].
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

            // Unresolved symbol at Span { lo: 189, hi: 196 } with candidates [DefId { package: PackageIndex(0), node: NodeId(5) }, DefId { package: PackageIndex(0), node: NodeId(13) }].
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

            // Unresolved symbol at Span { lo: 164, hi: 171 } with candidates [DefId { package: PackageIndex(0), node: NodeId(4) }, DefId { package: PackageIndex(0), node: NodeId(10) }].
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
