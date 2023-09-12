// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Compiler, Fragment};
use crate::compile::{self, CompileUnit, PackageStore};
use expect_test::{expect, Expect};
use miette::Diagnostic;

#[test]
fn one_callable() {
    let store = PackageStore::new(compile::core());
    let mut compiler = Compiler::new(&store, vec![]);
    let fragments = compiler
        .compile_fragments(
            &mut CompileUnit::default(),
            "test_1",
            "namespace Foo { operation Main() : Unit {} }",
        )
        .expect("compilation should succeed");

    check_fragment_kinds(
        &expect![[r#"
        [
            "callable",
            "namespace",
        ]
    "#]],
        &fragments,
    );
}

#[test]
fn one_statement() {
    let store = PackageStore::new(compile::core());
    let mut compiler = Compiler::new(&store, vec![]);
    let fragments = compiler
        .compile_fragments(&mut CompileUnit::default(), "test_1", "use q = Qubit();")
        .expect("compilation should succeed");

    check_fragment_kinds(
        &expect![[r#"
            [
                "statement",
            ]
        "#]],
        &fragments,
    );
}

#[test]
fn parse_error() {
    let store = PackageStore::new(compile::core());
    let mut compiler = Compiler::new(&store, vec![]);
    let errors = compiler
        .compile_fragments(&mut CompileUnit::default(), "test_1", "}}")
        .expect_err("should fail");

    assert!(!errors.is_empty());
}

#[test]
fn errors_across_multiple_lines() {
    let mut store = PackageStore::new(compile::core());
    let std = compile::std(&store, compile::TargetProfile::Full);
    let std_id = store.insert(std);
    let mut compiler = Compiler::new(&store, [std_id]);
    let mut unit = CompileUnit::default();
    compiler
        .compile_fragments(
            &mut unit,
            "line_1",
            "namespace Other { operation DumpMachine() : Unit { } }",
        )
        .expect("should succeed");

    compiler
        .compile_fragments(&mut unit, "line_2", "open Other;")
        .expect("should succeed");

    compiler
        .compile_fragments(&mut unit, "line_3", "open Microsoft.Quantum.Diagnostics;")
        .expect("should succeed");

    let errors = compiler
        .compile_fragments(&mut unit, "line_4", "DumpMachine()")
        .expect_err("should fail");

    // Here we're validating that the compiler is able to return
    // error labels mapping to different lines.
    // The `Ambiguous` error is chosen as a test case because
    // it contains multiple spans.
    let labels = errors
        .iter()
        .flat_map(|e| e.labels().into_iter().flatten())
        .map(|l| {
            unit.sources
                .find_by_offset(u32::try_from(l.offset()).expect("offset should fit into u32"))
                .map(|s| &s.name)
        })
        .collect::<Vec<_>>();

    expect![[r#"
        [
            Some(
                "line_4",
            ),
            Some(
                "line_2",
            ),
            Some(
                "line_3",
            ),
            Some(
                "line_4",
            ),
        ]
    "#]]
    .assert_debug_eq(&labels);
}

fn check_fragment_kinds(expect: &Expect, actual: &[Fragment]) {
    expect.assert_debug_eq(
        &actual
            .iter()
            .map(|f| match f {
                Fragment::Stmt(_) => "statement",
                Fragment::Item(item) => match item.kind {
                    qsc_hir::hir::ItemKind::Callable(_) => "callable",
                    qsc_hir::hir::ItemKind::Namespace(_, _) => "namespace",
                    qsc_hir::hir::ItemKind::Ty(_, _) => "ty",
                },
            })
            .collect::<Vec<&str>>(),
    );
}
