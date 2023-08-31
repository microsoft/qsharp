// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap, TargetProfile};
use qsc_hir::visit::Visitor;

use crate::borrowck::Checker;

fn check(expr: &str, expect: &Expect) {
    let store = PackageStore::new(compile::core());
    let sources = SourceMap::new([("test".into(), "".into())], Some(expr.into()));
    let unit = compile(&store, &[], sources, TargetProfile::Full);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);

    let mut borrow_check = Checker::default();
    borrow_check.visit_package(&unit.package);
    expect.assert_debug_eq(&borrow_check.errors);
}

#[test]
fn assign_invalid_expr() {
    check(
        "set 0 = 1",
        &expect![[r#"
            [
                Unassignable(
                    Span {
                        lo: 5,
                        hi: 6,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn assignop_mutability_expr() {
    check(
        indoc! {"{
            let x = false;
            set x or= true;
            x
        }"},
        &expect![[r#"
            [
                Mutability(
                    Span {
                        lo: 30,
                        hi: 31,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn assignupdate_immutable_expr() {
    check(
        indoc! {"{
            let x = [1, 2, 3];
            set x w/= 2 <- 4;
            x
        }"},
        &expect![[r#"
            [
                Mutability(
                    Span {
                        lo: 34,
                        hi: 35,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn block_mutable_immutable_expr() {
    check(
        indoc! {"{
            let x = 0;
            set x = 1;
        }"},
        &expect![[r#"
            [
                Mutability(
                    Span {
                        lo: 26,
                        hi: 27,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn block_mutable_expr() {
    check(
        indoc! {"{
            mutable x = 0;
            set x = 1;
        }"},
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn for_loop_iterator_immutable_expr() {
    check(
        "for i in 0..10 { set i = 0; }",
        &expect![[r#"
            [
                Mutability(
                    Span {
                        lo: 22,
                        hi: 23,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn complex_expr_not_assignable() {
    check(
        indoc! {"{
            mutable (x, y) = (0, 0);
            set if false { x } else { y } = 1;
        }"},
        &expect![[r#"
            [
                Unassignable(
                    Span {
                        lo: 40,
                        hi: 65,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn lambda_mutable_closure() {
    check(
        indoc! {"{
            mutable x = 1;
            let f = y -> x + y;
        }"},
        &expect![[r#"
            [
                MutableClosure(
                    Span {
                        lo: 34,
                        hi: 44,
                    },
                ),
            ]
        "#]],
    );
}
