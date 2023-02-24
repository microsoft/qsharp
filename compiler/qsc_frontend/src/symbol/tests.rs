// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{GlobalTable, Id, Table};
use crate::{id, parse};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::{
    ast::{Ident, Path, Span},
    mut_visit::MutVisitor,
    visit::Visitor,
};

struct Renamer<'a> {
    symbols: &'a Table,
    changes: Vec<(Span, Id)>,
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
            input.replace_range(span, &format!("_{}", id.0));
        }
    }
}

impl Visitor<'_> for Renamer<'_> {
    fn visit_path(&mut self, path: &Path) {
        if let Some(&id) = self.symbols.nodes.get(&path.id) {
            self.changes.push((path.span, id));
        }
    }

    fn visit_ident(&mut self, ident: &Ident) {
        if let Some(&id) = self.symbols.nodes.get(&ident.id) {
            self.changes.push((ident.span, id));
        }
    }
}

fn check(input: &str, expect: &Expect) {
    let (mut package, errors) = parse::package(input);
    assert!(errors.is_empty(), "Program has syntax errors: {errors:#?}");

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

    if errors.is_empty() {
        expect.assert_eq(&output);
    } else {
        expect.assert_eq(&format!("{output}\n{errors:#?}"));
    }
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
                function _0() : Unit {}

                function _1() : Unit {
                    _0();
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
                function _0() : Unit {}
            }

            namespace Bar {
                open Foo;

                function _1() : Unit {
                    _0();
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
                function _0() : Unit {}
            }

            namespace Bar {
                open Foo as F;

                function _1() : Unit {
                    _0();
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
                function _0() : Int {
                    let _1 = 0;
                    _1
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
                function _0() : Int {
                    let _1 = 0;
                    let _2 = {
                        let _3 = 1;
                        _3
                    };
                    _1 + _2
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
                function _0(_1 : Int) : Int {
                    _1
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
                operation _0(_1 : Qubit) : (Qubit[], Qubit) {
                    controlled (_2, ...) {
                        (_2, _1)
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
                operation _0(_1 : Qubit[]) : Qubit[] {
                    controlled (_2, ...) {
                        _2
                    }
                    body ... {
                        _1
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
                function _0() : Unit {}

                function _1() : Int {
                    _0();
                    let _2 = 1;
                    _2
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
                function _0() : Unit {}
            }

            namespace Bar {
                function _1() : Unit {}
            }

            namespace Baz {
                open Foo as Alias;
                open Bar as Alias;

                function _2() : Unit {
                    _0();
                    _1();
                }
            }
        "#]],
    );
}
