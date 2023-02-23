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
fn local_var() {
    check(
        indoc! {"
            namespace A { 
                function B() : Int {
                    let x = 0;
                    x
                }
            }
        "},
        &expect![[r#"
            namespace A { 
                function _0() : Int {
                    let _1 = 0;
                    _1
                }
            }
        "#]],
    );
}

#[test]
fn global_callable() {
    check(
        indoc! {"
            namespace A {
                function B() : Unit {}

                function C() : Unit {
                    B();
                }
            }
        "},
        &expect![[r#"
            namespace A {
                function _0() : Unit {}

                function _1() : Unit {
                    _0();
                }
            }
        "#]],
    );
}
