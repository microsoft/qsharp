// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// expect-test updates these strings automatically
#![allow(clippy::needless_raw_string_hashes)]

use super::{CompilationState, CompilationStateUpdater};
use crate::protocol::{DiagnosticUpdate, NotebookMetadata, WorkspaceConfigurationUpdate};
use expect_test::{expect, Expect};
use qsc::{compile::ErrorKind, target::Profile, PackageType};
use qsc_project::{EntryType, JSFileEntry, Manifest, ManifestDescriptor};
use std::{cell::RefCell, fmt::Write, future::ready, rc::Rc, sync::Arc};

#[tokio::test]
async fn no_error() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater
        .update_document(
            "foo.qs",
            1,
            "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
        )
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
        []
    "#]],
    );
}

#[tokio::test]
async fn clear_error() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater.update_document("foo.qs", 1, "namespace {").await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "foo.qs",
                    Some(
                        1,
                    ),
                    [
                        Frontend(
                            Error(
                                Parse(
                                    Error(
                                        Rule(
                                            "identifier",
                                            Open(
                                                Brace,
                                            ),
                                            Span {
                                                lo: 10,
                                                hi: 11,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
    );

    updater
        .update_document(
            "foo.qs",
            2,
            "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
        )
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "foo.qs",
                    Some(
                        2,
                    ),
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn close_last_doc_in_project() {
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors);

    updater
        .update_document(
            "other_file.qs",
            1,
            "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
        )
        .await;
    updater
        .update_document(
            "this_file.qs",
            1,
            "/* this should not show up in the final state */ we should not see compile errors",
        )
        .await;

    updater.close_document("this_file.qs").await;
    // now there should be one compilation and one open document

    check_errors_and_compilation(
        &updater,
        &received_errors,
        &expect![[r#"
            {
                "other_file.qs": OpenDocument {
                    version: 1,
                    compilation: "./qsharp.json",
                    latest_str_content: "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
                },
            }
        "#]],
        &expect![[r#"
            SourceMap {
                sources: [
                    Source {
                        name: "other_file.qs",
                        contents: "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
                        offset: 0,
                    },
                    Source {
                        name: "this_file.qs",
                        contents: "// DISK CONTENTS\n namespace Foo { }",
                        offset: 59,
                    },
                ],
                entry: None,
            }"#]],
        &expect![[r#"
            [
                (
                    "this_file.qs",
                    Some(
                        1,
                    ),
                    [
                        Frontend(
                            Error(
                                Parse(
                                    Error(
                                        Token(
                                            Eof,
                                            ClosedBinOp(
                                                Slash,
                                            ),
                                            Span {
                                                lo: 59,
                                                hi: 60,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ],
                ),
                (
                    "this_file.qs",
                    None,
                    [],
                ),
            ]
        "#]],
    );
    updater.close_document("other_file.qs").await;

    // now there should be no file and no compilation
    check_errors_and_compilation(
        &updater,
        &received_errors,
        &expect![[r#"
            {}
        "#]],
        &expect![""],
        &expect![[r#"
            []
        "#]],
    );
}

#[tokio::test]
async fn clear_on_document_close() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater.update_document("foo.qs", 1, "namespace {").await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "foo.qs",
                    Some(
                        1,
                    ),
                    [
                        Frontend(
                            Error(
                                Parse(
                                    Error(
                                        Rule(
                                            "identifier",
                                            Open(
                                                Brace,
                                            ),
                                            Span {
                                                lo: 10,
                                                hi: 11,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
    );

    updater.close_document("foo.qs").await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "foo.qs",
                    None,
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn compile_error() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater.update_document("foo.qs", 1, "badsyntax").await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "foo.qs",
                    Some(
                        1,
                    ),
                    [
                        Frontend(
                            Error(
                                Parse(
                                    Error(
                                        Token(
                                            Eof,
                                            Ident,
                                            Span {
                                                lo: 0,
                                                hi: 9,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn package_type_update_causes_error() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater.update_configuration(WorkspaceConfigurationUpdate {
        target_profile: None,
        package_type: Some(PackageType::Lib),
    });

    updater
        .update_document("foo.qs", 1, "namespace Foo { operation Main() : Unit {} }")
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            []
    "#]],
    );

    updater.update_configuration(WorkspaceConfigurationUpdate {
        target_profile: None,
        package_type: Some(PackageType::Exe),
    });

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "foo.qs",
                    Some(
                        1,
                    ),
                    [
                        Pass(
                            EntryPoint(
                                NotFound,
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn target_profile_update_fixes_error() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater.update_configuration(WorkspaceConfigurationUpdate {
        target_profile: Some(Profile::Base),
        package_type: Some(PackageType::Lib),
    });

    updater
        .update_document(
            "foo.qs",
            1,
            r#"namespace Foo { operation Main() : Unit { if Zero == Zero { Message("hi") } } }"#,
        )
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "foo.qs",
                    Some(
                        1,
                    ),
                    [
                        Pass(
                            BaseProfCk(
                                ResultComparison(
                                    Span {
                                        lo: 45,
                                        hi: 57,
                                    },
                                ),
                            ),
                        ),
                        Pass(
                            BaseProfCk(
                                ResultLiteral(
                                    Span {
                                        lo: 45,
                                        hi: 49,
                                    },
                                ),
                            ),
                        ),
                        Pass(
                            BaseProfCk(
                                ResultLiteral(
                                    Span {
                                        lo: 53,
                                        hi: 57,
                                    },
                                ),
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
    );

    updater.update_configuration(WorkspaceConfigurationUpdate {
        target_profile: Some(Profile::Unrestricted),
        package_type: None,
    });

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "foo.qs",
                    Some(
                        1,
                    ),
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn target_profile_update_causes_error_in_stdlib() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater.update_document(
        "foo.qs",
        1,
        r#"namespace Foo { @EntryPoint() operation Main() : Unit { use q = Qubit(); let r = M(q); let b = Microsoft.Quantum.Convert.ResultAsBool(r); } }"#,
    ).await;

    expect_errors(
        &errors,
        &expect![[r#"
            []
        "#]],
    );

    updater.update_configuration(WorkspaceConfigurationUpdate {
        target_profile: Some(Profile::Base),
        package_type: None,
    });

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "foo.qs",
                    Some(
                        1,
                    ),
                    [
                        Frontend(
                            Error(
                                Resolve(
                                    NotAvailable(
                                        "ResultAsBool",
                                        "Microsoft.Quantum.Convert.ResultAsBool",
                                        Span {
                                            lo: 121,
                                            hi: 133,
                                        },
                                    ),
                                ),
                            ),
                        ),
                        Frontend(
                            Error(
                                Type(
                                    Error(
                                        AmbiguousTy(
                                            Span {
                                                lo: 95,
                                                hi: 136,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
    );
}

#[test]
fn notebook_document_no_errors() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater.update_notebook_document(
        "notebook.ipynb",
        NotebookMetadata::default(),
        [
            ("cell1", 1, "operation Main() : Unit {}"),
            ("cell2", 1, "Main()"),
        ]
        .into_iter(),
    );

    expect_errors(
        &errors,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn notebook_document_errors() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater.update_notebook_document(
        "notebook.ipynb",
        NotebookMetadata::default(),
        [
            ("cell1", 1, "operation Main() : Unit {}"),
            ("cell2", 1, "Foo()"),
        ]
        .into_iter(),
    );

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "cell2",
                    Some(
                        1,
                    ),
                    [
                        Frontend(
                            Error(
                                Resolve(
                                    NotFound(
                                        "Foo",
                                        Span {
                                            lo: 27,
                                            hi: 30,
                                        },
                                    ),
                                ),
                            ),
                        ),
                        Frontend(
                            Error(
                                Type(
                                    Error(
                                        AmbiguousTy(
                                            Span {
                                                lo: 27,
                                                hi: 32,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
    );
}

#[test]
fn notebook_update_remove_cell_clears_errors() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater.update_notebook_document(
        "notebook.ipynb",
        NotebookMetadata::default(),
        [
            ("cell1", 1, "operation Main() : Unit {}"),
            ("cell2", 1, "Foo()"),
        ]
        .into_iter(),
    );

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "cell2",
                    Some(
                        1,
                    ),
                    [
                        Frontend(
                            Error(
                                Resolve(
                                    NotFound(
                                        "Foo",
                                        Span {
                                            lo: 27,
                                            hi: 30,
                                        },
                                    ),
                                ),
                            ),
                        ),
                        Frontend(
                            Error(
                                Type(
                                    Error(
                                        AmbiguousTy(
                                            Span {
                                                lo: 27,
                                                hi: 32,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
    );

    updater.update_notebook_document(
        "notebook.ipynb",
        NotebookMetadata::default(),
        [("cell1", 1, "operation Main() : Unit {}")].into_iter(),
    );

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "cell2",
                    None,
                    [],
                ),
            ]
        "#]],
    );
}

#[test]
fn close_notebook_clears_errors() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater.update_notebook_document(
        "notebook.ipynb",
        NotebookMetadata::default(),
        [
            ("cell1", 1, "operation Main() : Unit {}"),
            ("cell2", 1, "Foo()"),
        ]
        .into_iter(),
    );

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "cell2",
                    Some(
                        1,
                    ),
                    [
                        Frontend(
                            Error(
                                Resolve(
                                    NotFound(
                                        "Foo",
                                        Span {
                                            lo: 27,
                                            hi: 30,
                                        },
                                    ),
                                ),
                            ),
                        ),
                        Frontend(
                            Error(
                                Type(
                                    Error(
                                        AmbiguousTy(
                                            Span {
                                                lo: 27,
                                                hi: 32,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
    );

    updater.close_notebook_document("notebook.ipynb");

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "cell2",
                    None,
                    [],
                ),
            ]
        "#]],
    );
}

#[allow(clippy::too_many_lines)]
#[tokio::test]
async fn update_doc_updates_project() {
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors);

    updater
        .update_document(
            "other_file.qs",
            1,
            "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
        )
        .await;
    updater
        .update_document(
            "this_file.qs",
            1,
            "namespace Foo { we should see this in the source }",
        )
        .await;

    check_errors_and_compilation(
        &updater,
        &received_errors,
        &expect![[r#"
            {
                "other_file.qs": OpenDocument {
                    version: 1,
                    compilation: "./qsharp.json",
                    latest_str_content: "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
                },
                "this_file.qs": OpenDocument {
                    version: 1,
                    compilation: "./qsharp.json",
                    latest_str_content: "namespace Foo { we should see this in the source }",
                },
            }
        "#]],
        &expect![[r#"
            SourceMap {
                sources: [
                    Source {
                        name: "other_file.qs",
                        contents: "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
                        offset: 0,
                    },
                    Source {
                        name: "this_file.qs",
                        contents: "namespace Foo { we should see this in the source }",
                        offset: 59,
                    },
                ],
                entry: None,
            }"#]],
        &expect![[r#"
            [
                (
                    "this_file.qs",
                    Some(
                        1,
                    ),
                    [
                        Frontend(
                            Error(
                                Parse(
                                    Error(
                                        Token(
                                            Close(
                                                Brace,
                                            ),
                                            Ident,
                                            Span {
                                                lo: 75,
                                                hi: 77,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
    );
}

/// In this test, we:
/// open a project
/// update a buffer in the LS
/// close that buffer
/// assert that the LS no longer prioritizes that open buffer
/// over the FS
#[allow(clippy::too_many_lines)]
#[tokio::test]
async fn close_doc_prioritizes_fs() {
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors);

    updater
        .update_document(
            "other_file.qs",
            1,
            "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
        )
        .await;
    updater
        .update_document(
            "this_file.qs",
            1,
            "/* this should not show up in the final state */ we should not see compile errors",
        )
        .await;

    updater.close_document("this_file.qs").await;

    check_errors_and_compilation(
        &updater,
        &received_errors,
        &expect![[r#"
            {
                "other_file.qs": OpenDocument {
                    version: 1,
                    compilation: "./qsharp.json",
                    latest_str_content: "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
                },
            }
        "#]],
        &expect![[r#"
            SourceMap {
                sources: [
                    Source {
                        name: "other_file.qs",
                        contents: "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
                        offset: 0,
                    },
                    Source {
                        name: "this_file.qs",
                        contents: "// DISK CONTENTS\n namespace Foo { }",
                        offset: 59,
                    },
                ],
                entry: None,
            }"#]],
        &expect![[r#"
            [
                (
                    "this_file.qs",
                    Some(
                        1,
                    ),
                    [
                        Frontend(
                            Error(
                                Parse(
                                    Error(
                                        Token(
                                            Eof,
                                            ClosedBinOp(
                                                Slash,
                                            ),
                                            Span {
                                                lo: 59,
                                                hi: 60,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ],
                ),
                (
                    "this_file.qs",
                    None,
                    [],
                ),
            ]
        "#]],
    );
}
type ErrorInfo = (String, Option<u32>, Vec<ErrorKind>);

fn new_updater(received_errors: &RefCell<Vec<ErrorInfo>>) -> CompilationStateUpdater<'_> {
    let diagnostic_receiver = move |update: DiagnosticUpdate| {
        let mut v = received_errors.borrow_mut();

        v.push((
            update.uri.to_string(),
            update.version,
            update
                .errors
                .iter()
                .map(|e| e.error().clone())
                .collect::<Vec<_>>(),
        ));
    };

    CompilationStateUpdater::new(
        Rc::new(RefCell::new(CompilationState::default())),
        diagnostic_receiver,
        |file| {
            Box::pin(async {
                tokio::spawn(ready(match file.as_str() {
                    "other_file.qs" => (
                        Arc::from(file),
                        Arc::from("// DISK CONTENTS\n namespace OtherFile { operation Other() : Unit {} }"),
                    ),
                    "this_file.qs" => (Arc::from(file), Arc::from("// DISK CONTENTS\n namespace Foo { }")),
                    _ => panic!("unknown file"),
                }))
                .await
                .expect("spawn should not fail")
            })
        },
        |dir_name| {
            Box::pin(async move {
                tokio::spawn(ready({
                    vec![
                        JSFileEntry {
                            name: "src".into(),
                            r#type: (if dir_name.as_str() == "src" {
                                EntryType::File
                            } else {
                                EntryType::Folder
                            }),
                        },
                        JSFileEntry {
                            name: "other_file.qs".into(),
                            r#type: EntryType::File,
                        },
                        JSFileEntry {
                            name: "this_file.qs".into(),
                            r#type: EntryType::File,
                        },
                    ]
                }))
                .await
                .expect("spawn should not fail")
            })
        },
        |file| {
            Box::pin(async move {
                tokio::spawn(ready(match file.as_str() {
                    "other_file.qs" | "this_file.qs" => Some(ManifestDescriptor {
                        manifest: Manifest::default(),
                        manifest_dir: ".".into(),
                    }),
                    "foo.qs" => None,
                    _ => panic!("unknown file"),
                }))
                .await
                .expect("spawn should not fail")
            })
        },
    )
}

fn expect_errors(errors: &RefCell<Vec<ErrorInfo>>, expected: &Expect) {
    expected.assert_debug_eq(&errors.borrow());
    // reset accumulated errors after each check
    errors.borrow_mut().clear();
}

fn assert_compilation_sources(updater: &CompilationStateUpdater<'_>, expected: &Expect) {
    let state = updater.state.try_borrow().expect("borrow should succeed");

    let compilation_sources = state
        .compilations
        .values()
        .fold(String::new(), |mut output, c| {
            let _ = write!(output, "{:#?}", c.0.user_unit().sources);
            output
        });
    expected.assert_eq(&compilation_sources);
}

fn assert_open_documents(updater: &CompilationStateUpdater<'_>, expected: &Expect) {
    let state = updater.state.try_borrow().expect("borrow should succeed");
    expected.assert_debug_eq(&state.open_documents);
}

fn check_errors_and_compilation(
    updater: &CompilationStateUpdater<'_>,
    received_errors: &RefCell<Vec<ErrorInfo>>,
    expected_open_documents: &Expect,
    expected_compilation_sources: &Expect,
    expected_errors: &Expect,
) {
    assert_open_documents(updater, expected_open_documents);
    assert_compilation_sources(updater, expected_compilation_sources);
    expect_errors(received_errors, expected_errors);
}
