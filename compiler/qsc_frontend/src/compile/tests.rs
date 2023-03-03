// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{compile, FileId};
use indoc::indoc;
use qsc_ast::ast::{Expr, ExprKind, ItemKind, Path};

#[test]
fn one_file_no_entry() {
    let context = compile(
        &[indoc! {"
            namespace Foo {
                function A() : Unit {}
            }
        "}],
        "",
    );
    assert!(context.errors().is_empty(), "{:#?}", context.errors());
    let entry = &context.package().entry;
    assert!(entry.is_none(), "{entry:#?}");
}

#[test]
fn one_file_error() {
    let context = compile(
        &[indoc! {"
            namespace Foo {
                function A() : Unit {
                    x
                }
            }
        "}],
        "",
    );

    assert_eq!(context.errors().len(), 1, "{:#?}", context.errors());
    let error = &context.errors()[0];
    let (file, span) = context.file_span(error.span);
    assert_eq!(file, FileId(0));
    assert_eq!(span.lo, 50);
    assert_eq!(span.hi, 51);
}

#[test]
fn two_files_dependency() {
    let context = compile(
        &[
            indoc! {"
                namespace Foo {
                    function A() : Unit {}
                }
            "},
            indoc! {"
                namespace Foo {
                    function B() : Unit {
                        A();
                    }
                }
            "},
        ],
        "",
    );
    assert!(context.errors().is_empty(), "{:#?}", context.errors());
}

#[test]
fn two_files_mutual_dependency() {
    let context = compile(
        &[
            indoc! {"
                namespace Foo {
                    function A() : Unit {
                        B();
                    }
                }
            "},
            indoc! {"
                namespace Foo {
                    function B() : Unit {
                        A();
                    }
                }    
            "},
        ],
        "",
    );
    assert!(context.errors().is_empty(), "{:#?}", context.errors());
}

#[test]
fn two_files_error() {
    let context = compile(
        &[
            indoc! {"
                namespace Foo {
                    function A() : Unit {}
                }
            "},
            indoc! {"
                namespace Foo {
                    function B() : Unit {
                        C();
                    }
                }
            "},
        ],
        "",
    );

    assert_eq!(context.errors.len(), 1, "{:#?}", context.errors());
    let error = &context.errors()[0];
    let (file, span) = context.file_span(error.span);
    assert_eq!(file, FileId(1));
    assert_eq!(span.lo, 50);
    assert_eq!(span.hi, 51);
}

#[test]
fn entry_call_operation() {
    let context = compile(
        &[indoc! {"
                namespace Foo {
                    operation A() : Unit {}
                }
            "}],
        "Foo.A()",
    );
    assert!(context.errors.is_empty(), "{:#?}", context.errors());

    let operation =
        if let ItemKind::Callable(callable) = &context.package().namespaces[0].items[0].kind {
            context
                .symbols
                .get(callable.name.id)
                .expect("Callable should have a symbol ID.")
        } else {
            panic!("First item should be a callable.")
        };

    if let Some(Expr {
        kind: ExprKind::Call(callee, _),
        ..
    }) = &context.package().entry
    {
        if let ExprKind::Path(Path { id, .. }) = callee.kind {
            assert_eq!(context.symbols.get(id), Some(operation));
        } else {
            panic!("Callee should be a path.");
        }
    } else {
        panic!("Entry should be a call expression.");
    }
}

#[test]
fn entry_error() {
    let context = compile(
        &[indoc! {"
                namespace Foo {
                    operation A() : Unit {}
                }
            "}],
        "Foo.B()",
    );

    assert_eq!(context.errors.len(), 1, "{:#?}", context.errors());
    let error = &context.errors()[0];
    let (file, span) = context.file_span(error.span);
    assert_eq!(file, FileId(1));
    assert_eq!(span.lo, 0);
    assert_eq!(span.hi, 5);
}
