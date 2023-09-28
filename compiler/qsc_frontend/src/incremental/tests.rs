// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Compiler, Increment};
use crate::compile::{self, CompileUnit, PackageStore, TargetProfile};
use expect_test::{expect, Expect};
use indoc::indoc;
use miette::Diagnostic;

#[test]
fn one_callable() {
    let store = PackageStore::new(compile::core());
    let mut compiler = Compiler::new(&store, vec![], TargetProfile::Full);
    let unit = compiler
        .compile_fragments(
            &mut CompileUnit::default(),
            "test_1",
            "namespace Foo { operation Main() : Unit {} }",
        )
        .expect("compilation should succeed");

    check_unit(
        &expect![[r#"
            ast:
            Package 0:
                Namespace 1 [0-44] (Ident 2 [10-13] "Foo"):
                    Item 3 [16-42]:
                        Callable 4 [16-42] (Operation):
                            name: Ident 5 [26-30] "Main"
                            input: Pat 6 [30-32]: Unit
                            output: Type 7 [35-39]: Path: Path 8 [35-39] (Ident 9 [35-39] "Unit")
                            body: Block: Block 10 [40-42]: <empty>
            names:
            node_id:2,node_id:5,node_id:8,
            terms:
            node_id:6,node_id:10,
            hir:
            Package:
                Item 0 [0-44] (Public):
                    Namespace (Ident 5 [10-13] "Foo"): Item 1
                Item 1 [16-42] (Public):
                    Parent: 0
                    Callable 0 [16-42] (operation):
                        name: Ident 1 [26-30] "Main"
                        input: Pat 2 [30-32] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [16-42]: Impl:
                            Block 4 [40-42]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
        &unit,
    );
}

#[test]
fn one_statement() {
    let store = PackageStore::new(compile::core());
    let mut compiler = Compiler::new(&store, vec![], TargetProfile::Full);
    let unit = compiler
        .compile_fragments(&mut CompileUnit::default(), "test_1", "use q = Qubit();")
        .expect("compilation should succeed");

    check_unit(
        &expect![[r#"
            ast:
            Package 0:
                Stmt 1 [0-16]: Qubit (Fresh)
                    Pat 2 [4-5]: Bind:
                        Ident 3 [4-5] "q"
                    QubitInit 4 [8-15] Single
            names:
            node_id:3,
            terms:
            node_id:1,node_id:2,node_id:3,node_id:4,
            hir:
            Package:"#]],
        &unit,
    );
}

#[test]
fn parse_error() {
    let store = PackageStore::new(compile::core());
    let mut compiler = Compiler::new(&store, vec![], TargetProfile::Full);
    let errors = compiler
        .compile_fragments(&mut CompileUnit::default(), "test_1", "}}")
        .expect_err("should fail");

    assert!(!errors.is_empty());
}

#[test]
fn conditional_compilation_not_available() {
    let store = PackageStore::new(compile::core());
    let mut compiler = Compiler::new(&store, vec![], TargetProfile::Full);
    let errors = compiler
        .compile_fragments(
            &mut CompileUnit::default(),
            "test_1",
            indoc! {"
                @Config(Base)
                function Dropped() : Unit {}

                function Usage() : Unit {
                    Dropped();
                }
            "},
        )
        .expect_err("should fail");

    assert!(!errors.is_empty());
}

#[test]
fn errors_across_multiple_lines() {
    let mut store = PackageStore::new(compile::core());
    let std = compile::std(&store, TargetProfile::Full);
    let std_id = store.insert(std);
    let mut compiler = Compiler::new(&store, [std_id], TargetProfile::Full);
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

fn check_unit(expect: &Expect, actual: &Increment) {
    let ast = format!("ast:\n{}", actual.ast.package);

    let names = format!(
        "\nnames:\n{}",
        actual
            .ast
            .names
            .iter()
            .map(|n| format!("node_id:{},", n.0))
            .collect::<String>()
    );
    let terms = format!(
        "\nterms:\n{}",
        actual
            .ast
            .tys
            .terms
            .iter()
            .map(|n| format!("node_id:{},", n.0))
            .collect::<String>()
    );

    let hir = format!("\nhir:\n{}", actual.hir);

    expect.assert_eq(&[ast, names, terms, hir].into_iter().collect::<String>());
}
