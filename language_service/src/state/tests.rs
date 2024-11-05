// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// expect-test updates these strings automatically
#![allow(clippy::needless_raw_string_hashes, clippy::too_many_lines)]

use super::{CompilationState, CompilationStateUpdater};
use crate::{
    protocol::{DiagnosticUpdate, ErrorKind, NotebookMetadata, WorkspaceConfigurationUpdate},
    tests::test_fs::{dir, file, FsNode, TestProjectHost},
};
use expect_test::{expect, Expect};
use qsc::{compile, project, target::Profile, LanguageFeatures, PackageType};
use qsc_linter::{AstLint, LintConfig, LintKind, LintLevel};
use std::{cell::RefCell, fmt::Write, rc::Rc};

#[tokio::test]
async fn no_error() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater
        .update_document(
            "single/foo.qs",
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

    updater
        .update_document("single/foo.qs", 1, "namespace {")
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "single/foo.qs",
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
                    [],
                ),
            ]
        "#]],
    );

    updater
        .update_document(
            "single/foo.qs",
            2,
            "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
        )
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "single/foo.qs",
                    Some(
                        2,
                    ),
                    [],
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn close_last_doc_in_project() {
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors);

    updater
        .update_document(
            "project/src/other_file.qs",
            1,
            "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
        )
        .await;
    updater
        .update_document(
            "project/src/this_file.qs",
            1,
            "/* this should not show up in the final state */ we should not see compile errors",
        )
        .await;

    updater.close_document("project/src/this_file.qs").await;
    // now there should be one compilation and one open document

    check_state_and_errors(
        &updater,
        &received_errors,
        &expect![[r#"
            {
                "project/src/other_file.qs": OpenDocument {
                    version: 1,
                    compilation: "project/qsharp.json",
                    latest_str_content: "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
                },
            }
        "#]],
        &expect![[r#"
            project/qsharp.json: SourceMap {
                sources: [
                    Source {
                        name: "project/src/other_file.qs",
                        contents: "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
                        offset: 0,
                    },
                    Source {
                        name: "project/src/this_file.qs",
                        contents: "// DISK CONTENTS\n namespace Foo { }",
                        offset: 59,
                    },
                ],
                common_prefix: Some(
                    "project/src/",
                ),
                entry: None,
            }
        "#]],
        &expect![[r#"
            [
                (
                    "project/src/this_file.qs",
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
                    [],
                ),
                (
                    "project/src/this_file.qs",
                    None,
                    [],
                    [],
                ),
            ]
        "#]],
    );
    updater.close_document("project/src/other_file.qs").await;

    // now there should be no file and no compilation
    check_state_and_errors(
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

    updater
        .update_document("single/foo.qs", 1, "namespace {")
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "single/foo.qs",
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
                    [],
                ),
            ]
        "#]],
    );

    updater.close_document("single/foo.qs").await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "single/foo.qs",
                    None,
                    [],
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

    updater
        .update_document("single/foo.qs", 1, "badsyntax")
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "single/foo.qs",
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
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn rca_errors_are_reported_when_compilation_succeeds() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater.update_configuration(WorkspaceConfigurationUpdate {
        target_profile: Some(Profile::AdaptiveRI),
        package_type: Some(PackageType::Lib),
        ..WorkspaceConfigurationUpdate::default()
    });

    updater
        .update_document("single/foo.qs", 1, "namespace Test { operation RcaCheck() : Double { use q = Qubit(); mutable x = 1.0; if MResetZ(q) == One { set x = 2.0; } x } }")
        .await;

    // we expect two errors, one for `set x = 2` and one for `x`
    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "single/foo.qs",
                    Some(
                        1,
                    ),
                    [
                        Pass(
                            CapabilitiesCk(
                                UseOfDynamicDouble(
                                    Span {
                                        lo: 106,
                                        hi: 117,
                                    },
                                ),
                            ),
                        ),
                        Pass(
                            CapabilitiesCk(
                                UseOfDynamicDouble(
                                    Span {
                                        lo: 121,
                                        hi: 122,
                                    },
                                ),
                            ),
                        ),
                    ],
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn base_profile_rca_errors_are_reported_when_compilation_succeeds() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater.update_configuration(WorkspaceConfigurationUpdate {
        target_profile: Some(Profile::Base),
        package_type: Some(PackageType::Lib),
        ..WorkspaceConfigurationUpdate::default()
    });

    updater
        .update_document("single/foo.qs", 1, "namespace Test { operation RcaCheck() : Double { use q = Qubit(); mutable x = 1.0; if MResetZ(q) == One { set x = 2.0; } x } }")
        .await;

    // we expect two errors, one for `set x = 2.0` and one for `x`
    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "single/foo.qs",
                    Some(
                        1,
                    ),
                    [
                        Pass(
                            CapabilitiesCk(
                                UseOfDynamicBool(
                                    Span {
                                        lo: 86,
                                        hi: 103,
                                    },
                                ),
                            ),
                        ),
                        Pass(
                            CapabilitiesCk(
                                UseOfDynamicDouble(
                                    Span {
                                        lo: 106,
                                        hi: 117,
                                    },
                                ),
                            ),
                        ),
                        Pass(
                            CapabilitiesCk(
                                UseOfDynamicDouble(
                                    Span {
                                        lo: 121,
                                        hi: 122,
                                    },
                                ),
                            ),
                        ),
                    ],
                    [],
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
        package_type: Some(PackageType::Lib),
        ..WorkspaceConfigurationUpdate::default()
    });

    updater
        .update_document(
            "single/foo.qs",
            1,
            "namespace Foo { operation Test() : Unit {} }",
        )
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            []
    "#]],
    );

    updater.update_configuration(WorkspaceConfigurationUpdate {
        package_type: Some(PackageType::Exe),
        ..WorkspaceConfigurationUpdate::default()
    });

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "single/foo.qs",
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
                    [],
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
        ..WorkspaceConfigurationUpdate::default()
    });

    updater
        .update_document(
            "single/foo.qs",
            1,
            r#"namespace Foo { operation Main() : Unit { use q = Qubit(); if M(q) == Zero { Message("hi") } } }"#,
        )
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "single/foo.qs",
                    Some(
                        1,
                    ),
                    [
                        Pass(
                            CapabilitiesCk(
                                UseOfDynamicBool(
                                    Span {
                                        lo: 62,
                                        hi: 74,
                                    },
                                ),
                            ),
                        ),
                    ],
                    [],
                ),
            ]
        "#]],
    );

    updater.update_configuration(WorkspaceConfigurationUpdate {
        target_profile: Some(Profile::Unrestricted),
        ..WorkspaceConfigurationUpdate::default()
    });

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "single/foo.qs",
                    Some(
                        1,
                    ),
                    [],
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
        "single/foo.qs",
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
        ..WorkspaceConfigurationUpdate::default()
    });

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "single/foo.qs",
                    Some(
                        1,
                    ),
                    [
                        Pass(
                            CapabilitiesCk(
                                UseOfDynamicBool(
                                    Span {
                                        lo: 95,
                                        hi: 136,
                                    },
                                ),
                            ),
                        ),
                    ],
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn notebook_document_no_errors() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater
        .update_notebook_document(
            "notebook.ipynb",
            &NotebookMetadata::default(),
            [
                ("cell1", 1, "operation Main() : Unit {}"),
                ("cell2", 1, "Main()"),
            ]
            .into_iter(),
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
async fn notebook_document_errors() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater
        .update_notebook_document(
            "notebook.ipynb",
            &NotebookMetadata::default(),
            [
                ("cell1", 1, "operation Main() : Unit {}"),
                ("cell2", 1, "Foo()"),
            ]
            .into_iter(),
        )
        .await;

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
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn notebook_document_lints() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater
        .update_notebook_document(
            "notebook.ipynb",
            &NotebookMetadata::default(),
            [
                ("cell1", 1, "function Foo() : Unit { let x = 4;;;; }"),
                ("cell2", 1, "function Bar() : Unit { let y = 5 / 0; }"),
            ]
            .into_iter(),
        )
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "cell1",
                    Some(
                        1,
                    ),
                    [
                        Lint(
                            Lint {
                                span: Span {
                                    lo: 34,
                                    hi: 37,
                                },
                                level: Warn,
                                message: "redundant semicolons",
                                help: "remove the redundant semicolons",
                                kind: Ast(
                                    RedundantSemicolons,
                                ),
                                code_action_edits: [
                                    (
                                        "",
                                        Span {
                                            lo: 34,
                                            hi: 37,
                                        },
                                    ),
                                ],
                            },
                        ),
                    ],
                    [],
                ),
                (
                    "cell2",
                    Some(
                        1,
                    ),
                    [
                        Lint(
                            Lint {
                                span: Span {
                                    lo: 72,
                                    hi: 77,
                                },
                                level: Error,
                                message: "attempt to divide by zero",
                                help: "division by zero will fail at runtime",
                                kind: Ast(
                                    DivisionByZero,
                                ),
                                code_action_edits: [],
                            },
                        ),
                    ],
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn notebook_update_remove_cell_clears_errors() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater
        .update_notebook_document(
            "notebook.ipynb",
            &NotebookMetadata::default(),
            [
                ("cell1", 1, "operation Main() : Unit {}"),
                ("cell2", 1, "Foo()"),
            ]
            .into_iter(),
        )
        .await;

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
                    [],
                ),
            ]
        "#]],
    );

    updater
        .update_notebook_document(
            "notebook.ipynb",
            &NotebookMetadata::default(),
            [("cell1", 1, "operation Main() : Unit {}")].into_iter(),
        )
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "cell2",
                    None,
                    [],
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn close_notebook_clears_errors() {
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors);

    updater
        .update_notebook_document(
            "notebook.ipynb",
            &NotebookMetadata::default(),
            [
                ("cell1", 1, "operation Main() : Unit {}"),
                ("cell2", 1, "Foo()"),
            ]
            .into_iter(),
        )
        .await;

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
                    [],
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
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn update_notebook_with_valid_dependencies() {
    let fs = FsNode::Dir(
        [dir(
            "project",
            [
                file("qsharp.json", r#"{ }"#),
                dir(
                    "src",
                    [file(
                        "file.qs",
                        r#"namespace Foo { function Bar() : Unit { } }"#,
                    )],
                ),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&errors, &fs);

    updater
        .update_notebook_document(
            "notebook.ipynb",
            &NotebookMetadata {
                target_profile: None,
                language_features: LanguageFeatures::default(),
                manifest: None,
                project_root: Some("project".to_string()),
            },
            [("cell1", 1, "open Foo;Bar();")].into_iter(),
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
async fn update_notebook_reports_errors_from_dependencies() {
    let fs = FsNode::Dir(
        [dir(
            "project",
            [
                file("qsharp.json", r#"{ }"#),
                dir(
                    "src",
                    [file(
                        "file.qs",
                        r#"namespace Foo { function Bar() : Int { } }"#,
                    )],
                ),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&errors, &fs);

    updater
        .update_notebook_document(
            "notebook.ipynb",
            &NotebookMetadata {
                target_profile: None,
                language_features: LanguageFeatures::default(),
                manifest: None,
                project_root: Some("project".to_string()),
            },
            [("cell1", 1, "open Foo;Bar();")].into_iter(),
        )
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
                (
                    "cell1",
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
                                            lo: 5,
                                            hi: 8,
                                        },
                                    ),
                                ),
                            ),
                        ),
                        Frontend(
                            Error(
                                Resolve(
                                    NotFound(
                                        "Foo",
                                        Span {
                                            lo: 5,
                                            hi: 8,
                                        },
                                    ),
                                ),
                            ),
                        ),
                        Frontend(
                            Error(
                                Resolve(
                                    NotFound(
                                        "Bar",
                                        Span {
                                            lo: 9,
                                            hi: 12,
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
                                                lo: 9,
                                                hi: 14,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ],
                    [],
                ),
                (
                    "project/src/file.qs",
                    None,
                    [
                        Frontend(
                            Error(
                                Type(
                                    Error(
                                        TyMismatch(
                                            "Unit",
                                            "Int",
                                            Span {
                                                lo: 33,
                                                hi: 36,
                                            },
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ],
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn update_notebook_reports_errors_from_dependency_of_dependencies() {
    let fs = FsNode::Dir(
        [
            dir(
                "project",
                [
                    file(
                        "qsharp.json",
                        r#"{ "dependencies" : { "MyDep" : { "path" : "../project2" } } }"#,
                    ),
                    dir(
                        "src",
                        [file(
                            "file.qs",
                            r#"namespace Foo { function Bar() : Unit { } }"#,
                        )],
                    ),
                ],
            ),
            dir(
                "project2",
                [
                    file("qsharp.json", r#"{ }"#),
                    dir(
                        "src",
                        [file(
                            "file.qs",
                            r#"namespace Foo { function Baz() : Int { } }"#,
                        )],
                    ),
                ],
            ),
        ]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let errors = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&errors, &fs);

    updater
        .update_notebook_document(
            "notebook.ipynb",
            &NotebookMetadata {
                target_profile: None,
                language_features: LanguageFeatures::default(),
                manifest: None,
                project_root: Some("project".to_string()),
            },
            [("cell1", 1, "open Foo;Bar();")].into_iter(),
        )
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
        [
            (
                "project2/src/file.qs",
                None,
                [
                    Frontend(
                        Error(
                            Type(
                                Error(
                                    TyMismatch(
                                        "Unit",
                                        "Int",
                                        Span {
                                            lo: 33,
                                            hi: 36,
                                        },
                                    ),
                                ),
                            ),
                        ),
                    ),
                ],
                [],
            ),
        ]
    "#]],
    );
}

#[tokio::test]
async fn update_doc_updates_project() {
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors);

    updater
        .update_document(
            "project/src/other_file.qs",
            1,
            "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
        )
        .await;
    updater
        .update_document(
            "project/src/this_file.qs",
            1,
            "namespace Foo { we should see this in the source }",
        )
        .await;

    check_state_and_errors(
        &updater,
        &received_errors,
        &expect![[r#"
            {
                "project/src/this_file.qs": OpenDocument {
                    version: 1,
                    compilation: "project/qsharp.json",
                    latest_str_content: "namespace Foo { we should see this in the source }",
                },
                "project/src/other_file.qs": OpenDocument {
                    version: 1,
                    compilation: "project/qsharp.json",
                    latest_str_content: "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
                },
            }
        "#]],
        &expect![[r#"
            project/qsharp.json: SourceMap {
                sources: [
                    Source {
                        name: "project/src/other_file.qs",
                        contents: "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
                        offset: 0,
                    },
                    Source {
                        name: "project/src/this_file.qs",
                        contents: "namespace Foo { we should see this in the source }",
                        offset: 59,
                    },
                ],
                common_prefix: Some(
                    "project/src/",
                ),
                entry: None,
            }
        "#]],
        &expect![[r#"
            [
                (
                    "project/src/this_file.qs",
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
                    [],
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
#[tokio::test]
async fn close_doc_prioritizes_fs() {
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors);

    updater
        .update_document(
            "project/src/other_file.qs",
            1,
            "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
        )
        .await;
    updater
        .update_document(
            "project/src/this_file.qs",
            1,
            "/* this should not show up in the final state */ we should not see compile errors",
        )
        .await;

    updater.close_document("project/src/this_file.qs").await;

    check_state_and_errors(
        &updater,
        &received_errors,
        &expect![[r#"
            {
                "project/src/other_file.qs": OpenDocument {
                    version: 1,
                    compilation: "project/qsharp.json",
                    latest_str_content: "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
                },
            }
        "#]],
        &expect![[r#"
            project/qsharp.json: SourceMap {
                sources: [
                    Source {
                        name: "project/src/other_file.qs",
                        contents: "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
                        offset: 0,
                    },
                    Source {
                        name: "project/src/this_file.qs",
                        contents: "// DISK CONTENTS\n namespace Foo { }",
                        offset: 59,
                    },
                ],
                common_prefix: Some(
                    "project/src/",
                ),
                entry: None,
            }
        "#]],
        &expect![[r#"
            [
                (
                    "project/src/this_file.qs",
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
                    [],
                ),
                (
                    "project/src/this_file.qs",
                    None,
                    [],
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn delete_manifest() {
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors);

    updater
        .update_document(
            "project/src/this_file.qs",
            1,
            "// DISK CONTENTS\n namespace Foo { }",
        )
        .await;

    check_state(
        &updater,
        &expect![[r#"
            {
                "project/src/this_file.qs": OpenDocument {
                    version: 1,
                    compilation: "project/qsharp.json",
                    latest_str_content: "// DISK CONTENTS\n namespace Foo { }",
                },
            }
        "#]],
        &expect![[r#"
            project/qsharp.json: SourceMap {
                sources: [
                    Source {
                        name: "project/src/other_file.qs",
                        contents: "// DISK CONTENTS\n namespace OtherFile { operation Other() : Unit { } }",
                        offset: 0,
                    },
                    Source {
                        name: "project/src/this_file.qs",
                        contents: "// DISK CONTENTS\n namespace Foo { }",
                        offset: 71,
                    },
                ],
                common_prefix: Some(
                    "project/src/",
                ),
                entry: None,
            }
        "#]],
    );

    TEST_FS.with(|fs| fs.borrow_mut().remove("project/qsharp.json"));

    updater
        .update_document(
            "project/src/this_file.qs",
            2,
            "// DISK CONTENTS\n namespace Foo { }",
        )
        .await;

    check_state(
        &updater,
        &expect![[r#"
            {
                "project/src/this_file.qs": OpenDocument {
                    version: 2,
                    compilation: "project/src/this_file.qs",
                    latest_str_content: "// DISK CONTENTS\n namespace Foo { }",
                },
            }
        "#]],
        &expect![[r#"
            project/src/this_file.qs: SourceMap {
                sources: [
                    Source {
                        name: "project/src/this_file.qs",
                        contents: "// DISK CONTENTS\n namespace Foo { }",
                        offset: 0,
                    },
                ],
                common_prefix: Some(
                    "project/src/",
                ),
                entry: None,
            }
        "#]],
    );
}

#[tokio::test]
async fn delete_manifest_then_close() {
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors);

    updater
        .update_document(
            "project/src/this_file.qs",
            1,
            "// DISK CONTENTS\n namespace Foo { }",
        )
        .await;

    check_state(
        &updater,
        &expect![[r#"
            {
                "project/src/this_file.qs": OpenDocument {
                    version: 1,
                    compilation: "project/qsharp.json",
                    latest_str_content: "// DISK CONTENTS\n namespace Foo { }",
                },
            }
        "#]],
        &expect![[r#"
            project/qsharp.json: SourceMap {
                sources: [
                    Source {
                        name: "project/src/other_file.qs",
                        contents: "// DISK CONTENTS\n namespace OtherFile { operation Other() : Unit { } }",
                        offset: 0,
                    },
                    Source {
                        name: "project/src/this_file.qs",
                        contents: "// DISK CONTENTS\n namespace Foo { }",
                        offset: 71,
                    },
                ],
                common_prefix: Some(
                    "project/src/",
                ),
                entry: None,
            }
        "#]],
    );

    TEST_FS.with(|fs| fs.borrow_mut().remove("project/qsharp.json"));

    updater.close_document("project/src/this_file.qs").await;

    check_state(
        &updater,
        &expect![[r#"
            {}
        "#]],
        &expect![""],
    );
}

#[tokio::test]
async fn doc_switches_project() {
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors);

    updater
        .update_document("nested_projects/src/subdir/src/a.qs", 1, "namespace A {}")
        .await;

    updater
        .update_document("nested_projects/src/subdir/src/b.qs", 1, "namespace B {}")
        .await;

    check_state(
        &updater,
        &expect![[r#"
            {
                "nested_projects/src/subdir/src/a.qs": OpenDocument {
                    version: 1,
                    compilation: "nested_projects/src/subdir/qsharp.json",
                    latest_str_content: "namespace A {}",
                },
                "nested_projects/src/subdir/src/b.qs": OpenDocument {
                    version: 1,
                    compilation: "nested_projects/src/subdir/qsharp.json",
                    latest_str_content: "namespace B {}",
                },
            }
        "#]],
        &expect![[r#"
            nested_projects/src/subdir/qsharp.json: SourceMap {
                sources: [
                    Source {
                        name: "nested_projects/src/subdir/src/a.qs",
                        contents: "namespace A {}",
                        offset: 0,
                    },
                    Source {
                        name: "nested_projects/src/subdir/src/b.qs",
                        contents: "namespace B {}",
                        offset: 15,
                    },
                ],
                common_prefix: Some(
                    "nested_projects/src/subdir/src/",
                ),
                entry: None,
            }
        "#]],
    );

    // This is just a trick to cause the file to move between projects.
    // Deleting subdir/qsharp.json will cause subdir/a.qs to be picked up
    // by the parent directory's qsharp.json
    TEST_FS.with(|fs| {
        fs.borrow_mut()
            .remove("nested_projects/src/subdir/qsharp.json");
    });

    updater
        .update_document("nested_projects/src/subdir/src/a.qs", 2, "namespace A {}")
        .await;

    updater
        .update_document("nested_projects/src/subdir/src/b.qs", 2, "namespace B {}")
        .await;

    // the error should now be coming from the parent qsharp.json? But the document
    // is closed........
    check_state(
        &updater,
        &expect![[r#"
            {
                "nested_projects/src/subdir/src/a.qs": OpenDocument {
                    version: 2,
                    compilation: "nested_projects/qsharp.json",
                    latest_str_content: "namespace A {}",
                },
                "nested_projects/src/subdir/src/b.qs": OpenDocument {
                    version: 2,
                    compilation: "nested_projects/qsharp.json",
                    latest_str_content: "namespace B {}",
                },
            }
        "#]],
        &expect![[r#"
            nested_projects/qsharp.json: SourceMap {
                sources: [
                    Source {
                        name: "nested_projects/src/subdir/src/a.qs",
                        contents: "namespace A {}",
                        offset: 0,
                    },
                    Source {
                        name: "nested_projects/src/subdir/src/b.qs",
                        contents: "namespace B {}",
                        offset: 15,
                    },
                ],
                common_prefix: Some(
                    "nested_projects/src/subdir/src/",
                ),
                entry: None,
            }
        "#]],
    );
}

#[tokio::test]
async fn doc_switches_project_on_close() {
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors);

    updater
        .update_document("nested_projects/src/subdir/src/a.qs", 1, "namespace A {}")
        .await;

    updater
        .update_document("nested_projects/src/subdir/src/b.qs", 1, "namespace B {}")
        .await;

    check_state(
        &updater,
        &expect![[r#"
            {
                "nested_projects/src/subdir/src/a.qs": OpenDocument {
                    version: 1,
                    compilation: "nested_projects/src/subdir/qsharp.json",
                    latest_str_content: "namespace A {}",
                },
                "nested_projects/src/subdir/src/b.qs": OpenDocument {
                    version: 1,
                    compilation: "nested_projects/src/subdir/qsharp.json",
                    latest_str_content: "namespace B {}",
                },
            }
        "#]],
        &expect![[r#"
            nested_projects/src/subdir/qsharp.json: SourceMap {
                sources: [
                    Source {
                        name: "nested_projects/src/subdir/src/a.qs",
                        contents: "namespace A {}",
                        offset: 0,
                    },
                    Source {
                        name: "nested_projects/src/subdir/src/b.qs",
                        contents: "namespace B {}",
                        offset: 15,
                    },
                ],
                common_prefix: Some(
                    "nested_projects/src/subdir/src/",
                ),
                entry: None,
            }
        "#]],
    );

    // This is just a trick to cause the file to move between projects.
    // Deleting subdir/qsharp.json will cause subdir/src/a.qs to be picked up
    // by the parent directory's qsharp.json
    TEST_FS.with(|fs| {
        fs.borrow_mut()
            .remove("nested_projects/src/subdir/qsharp.json");
    });

    updater
        .close_document("nested_projects/src/subdir/src/a.qs")
        .await;

    updater
        .update_document("nested_projects/src/subdir/src/b.qs", 2, "namespace B {}")
        .await;

    check_state(
        &updater,
        &expect![[r#"
            {
                "nested_projects/src/subdir/src/b.qs": OpenDocument {
                    version: 2,
                    compilation: "nested_projects/qsharp.json",
                    latest_str_content: "namespace B {}",
                },
            }
        "#]],
        &expect![[r#"
            nested_projects/qsharp.json: SourceMap {
                sources: [
                    Source {
                        name: "nested_projects/src/subdir/src/a.qs",
                        contents: "namespace A {}",
                        offset: 0,
                    },
                    Source {
                        name: "nested_projects/src/subdir/src/b.qs",
                        contents: "namespace B {}",
                        offset: 15,
                    },
                ],
                common_prefix: Some(
                    "nested_projects/src/subdir/src/",
                ),
                entry: None,
            }
        "#]],
    );
}

#[tokio::test]
async fn loading_lints_config_from_manifest() {
    let this_file_qs = "namespace Foo { operation Main() : Unit { let x = 5 / 0 + (2 ^ 4); } }";
    let fs = FsNode::Dir(
        [dir(
            "project",
            [
                file(
                    "qsharp.json",
                    r#"{ "lints": [{ "lint": "divisionByZero", "level": "error" }, { "lint": "needlessParens", "level": "error" }] }"#,
                ),
                dir(
                    "src",
                    [file(
                        "this_file.qs",
                        this_file_qs,
                    )],
                ),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let received_errors = RefCell::new(Vec::new());
    let updater = new_updater_with_file_system(&received_errors, &fs);

    // Check the LintConfig.
    check_lints_config(
        &updater,
        &expect![[r#"
            [
                LintConfig {
                    kind: Ast(
                        DivisionByZero,
                    ),
                    level: Error,
                },
                LintConfig {
                    kind: Ast(
                        NeedlessParens,
                    ),
                    level: Error,
                },
            ]"#]],
    )
    .await;
}

#[allow(clippy::too_many_lines)]
#[tokio::test]
async fn lints_update_after_manifest_change() {
    let this_file_qs =
        "namespace Foo { @EntryPoint() function Main() : Unit { let x = 5 / 0 + (2 ^ 4); } }";
    let fs = FsNode::Dir(
        [dir(
            "project",
            [
                file(
                    "qsharp.json",
                    r#"{ "lints": [{ "lint": "divisionByZero", "level": "error" }, { "lint": "needlessParens", "level": "error" }] }"#,
                ),
                dir(
                    "src",
                    [file(
                        "this_file.qs",
                        this_file_qs,
                    )],
                ),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &fs);

    // Trigger a document update.
    updater
        .update_document("project/src/this_file.qs", 1, this_file_qs)
        .await;

    // Check generated lints.
    let lints: &[compile::ErrorKind] = &received_errors.take()[0].2;
    check_lints(
        lints,
        &expect![[r#"
            [
                Lint(
                    Lint {
                        span: Span {
                            lo: 71,
                            hi: 78,
                        },
                        level: Error,
                        message: "unnecessary parentheses",
                        help: "remove the extra parentheses for clarity",
                        kind: Ast(
                            NeedlessParens,
                        ),
                        code_action_edits: [
                            (
                                "",
                                Span {
                                    lo: 71,
                                    hi: 72,
                                },
                            ),
                            (
                                "",
                                Span {
                                    lo: 77,
                                    hi: 78,
                                },
                            ),
                        ],
                    },
                ),
                Lint(
                    Lint {
                        span: Span {
                            lo: 63,
                            hi: 68,
                        },
                        level: Error,
                        message: "attempt to divide by zero",
                        help: "division by zero will fail at runtime",
                        kind: Ast(
                            DivisionByZero,
                        ),
                        code_action_edits: [],
                    },
                ),
            ]"#]],
    );

    // Modify the manifest.
    fs
        .borrow_mut()
        .write_file("project/qsharp.json", r#"{ "lints": [{ "lint": "divisionByZero", "level": "warn" }, { "lint": "needlessParens", "level": "warn" }] }"#)
        .expect("qsharp.json should exist");

    // Trigger a document update
    updater
        .update_document("project/src/this_file.qs", 1, this_file_qs)
        .await;

    // Check lints again
    let lints: &[compile::ErrorKind] = &received_errors.take()[0].2;
    check_lints(
        lints,
        &expect![[r#"
            [
                Lint(
                    Lint {
                        span: Span {
                            lo: 71,
                            hi: 78,
                        },
                        level: Warn,
                        message: "unnecessary parentheses",
                        help: "remove the extra parentheses for clarity",
                        kind: Ast(
                            NeedlessParens,
                        ),
                        code_action_edits: [
                            (
                                "",
                                Span {
                                    lo: 71,
                                    hi: 72,
                                },
                            ),
                            (
                                "",
                                Span {
                                    lo: 77,
                                    hi: 78,
                                },
                            ),
                        ],
                    },
                ),
                Lint(
                    Lint {
                        span: Span {
                            lo: 63,
                            hi: 68,
                        },
                        level: Warn,
                        message: "attempt to divide by zero",
                        help: "division by zero will fail at runtime",
                        kind: Ast(
                            DivisionByZero,
                        ),
                        code_action_edits: [],
                    },
                ),
            ]"#]],
    );
}

#[tokio::test]
async fn lints_prefer_workspace_over_defaults() {
    let this_file_qs =
        "namespace Foo { @EntryPoint() function Main() : Unit { let x = 5 / 0 + (2 ^ 4); } }";

    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors);
    updater.update_configuration(WorkspaceConfigurationUpdate {
        lints_config: Some(vec![LintConfig {
            kind: LintKind::Ast(AstLint::DivisionByZero),
            level: LintLevel::Warn,
        }]),
        ..WorkspaceConfigurationUpdate::default()
    });

    // Trigger a document update.
    updater
        .update_document("project/src/this_file.qs", 1, this_file_qs)
        .await;

    // Check generated lints.
    let lints: &[compile::ErrorKind] = &received_errors.take()[0].2;
    check_lints(
        lints,
        &expect![[r#"
            [
                Lint(
                    Lint {
                        span: Span {
                            lo: 134,
                            hi: 139,
                        },
                        level: Warn,
                        message: "attempt to divide by zero",
                        help: "division by zero will fail at runtime",
                        kind: Ast(
                            DivisionByZero,
                        ),
                        code_action_edits: [],
                    },
                ),
            ]"#]],
    );
}

#[tokio::test]
async fn lints_prefer_manifest_over_workspace() {
    let this_file_qs =
        "namespace Foo { @EntryPoint() function Main() : Unit { let x = 5 / 0 + (2 ^ 4); } }";
    let fs = FsNode::Dir(
        [dir(
            "project",
            [
                file(
                    "qsharp.json",
                    r#"{ "lints": [{ "lint": "divisionByZero", "level": "allow" }] }"#,
                ),
                dir("src", [file("this_file.qs", this_file_qs)]),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &fs);
    updater.update_configuration(WorkspaceConfigurationUpdate {
        lints_config: Some(vec![LintConfig {
            kind: LintKind::Ast(AstLint::DivisionByZero),
            level: LintLevel::Warn,
        }]),
        ..WorkspaceConfigurationUpdate::default()
    });

    // Trigger a document update.
    updater
        .update_document("project/src/this_file.qs", 1, this_file_qs)
        .await;

    // No lints expected ("allow" wins over "warn")
    assert_eq!(received_errors.borrow().len(), 0);
}

#[tokio::test]
async fn missing_dependency_reported() {
    let fs = FsNode::Dir(
        [dir(
            "parent",
            [
                file(
                    "qsharp.json",
                    r#"{ "dependencies" : { "MyDep" : { "path" : "../child" } } }"#,
                ),
                dir("src", [file("main.qs", "function Main() : Unit {}")]),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &fs);

    // Triger a document update.
    updater
        .update_document("parent/src/main.qs", 1, "function Main() : Unit {}")
        .await;

    expect![[r#"
        [
            (
                "parent/qsharp.json",
                None,
                [],
                [
                    FileSystem {
                        about_path: "child/qsharp.json",
                        error: "file not found",
                    },
                ],
            ),
        ]
    "#]]
    .assert_debug_eq(&received_errors.borrow());
}

#[tokio::test]
async fn error_from_dependency_reported() {
    let fs = FsNode::Dir(
        [
            dir(
                "parent",
                [
                    file(
                        "qsharp.json",
                        r#"{ "dependencies" : { "MyDep" : { "path" : "../child" } } }"#,
                    ),
                    dir("src", [file("main.qs", "function Main() : Unit {}")]),
                ],
            ),
            dir(
                "child",
                [
                    file("qsharp.json", "{}"),
                    dir("src", [file("main.qs", "broken_syntax")]),
                ],
            ),
        ]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let received_errors = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &fs);

    // Triger a document update.
    updater
        .update_document("parent/src/main.qs", 1, "function Main() : Unit {}")
        .await;

    expect![[r#"
        [
            (
                "child/src/main.qs",
                None,
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
                                            hi: 13,
                                        },
                                    ),
                                ),
                            ),
                        ),
                    ),
                ],
                [],
            ),
        ]
    "#]]
    .assert_debug_eq(&received_errors.borrow());
}

type ErrorInfo = (
    String,
    Option<u32>,
    Vec<compile::ErrorKind>,
    Vec<project::Error>,
);

fn new_updater(received_errors: &RefCell<Vec<ErrorInfo>>) -> CompilationStateUpdater<'_> {
    let diagnostic_receiver = move |update: DiagnosticUpdate| {
        let project_errors = update.errors.iter().filter_map(|error| match error {
            ErrorKind::Project(error) => Some(error.clone()),
            ErrorKind::Compile(_) => None,
        });
        let compile_errors = update.errors.iter().filter_map(|error| match error {
            ErrorKind::Compile(error) => Some(error.error().clone()),
            ErrorKind::Project(_) => None,
        });

        let mut v = received_errors.borrow_mut();

        v.push((
            update.uri,
            update.version,
            compile_errors.collect(),
            project_errors.collect(),
        ));
    };

    CompilationStateUpdater::new(
        Rc::new(RefCell::new(CompilationState::default())),
        diagnostic_receiver,
        TestProjectHost {
            fs: TEST_FS.with(Clone::clone),
        },
    )
}

fn new_updater_with_file_system<'a>(
    received_errors: &'a RefCell<Vec<ErrorInfo>>,
    fs: &Rc<RefCell<FsNode>>,
) -> CompilationStateUpdater<'a> {
    let diagnostic_receiver = move |update: DiagnosticUpdate| {
        let project_errors = update.errors.iter().filter_map(|error| match error {
            ErrorKind::Project(error) => Some(error.clone()),
            ErrorKind::Compile(_) => None,
        });
        let compile_errors = update.errors.iter().filter_map(|error| match error {
            ErrorKind::Compile(error) => Some(error.error().clone()),
            ErrorKind::Project(_) => None,
        });

        let mut v = received_errors.borrow_mut();

        v.push((
            update.uri,
            update.version,
            compile_errors.collect(),
            project_errors.collect(),
        ));
    };

    CompilationStateUpdater::new(
        Rc::new(RefCell::new(CompilationState::default())),
        diagnostic_receiver,
        TestProjectHost { fs: fs.clone() },
    )
}

fn expect_errors(errors: &RefCell<Vec<ErrorInfo>>, expected: &Expect) {
    expected.assert_debug_eq(&errors.borrow());
    // reset accumulated errors after each check
    errors.borrow_mut().clear();
}

fn assert_compilation_sources(updater: &CompilationStateUpdater<'_>, expected: &Expect) {
    let state = updater.state.try_borrow().expect("borrow should succeed");

    let compilation_sources =
        state
            .compilations
            .iter()
            .fold(String::new(), |mut output, (name, compilation)| {
                let _ = writeln!(output, "{}: {:#?}", name, compilation.0.user_unit().sources);
                output
            });
    expected.assert_eq(&compilation_sources);
}

fn assert_open_documents(updater: &CompilationStateUpdater<'_>, expected: &Expect) {
    let state = updater.state.try_borrow().expect("borrow should succeed");
    expected.assert_debug_eq(&state.open_documents);
}

fn check_state_and_errors(
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

fn check_state(
    updater: &CompilationStateUpdater<'_>,
    expected_open_documents: &Expect,
    expected_compilation_sources: &Expect,
) {
    assert_open_documents(updater, expected_open_documents);
    assert_compilation_sources(updater, expected_compilation_sources);
}

/// Checks that the lints config is being loaded from the qsharp.json manifest
async fn check_lints_config(updater: &CompilationStateUpdater<'_>, expected_config: &Expect) {
    let manifest = updater
        .load_manifest(&"project/src/this_file.qs".into())
        .await
        .expect("manifest should load successfully")
        .expect("manifest should exist");

    let lints_config = manifest.lints;

    expected_config.assert_eq(&format!("{lints_config:#?}"));
}

fn check_lints(lints: &[compile::ErrorKind], expected_lints: &Expect) {
    expected_lints.assert_eq(&format!("{lints:#?}"));
}

thread_local! { static TEST_FS: Rc<RefCell<FsNode>> = Rc::new(RefCell::new(test_fs()))}

fn test_fs() -> FsNode {
    FsNode::Dir(
        [
            dir(
                "project",
                [
                    file("qsharp.json", "{}"),
                    dir(
                        "src",
                        [
                            file(
                                "other_file.qs",
                                "// DISK CONTENTS\n namespace OtherFile { operation Other() : Unit { } }",
                            ),
                            file("this_file.qs", "// DISK CONTENTS\n namespace Foo { }"),
                        ],
                    ),
                ],
            ),
            dir(
                "nested_projects",
                [
                    file("qsharp.json", "{}"),
                    dir(
                        "src",
                        [dir(
                            "subdir",
                            [
                                file("qsharp.json", "{}"),
                                dir(
                                    "src",
                                    [
                                        file("a.qs", "namespace A {}"),
                                        file("b.qs", "namespace B {}"),
                                    ],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ]
        .into_iter()
        .collect(),
    )
}
