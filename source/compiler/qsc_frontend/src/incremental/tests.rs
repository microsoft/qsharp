// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Compiler, Increment};
use crate::{
    compile::{self, CompileUnit, PackageStore, SourceMap},
    incremental::Error,
};
use expect_test::{Expect, expect};
use indoc::indoc;
use miette::Diagnostic;
use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use std::{fmt::Write, sync::Arc};

#[allow(clippy::too_many_lines)]
#[test]
fn one_callable() {
    let store = PackageStore::new(compile::core());
    let mut compiler = Compiler::new(
        &store,
        &[],
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    let unit = compiler
        .compile_fragments(
            &mut CompileUnit::default(),
            "test_1",
            "namespace Foo { operation Main() : Unit {} }",
            fail_on_error,
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
            node_id:1,node_id:5,node_id:8,
            terms:
            node_id:6,node_id:10,
            locals:
            Locals {
                scopes: [
                    Scope {
                        span: Span {
                            lo: 0,
                            hi: 4294967295,
                        },
                        kind: Block,
                        opens: {},
                        tys: {},
                        terms: {},
                        vars: {},
                        ty_vars: {},
                    },
                    Scope {
                        span: Span {
                            lo: 0,
                            hi: 44,
                        },
                        kind: Namespace(
                            NamespaceId(
                                5,
                            ),
                        ),
                        opens: {
                            []: [
                                Open {
                                    namespace: NamespaceId(
                                        5,
                                    ),
                                    span: Span {
                                        lo: 10,
                                        hi: 13,
                                    },
                                },
                            ],
                        },
                        tys: {},
                        terms: {},
                        vars: {},
                        ty_vars: {},
                    },
                    Scope {
                        span: Span {
                            lo: 16,
                            hi: 42,
                        },
                        kind: Callable,
                        opens: {},
                        tys: {},
                        terms: {},
                        vars: {},
                        ty_vars: {},
                    },
                    Scope {
                        span: Span {
                            lo: 40,
                            hi: 42,
                        },
                        kind: Block,
                        opens: {},
                        tys: {},
                        terms: {},
                        vars: {},
                        ty_vars: {},
                    },
                ],
            }
            hir:
            Package:
                Item 0 [0-44] (Public):
                    Namespace (Ident 5 [10-13] "Foo"): Item 1
                Item 1 [16-42] (Internal):
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
    let mut compiler = Compiler::new(
        &store,
        &[],
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    let unit = compiler
        .compile_fragments(
            &mut CompileUnit::default(),
            "test_1",
            "use q = Qubit();",
            fail_on_error,
        )
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
            locals:
            Locals {
                scopes: [
                    Scope {
                        span: Span {
                            lo: 0,
                            hi: 4294967295,
                        },
                        kind: Block,
                        opens: {},
                        tys: {},
                        terms: {},
                        vars: {
                            "q": (
                                16,
                                NodeId(
                                    3,
                                ),
                            ),
                        },
                        ty_vars: {},
                    },
                ],
            }
            hir:
            Package:
                Stmt 0 [0-16]: Qubit (Fresh)
                    Pat 1 [4-5] [Type Qubit]: Bind: Ident 2 [4-5] "q"
                    QubitInit 3 [8-15] [Type Qubit]: Single"#]],
        &unit,
    );
}

#[test]
fn parse_error() {
    let store = PackageStore::new(compile::core());
    let mut compiler = Compiler::new(
        &store,
        &[],
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    let errors = compiler
        .compile_fragments(&mut CompileUnit::default(), "test_1", "}}", fail_on_error)
        .expect_err("should fail");

    expect![[r#"
        [
            WithSource {
                sources: [
                    Source {
                        name: "test_1",
                        contents: "}}",
                        offset: 0,
                    },
                ],
                error: Error(
                    Parse(
                        Error(
                            Token(
                                Eof,
                                Close(
                                    Brace,
                                ),
                                Span {
                                    lo: 0,
                                    hi: 1,
                                },
                            ),
                        ),
                    ),
                ),
            },
        ]
    "#]]
    .assert_debug_eq(&errors);
}

#[test]
fn conditional_compilation_not_available() {
    let store = PackageStore::new(compile::core());
    let mut compiler = Compiler::new(
        &store,
        &[],
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
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
            fail_on_error,
        )
        .expect_err("should fail");

    assert!(!errors.is_empty());
}

#[test]
fn errors_across_multiple_lines() {
    let mut store = PackageStore::new(compile::core());
    let std_id = store.insert(compile::std(&store, TargetCapabilityFlags::all()));
    let mut compiler = Compiler::new(
        &store,
        &[(std_id, None)],
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    let mut unit = CompileUnit::default();
    compiler
        .compile_fragments(
            &mut unit,
            "line_1",
            "namespace Other { operation DumpMachine() : Unit { } }",
            fail_on_error,
        )
        .expect("should succeed");

    compiler
        .compile_fragments(&mut unit, "line_2", "open Other;", fail_on_error)
        .expect("should succeed");

    compiler
        .compile_fragments(
            &mut unit,
            "line_3",
            "open Microsoft.Quantum.Diagnostics;",
            fail_on_error,
        )
        .expect("should succeed");

    let errors = compiler
        .compile_fragments(&mut unit, "line_4", "DumpMachine()", fail_on_error)
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

#[test]
fn continue_after_parse_error() {
    let store = PackageStore::new(compile::core());
    let mut compiler = Compiler::new(
        &store,
        &Vec::new(),
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    let mut errors = Vec::new();

    compiler
        .compile_fragments(
            &mut CompileUnit::default(),
            "test_1",
            "operation Main() : Foo {
            }}",
            |e| -> Result<(), ()> {
                errors.extend(e);
                Ok(())
            },
        )
        .expect("compile_fragments should succeed");

    expect![[r#"
        [
            WithSource {
                sources: [
                    Source {
                        name: "test_1",
                        contents: "operation Main() : Foo {\n            }}",
                        offset: 0,
                    },
                ],
                error: Error(
                    Parse(
                        Error(
                            Token(
                                Eof,
                                Close(
                                    Brace,
                                ),
                                Span {
                                    lo: 38,
                                    hi: 39,
                                },
                            ),
                        ),
                    ),
                ),
            },
            WithSource {
                sources: [
                    Source {
                        name: "test_1",
                        contents: "operation Main() : Foo {\n            }}",
                        offset: 0,
                    },
                ],
                error: Error(
                    Resolve(
                        NotFound(
                            "Foo",
                            Span {
                                lo: 19,
                                hi: 22,
                            },
                        ),
                    ),
                ),
            },
        ]
    "#]]
    .assert_debug_eq(&errors);
}

#[test]
fn continue_after_lower_error() {
    let store = PackageStore::new(compile::core());
    let mut compiler = Compiler::new(
        &store,
        &[],
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    let mut unit = CompileUnit::default();

    let mut errors = Vec::new();

    compiler
        .compile_fragments(
            &mut unit,
            "test_1",
            "operation A(q : Qubit) : Unit is Adj {
                adjoint ... {}
            }",
            |e| -> Result<(), ()> {
                errors = e;
                Ok(())
            },
        )
        .expect("compile_fragments should succeed");

    expect![[r#"
        [
            WithSource {
                sources: [
                    Source {
                        name: "test_1",
                        contents: "operation A(q : Qubit) : Unit is Adj {\n                adjoint ... {}\n            }",
                        offset: 0,
                    },
                ],
                error: Error(
                    Lower(
                        MissingBody(
                            Span {
                                lo: 0,
                                hi: 83,
                            },
                        ),
                    ),
                ),
            },
        ]
    "#]].assert_debug_eq(&errors);
}
#[test]
fn import_foo() {
    multi_package_test(
        vec![(
            "PackageA.qs",
            indoc! {"
                operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;
            "},
        )],
        vec![(
            "PackageB.qs",
            indoc! {"
                import A.PackageA.Foo;
            "},
        )],
        &[("A", "PackageA")],
        "",
    );
}

#[test]
fn import_foo_with_alias() {
    multi_package_test(
        vec![(
            "PackageA.qs",
            indoc! {"
                operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;
            "},
        )],
        vec![(
            "PackageB.qs",
            indoc! {"
                import A.PackageA.Foo as Foo2;
            "},
        )],
        &[("A", "PackageA")],
        "",
    );
}

#[test]
fn export_foo_with_alias() {
    multi_package_test(
        vec![(
            "PackageA.qs",
            indoc! {"
                operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;
            "},
        )],
        vec![(
            "PackageB.qs",
            indoc! {"
                import A.PackageA.Foo;
                export Foo as Bar;
            "},
        )],
        &[("A", "PackageA")],
        "",
    );
}

#[test]
fn combined_import_export() {
    multi_package_test(
        vec![(
            "PackageA.qs",
            indoc! {"
                operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;
            "},
        )],
        vec![(
            "PackageB.qs",
            indoc! {"
                import A.PackageA.Foo;
                import A.PackageA.Foo as Foo2;
                export Foo, Foo as Bar, Foo2, Foo2 as Bar2;
            "},
        )],
        &[("A", "PackageA")],
        indoc! {"
            import B.PackageB.Foo, B.PackageB.Bar, B.PackageB.Foo2, B.PackageB.Bar2;
            @EntryPoint()
            function Main() : Unit {
                Foo(10, true);
                Foo2(10, true);
                Bar(10, true);
                Bar2(10, true);
            }
        "},
    );
}

#[test]
fn reexport_operation_from_a_dependency() {
    multi_package_test(
        vec![(
            "PackageA.qs",
            indoc! {"
                operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;
            "},
        )],
        vec![(
            "PackageB.qs",
            indoc! {"
                import A.PackageA.Foo;
                export Foo as Bar;
            "},
        )],
        &[("A", "PackageA")],
        indoc! {"
            import B.PackageB.Bar;
            @EntryPoint()
            function Main() : Unit {
                Bar(10, true);
            }
        "},
    );
}

fn multi_package_test(
    packages: Vec<(&str, &str)>,
    dependencies: Vec<(&str, &str)>,
    imports: &[(&str, &str)],
    user_code: &str,
) {
    let mut store = PackageStore::new(compile::core());

    let packages = packages
        .into_iter()
        .map(|(name, code)| {
            let source_map = SourceMap::new([(name.into(), code.into())], None);
            let compiled_package = compile::compile(
                &store,
                &[],
                source_map,
                TargetCapabilityFlags::all(),
                LanguageFeatures::default(),
            );
            assert!(
                compiled_package.errors.is_empty(),
                "{:#?}",
                compiled_package.errors
            );
            store.insert(compiled_package)
        })
        .collect::<Vec<_>>();

    let dependencies = dependencies
        .into_iter()
        .map(|(name, code)| {
            let source_map = SourceMap::new([(name.into(), code.into())], None);
            let compiled_package = compile::compile(
                &store,
                &imports
                    .iter()
                    .map(|(alias, _)| (packages[0], Some(Arc::from(*alias))))
                    .collect::<Vec<_>>(),
                source_map,
                TargetCapabilityFlags::all(),
                LanguageFeatures::default(),
            );
            assert!(
                compiled_package.errors.is_empty(),
                "{:#?}",
                compiled_package.errors
            );
            store.insert(compiled_package)
        })
        .collect::<Vec<_>>();

    let mut compiler = Compiler::new(
        &store,
        &dependencies
            .iter()
            .map(|&pkg| (pkg, Some(Arc::from("B"))))
            .collect::<Vec<_>>(),
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    let mut unit = CompileUnit::default();

    let mut errors = Vec::new();

    compiler
        .compile_fragments(&mut unit, "UserCode", user_code, |e| -> Result<(), ()> {
            errors = e;
            Ok(())
        })
        .expect("compile_fragments should succeed");

    expect!["[]"].assert_eq(&format!("{errors:#?}"));
}
fn check_unit(expect: &Expect, actual: &Increment) {
    let ast = format!("ast:\n{}", actual.ast.package);

    let names = format!(
        "\nnames:\n{}",
        actual
            .ast
            .names
            .iter()
            .fold(String::new(), |mut output, n| {
                let _ = write!(output, "node_id:{},", n.0);
                output
            })
    );
    let terms = format!(
        "\nterms:\n{}",
        actual
            .ast
            .tys
            .terms
            .iter()
            .fold(String::new(), |mut output, n| {
                let _ = write!(output, "node_id:{},", n.0);
                output
            })
    );
    let locals = format!("\nlocals:\n{:#?}", actual.ast.locals);

    let hir = format!("\nhir:\n{}", actual.hir);

    expect.assert_eq(
        &[ast, names, terms, locals, hir]
            .into_iter()
            .collect::<String>(),
    );
}

fn fail_on_error(errors: Vec<Error>) -> Result<(), Vec<Error>> {
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(())
}
