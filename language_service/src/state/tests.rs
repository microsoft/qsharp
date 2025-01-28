// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// expect-test updates these strings automatically
#![allow(clippy::needless_raw_string_hashes, clippy::too_many_lines)]

use super::{CompilationState, CompilationStateUpdater};
use crate::{
    protocol::{DiagnosticUpdate, NotebookMetadata, TestCallables, WorkspaceConfigurationUpdate},
    tests::test_fs::{dir, file, FsNode, TestProjectHost},
};
use expect_test::{expect, Expect};
use miette::Diagnostic;
use qsc::{line_column::Encoding, target::Profile, LanguageFeatures, PackageType};
use qsc_linter::{AstLint, LintConfig, LintKind, LintLevel};
use std::{
    cell::RefCell,
    fmt::{Display, Write},
    rc::Rc,
    str::from_utf8,
};

#[tokio::test]
async fn no_error() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

    updater
        .update_document(
            "single/foo.qs",
            1,
            "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
        )
        .await;

    expect_errors(&errors, &expect!["[]"]);
}

#[tokio::test]
async fn clear_error() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

    updater
        .update_document("single/foo.qs", 1, "namespace {")
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
              uri: "single/foo.qs" version: Some(1) errors: [
                syntax error
                  [single/foo.qs] [{]
              ],
            ]"#]],
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
              uri: "single/foo.qs" version: Some(2) errors: [],
            ]"#]],
    );
}

#[tokio::test]
async fn close_last_doc_in_project() {
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors, &test_cases);

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
            project/qsharp.json: [
              "project/src/other_file.qs": "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
              "project/src/this_file.qs": "// DISK CONTENTS\n namespace Foo { }",
            ],
        "#]],
        &expect![[r#"
            [
              uri: "project/src/this_file.qs" version: Some(1) errors: [
                syntax error
                  [project/src/this_file.qs] [/]
              ],

              uri: "project/src/this_file.qs" version: None errors: [],
            ]"#]],
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
        &expect!["[]"],
    );
}

#[tokio::test]
async fn clear_on_document_close() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());

    let mut updater = new_updater(&errors, &test_cases);

    updater
        .update_document("single/foo.qs", 1, "namespace {")
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
              uri: "single/foo.qs" version: Some(1) errors: [
                syntax error
                  [single/foo.qs] [{]
              ],
            ]"#]],
    );

    updater.close_document("single/foo.qs").await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
              uri: "single/foo.qs" version: None errors: [],
            ]"#]],
    );
}

#[tokio::test]
async fn compile_error() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

    updater
        .update_document("single/foo.qs", 1, "badsyntax")
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            [
              uri: "single/foo.qs" version: Some(1) errors: [
                syntax error
                  [single/foo.qs] [badsyntax]
              ],
            ]"#]],
    );
}

#[tokio::test]
async fn rca_errors_are_reported_when_compilation_succeeds() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

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
              uri: "single/foo.qs" version: Some(1) errors: [
                cannot use a dynamic double value
                  [single/foo.qs] [set x = 2.0]
                cannot use a dynamic double value
                  [single/foo.qs] [x]
              ],
            ]"#]],
    );
}

#[tokio::test]
async fn base_profile_rca_errors_are_reported_when_compilation_succeeds() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

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
              uri: "single/foo.qs" version: Some(1) errors: [
                cannot use a dynamic bool value
                  [single/foo.qs] [MResetZ(q) == One]
                cannot use a dynamic double value
                  [single/foo.qs] [set x = 2.0]
                cannot use a dynamic double value
                  [single/foo.qs] [x]
              ],
            ]"#]],
    );
}

#[tokio::test]
async fn package_type_update_causes_error() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

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

    expect_errors(&errors, &expect!["[]"]);

    updater.update_configuration(WorkspaceConfigurationUpdate {
        package_type: Some(PackageType::Exe),
        ..WorkspaceConfigurationUpdate::default()
    });

    expect_errors(
        &errors,
        &expect![[r#"
            [
              uri: "single/foo.qs" version: Some(1) errors: [
                entry point not found
              ],
            ]"#]],
    );
}

#[tokio::test]
async fn target_profile_update_fixes_error() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

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
              uri: "single/foo.qs" version: Some(1) errors: [
                cannot use a dynamic bool value
                  [single/foo.qs] [M(q) == Zero]
              ],
            ]"#]],
    );

    updater.update_configuration(WorkspaceConfigurationUpdate {
        target_profile: Some(Profile::Unrestricted),
        ..WorkspaceConfigurationUpdate::default()
    });

    expect_errors(
        &errors,
        &expect![[r#"
            [
              uri: "single/foo.qs" version: Some(1) errors: [],
            ]"#]],
    );
}

#[tokio::test]
async fn target_profile_update_causes_error_in_stdlib() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

    updater.update_document(
        "single/foo.qs",
        1,
        r#"namespace Foo { @EntryPoint() operation Main() : Unit { use q = Qubit(); let r = M(q); let b = Microsoft.Quantum.Convert.ResultAsBool(r); } }"#,
    ).await;

    expect_errors(&errors, &expect!["[]"]);

    updater.update_configuration(WorkspaceConfigurationUpdate {
        target_profile: Some(Profile::Base),
        ..WorkspaceConfigurationUpdate::default()
    });

    expect_errors(
        &errors,
        &expect![[r#"
            [
              uri: "single/foo.qs" version: Some(1) errors: [
                cannot use a dynamic bool value
                  [single/foo.qs] [Microsoft.Quantum.Convert.ResultAsBool(r)]
              ],
            ]"#]],
    );
}

#[tokio::test]
async fn notebook_document_no_errors() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

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

    expect_errors(&errors, &expect!["[]"]);
}

#[tokio::test]
async fn notebook_document_errors() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

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
              uri: "cell2" version: Some(1) errors: [
                name error
                  [cell2] [Foo]
                type error
                  [cell2] [Foo()]
              ],
            ]"#]],
    );
}

#[tokio::test]
async fn notebook_document_lints() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

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
              uri: "cell1" version: Some(1) errors: [
                redundant semicolons
                  [cell1] [;;;]
              ],

              uri: "cell2" version: Some(1) errors: [
                attempt to divide by zero
                  [cell2] [5 / 0]
              ],
            ]"#]],
    );
}

#[tokio::test]
async fn notebook_update_remove_cell_clears_errors() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

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
              uri: "cell2" version: Some(1) errors: [
                name error
                  [cell2] [Foo]
                type error
                  [cell2] [Foo()]
              ],
            ]"#]],
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
              uri: "cell2" version: None errors: [],
            ]"#]],
    );
}

#[tokio::test]
async fn close_notebook_clears_errors() {
    let errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&errors, &test_cases);

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
              uri: "cell2" version: Some(1) errors: [
                name error
                  [cell2] [Foo]
                type error
                  [cell2] [Foo()]
              ],
            ]"#]],
    );

    updater.close_notebook_document("notebook.ipynb");

    expect_errors(
        &errors,
        &expect![[r#"
            [
              uri: "cell2" version: None errors: [],
            ]"#]],
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
    let test_cases = RefCell::new(Vec::new());

    let mut updater = new_updater_with_file_system(&errors, &test_cases, &fs);

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

    expect_errors(&errors, &expect!["[]"]);
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
    let test_cases = RefCell::new(Vec::new());

    let mut updater = new_updater_with_file_system(&errors, &test_cases, &fs);

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
              uri: "cell1" version: Some(1) errors: [
                name error
                  [cell1] [Foo]
                name error
                  [cell1] [Foo]
                name error
                  [cell1] [Bar]
                type error
                  [cell1] [Bar()]
              ],

              uri: "project/src/file.qs" version: None errors: [
                type error
                  [project/src/file.qs] [Int]
              ],
            ]"#]],
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
    let test_cases = RefCell::new(Vec::new());

    let mut updater = new_updater_with_file_system(&errors, &test_cases, &fs);

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
              uri: "project2/src/file.qs" version: None errors: [
                type error
                  [project2/src/file.qs] [Int]
              ],
            ]"#]],
    );
}

#[tokio::test]
async fn update_doc_updates_project() {
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors, &test_cases);

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
            project/qsharp.json: [
              "project/src/other_file.qs": "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
              "project/src/this_file.qs": "namespace Foo { we should see this in the source }",
            ],
        "#]],
        &expect![[r#"
            [
              uri: "project/src/this_file.qs" version: Some(1) errors: [
                syntax error
                  [project/src/this_file.qs] [we]
              ],
            ]"#]],
    );
}

#[tokio::test]
async fn file_not_in_files_list() {
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());

    // Manifest has a "files" field.
    // One file is listed in it, the other is not.
    // This shouldn't block project load, but should generate an error.
    let fs = FsNode::Dir(
        [dir(
            "project",
            [
                file(
                    "qsharp.json",
                    r#"{
                        "files" : [
                            "src/explicitly_listed.qs"
                        ]
                    }"#,
                ),
                dir(
                    "src",
                    [
                        file("explicitly_listed.qs", "// CONTENTS"),
                        file("unlisted.qs", "// CONTENTS"),
                    ],
                ),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let mut updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);

    // Open the file that is listed in the files list
    updater
        .update_document("project/src/explicitly_listed.qs", 1, "// CONTENTS")
        .await;

    // The whole project should be loaded, which should generate
    // an error about the other file that's unlisted.
    // They are both in the compilation.
    check_state_and_errors(
        &updater,
        &received_errors,
        &expect![[r#"
            {
                "project/src/explicitly_listed.qs": OpenDocument {
                    version: 1,
                    compilation: "project/qsharp.json",
                    latest_str_content: "// CONTENTS",
                },
            }
        "#]],
        &expect![[r#"
            project/qsharp.json: [
              "project/src/explicitly_listed.qs": "// CONTENTS",
              "project/src/unlisted.qs": "// CONTENTS",
            ],
        "#]],
        &expect![[r#"
            [
              uri: "project/src/unlisted.qs" version: None errors: [
                File src/unlisted.qs is not listed in the `files` field of the manifest
              ],
            ]"#]],
    );

    // Open the unlisted file as well.
    updater
        .update_document("project/src/unlisted.qs", 1, "// CONTENTS")
        .await;

    // Documents are both open and correctly associated with the project.
    // The error about the unlisted file persists.
    check_state_and_errors(
        &updater,
        &received_errors,
        &expect![[r#"
            {
                "project/src/explicitly_listed.qs": OpenDocument {
                    version: 1,
                    compilation: "project/qsharp.json",
                    latest_str_content: "// CONTENTS",
                },
                "project/src/unlisted.qs": OpenDocument {
                    version: 1,
                    compilation: "project/qsharp.json",
                    latest_str_content: "// CONTENTS",
                },
            }
        "#]],
        &expect![[r#"
            project/qsharp.json: [
              "project/src/explicitly_listed.qs": "// CONTENTS",
              "project/src/unlisted.qs": "// CONTENTS",
            ],
        "#]],
        &expect![[r#"
            [
              uri: "project/src/unlisted.qs" version: Some(1) errors: [
                File src/unlisted.qs is not listed in the `files` field of the manifest
              ],
            ]"#]],
    );
}

#[tokio::test]
async fn file_not_under_src() {
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());

    // One file lives under the 'src' directory, the other does not.
    // The one that isn't under 'src' should not be associated with the project.
    let fs = FsNode::Dir(
        [dir(
            "project",
            [
                file(
                    "qsharp.json",
                    r#"{
                        "files" : [
                            "src/under_src.qs"
                        ]
                    }"#,
                ),
                file("not_under_src.qs", "// CONTENTS"),
                dir("src", [file("under_src.qs", "// CONTENTS")]),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let mut updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);

    // Open the file that is not under src.
    updater
        .update_document("project/not_under_src.qs", 1, "// CONTENTS")
        .await;

    // This document is not associated with the manifest,
    // didn't cause the manifest to be loaded,
    // and lives in its own project by itself.
    check_state_and_errors(
        &updater,
        &received_errors,
        &expect![[r#"
            {
                "project/not_under_src.qs": OpenDocument {
                    version: 1,
                    compilation: "project/not_under_src.qs",
                    latest_str_content: "// CONTENTS",
                },
            }
        "#]],
        &expect![[r#"
            project/not_under_src.qs: [
              "project/not_under_src.qs": "// CONTENTS",
            ],
        "#]],
        &expect!["[]"],
    );

    // Open the file that's properly under the "src" directory.
    updater
        .update_document("project/src/under_src.qs", 1, "// CONTENTS")
        .await;

    // The manifest is loaded, `not_under_src.qs` is still not associated with it.
    check_state_and_errors(
        &updater,
        &received_errors,
        &expect![[r#"
            {
                "project/not_under_src.qs": OpenDocument {
                    version: 1,
                    compilation: "project/not_under_src.qs",
                    latest_str_content: "// CONTENTS",
                },
                "project/src/under_src.qs": OpenDocument {
                    version: 1,
                    compilation: "project/qsharp.json",
                    latest_str_content: "// CONTENTS",
                },
            }
        "#]],
        &expect![[r#"
            project/not_under_src.qs: [
              "project/not_under_src.qs": "// CONTENTS",
            ],
            project/qsharp.json: [
              "project/src/under_src.qs": "// CONTENTS",
            ],
        "#]],
        &expect!["[]"],
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
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors, &test_cases);

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
            project/qsharp.json: [
              "project/src/other_file.qs": "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
              "project/src/this_file.qs": "// DISK CONTENTS\n namespace Foo { }",
            ],
        "#]],
        &expect![[r#"
            [
              uri: "project/src/this_file.qs" version: Some(1) errors: [
                syntax error
                  [project/src/this_file.qs] [/]
              ],

              uri: "project/src/this_file.qs" version: None errors: [],
            ]"#]],
    );
}

#[tokio::test]
async fn delete_manifest() {
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors, &test_cases);

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
            project/qsharp.json: [
              "project/src/other_file.qs": "// DISK CONTENTS\n namespace OtherFile { operation Other() : Unit { } }",
              "project/src/this_file.qs": "// DISK CONTENTS\n namespace Foo { }",
            ],
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
            project/src/this_file.qs: [
              "project/src/this_file.qs": "// DISK CONTENTS\n namespace Foo { }",
            ],
        "#]],
    );
}

#[tokio::test]
async fn delete_manifest_then_close() {
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors, &test_cases);

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
            project/qsharp.json: [
              "project/src/other_file.qs": "// DISK CONTENTS\n namespace OtherFile { operation Other() : Unit { } }",
              "project/src/this_file.qs": "// DISK CONTENTS\n namespace Foo { }",
            ],
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
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors, &test_cases);

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
            nested_projects/src/subdir/qsharp.json: [
              "nested_projects/src/subdir/src/a.qs": "namespace A {}",
              "nested_projects/src/subdir/src/b.qs": "namespace B {}",
            ],
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
            nested_projects/qsharp.json: [
              "nested_projects/src/subdir/src/a.qs": "namespace A {}",
              "nested_projects/src/subdir/src/b.qs": "namespace B {}",
            ],
        "#]],
    );
}

#[tokio::test]
async fn doc_switches_project_on_close() {
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors, &test_cases);

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
            nested_projects/src/subdir/qsharp.json: [
              "nested_projects/src/subdir/src/a.qs": "namespace A {}",
              "nested_projects/src/subdir/src/b.qs": "namespace B {}",
            ],
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
            nested_projects/qsharp.json: [
              "nested_projects/src/subdir/src/a.qs": "namespace A {}",
              "nested_projects/src/subdir/src/b.qs": "namespace B {}",
            ],
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
    let test_cases = RefCell::new(Vec::new());

    let updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);

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
    let test_cases = RefCell::new(Vec::new());

    let mut updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);

    // Trigger a document update.
    updater
        .update_document("project/src/this_file.qs", 1, this_file_qs)
        .await;

    // Check generated lints.
    expect_errors(
        &received_errors,
        &expect![[r#"
        [
          uri: "project/src/this_file.qs" version: Some(1) errors: [
            unnecessary parentheses
              [project/src/this_file.qs] [(2 ^ 4)]
            attempt to divide by zero
              [project/src/this_file.qs] [5 / 0]
          ],
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
    expect_errors(
        &received_errors,
        &expect![[r#"
            [
              uri: "project/src/this_file.qs" version: Some(1) errors: [
                unnecessary parentheses
                  [project/src/this_file.qs] [(2 ^ 4)]
                attempt to divide by zero
                  [project/src/this_file.qs] [5 / 0]
              ],
            ]"#]],
    );
}

#[tokio::test]
async fn lints_prefer_workspace_over_defaults() {
    let this_file_qs =
        "namespace Foo { @EntryPoint() function Main() : Unit { let x = 5 / 0 + (2 ^ 4); } }";

    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater(&received_errors, &test_cases);
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
    expect_errors(
        &received_errors,
        &expect![[r#"
            [
              uri: "project/src/this_file.qs" version: Some(1) errors: [
                attempt to divide by zero
                  [project/src/this_file.qs] [5 / 0]
              ],
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
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);
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
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);

    // Trigger a document update.
    updater
        .update_document("parent/src/main.qs", 1, "function Main() : Unit {}")
        .await;

    expect_errors(
        &received_errors,
        &expect![[r#"
            [
              uri: "parent/qsharp.json" version: None errors: [
                File system error: child/qsharp.json: file not found
              ],
            ]"#]],
    );
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
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);

    // Trigger a document update.
    updater
        .update_document("parent/src/main.qs", 1, "function Main() : Unit {}")
        .await;

    expect_errors(
        &received_errors,
        &expect![[r#"
            [
              uri: "child/src/main.qs" version: None errors: [
                syntax error
                  [child/src/main.qs] [broken_syntax]
              ],
            ]"#]],
    );
}

#[tokio::test]
async fn test_case_detected() {
    let fs = FsNode::Dir(
        [dir(
            "parent",
            [
                file("qsharp.json", r#"{}"#),
                dir("src", [file("main.qs", "function MyTestCase() : Unit {}")]),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);

    // Trigger a document update.
    updater
        .update_document(
            "parent/src/main.qs",
            1,
            "@Test() function MyTestCase() : Unit {}",
        )
        .await;

    expect![[r#"
        [
            TestCallables {
                callables: [
                    (
                        "parent/qsharp.json",
                        "main.MyTestCase",
                        Location {
                            source: "parent/src/main.qs",
                            range: Range {
                                start: Position {
                                    line: 0,
                                    column: 17,
                                },
                                end: Position {
                                    line: 0,
                                    column: 27,
                                },
                            },
                        },
                        "parent",
                    ),
                ],
            },
        ]
    "#]]
    .assert_debug_eq(&test_cases.borrow());
}

#[tokio::test]
async fn test_case_removed() {
    let fs = FsNode::Dir(
        [dir(
            "parent",
            [
                file("qsharp.json", r#"{}"#),
                dir(
                    "src",
                    [file("main.qs", "@Test() function MyTestCase() : Unit {}")],
                ),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);

    // Trigger a document update.
    updater
        .update_document("parent/src/main.qs", 1, "function MyTestCase() : Unit {}")
        .await;

    expect![[r#"
        [
            TestCallables {
                callables: [],
            },
        ]
    "#]]
    .assert_debug_eq(&test_cases.borrow());
}

#[tokio::test]
async fn test_case_modified() {
    let fs = FsNode::Dir(
        [dir(
            "parent",
            [
                file("qsharp.json", r#"{}"#),
                dir(
                    "src",
                    [file("main.qs", "@Test() function MyTestCase() : Unit {}")],
                ),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);

    // Trigger a document update.
    updater
        .update_document(
            "parent/src/main.qs",
            1,
            "@Test() function MyTestCase() : Unit {}",
        )
        .await;

    updater
        .update_document(
            "parent/src/main.qs",
            2,
            "@Test() function MyTestCase2() : Unit { }",
        )
        .await;

    expect![[r#"
        [
            TestCallables {
                callables: [
                    (
                        "parent/qsharp.json",
                        "main.MyTestCase",
                        Location {
                            source: "parent/src/main.qs",
                            range: Range {
                                start: Position {
                                    line: 0,
                                    column: 17,
                                },
                                end: Position {
                                    line: 0,
                                    column: 27,
                                },
                            },
                        },
                        "parent",
                    ),
                ],
            },
            TestCallables {
                callables: [
                    (
                        "parent/qsharp.json",
                        "main.MyTestCase2",
                        Location {
                            source: "parent/src/main.qs",
                            range: Range {
                                start: Position {
                                    line: 0,
                                    column: 17,
                                },
                                end: Position {
                                    line: 0,
                                    column: 28,
                                },
                            },
                        },
                        "parent",
                    ),
                ],
            },
        ]
    "#]]
    .assert_debug_eq(&test_cases.borrow());
}

#[tokio::test]
async fn test_annotation_removed() {
    let fs = FsNode::Dir(
        [dir(
            "parent",
            [
                file("qsharp.json", r#"{}"#),
                dir(
                    "src",
                    [file("main.qs", "@Test() function MyTestCase() : Unit {}")],
                ),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);

    // Trigger a document update.
    updater
        .update_document(
            "parent/src/main.qs",
            1,
            "@Test() function MyTestCase() : Unit {}",
        )
        .await;

    updater
        .update_document("parent/src/main.qs", 2, "function MyTestCase() : Unit {}")
        .await;

    expect![[r#"
        [
            TestCallables {
                callables: [
                    (
                        "parent/qsharp.json",
                        "main.MyTestCase",
                        Location {
                            source: "parent/src/main.qs",
                            range: Range {
                                start: Position {
                                    line: 0,
                                    column: 17,
                                },
                                end: Position {
                                    line: 0,
                                    column: 27,
                                },
                            },
                        },
                        "parent",
                    ),
                ],
            },
            TestCallables {
                callables: [],
            },
        ]
    "#]]
    .assert_debug_eq(&test_cases.borrow());
}

#[tokio::test]
async fn multiple_tests() {
    let fs = FsNode::Dir(
        [dir(
            "parent",
            [
                file("qsharp.json", r#"{}"#),
                dir(
                    "src",
                    [file(
                        "main.qs",
                        "@Test() function Test1() : Unit {} @Test() function Test2() : Unit {}",
                    )],
                ),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);

    // Trigger a document update.
    updater
        .update_document(
            "parent/src/main.qs",
            1,
            "@Test() function Test1() : Unit {} @Test() function Test2() : Unit {}",
        )
        .await;

    expect![[r#"
        [
            TestCallables {
                callables: [
                    (
                        "parent/qsharp.json",
                        "main.Test1",
                        Location {
                            source: "parent/src/main.qs",
                            range: Range {
                                start: Position {
                                    line: 0,
                                    column: 17,
                                },
                                end: Position {
                                    line: 0,
                                    column: 22,
                                },
                            },
                        },
                        "parent",
                    ),
                    (
                        "parent/qsharp.json",
                        "main.Test2",
                        Location {
                            source: "parent/src/main.qs",
                            range: Range {
                                start: Position {
                                    line: 0,
                                    column: 52,
                                },
                                end: Position {
                                    line: 0,
                                    column: 57,
                                },
                            },
                        },
                        "parent",
                    ),
                ],
            },
        ]
    "#]]
    .assert_debug_eq(&test_cases.borrow());
}

#[tokio::test]
async fn test_case_in_different_files() {
    let fs = FsNode::Dir(
        [dir(
            "parent",
            [
                file("qsharp.json", r#"{}"#),
                dir(
                    "src",
                    [
                        file("test1.qs", "@Test() function Test1() : Unit {}"),
                        file("test2.qs", "@Test() function Test2() : Unit {}"),
                    ],
                ),
            ],
        )]
        .into_iter()
        .collect(),
    );

    let fs = Rc::new(RefCell::new(fs));
    let received_errors = RefCell::new(Vec::new());
    let test_cases = RefCell::new(Vec::new());
    let mut updater = new_updater_with_file_system(&received_errors, &test_cases, &fs);

    // Trigger a document update for the first test file.
    updater
        .update_document(
            "parent/src/test1.qs",
            1,
            "@Test() function Test1() : Unit {}",
        )
        .await;

    expect![[r#"
        [
            TestCallables {
                callables: [
                    (
                        "parent/qsharp.json",
                        "test1.Test1",
                        Location {
                            source: "parent/src/test1.qs",
                            range: Range {
                                start: Position {
                                    line: 0,
                                    column: 17,
                                },
                                end: Position {
                                    line: 0,
                                    column: 22,
                                },
                            },
                        },
                        "parent",
                    ),
                    (
                        "parent/qsharp.json",
                        "test2.Test2",
                        Location {
                            source: "parent/src/test2.qs",
                            range: Range {
                                start: Position {
                                    line: 0,
                                    column: 17,
                                },
                                end: Position {
                                    line: 0,
                                    column: 22,
                                },
                            },
                        },
                        "parent",
                    ),
                ],
            },
        ]
    "#]]
    .assert_debug_eq(&test_cases.borrow());
}

impl Display for DiagnosticUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let DiagnosticUpdate {
            uri,
            version,
            errors,
        } = self;

        write!(f, "uri: {uri:?} version: {version:?} errors: [",)?;
        // Formatting loosely taken from compiler/qsc/src/interpret/tests.rs
        for error in errors {
            write!(f, "\n    {error}")?;
            for label in error.labels().into_iter().flatten() {
                let span = error
                    .source_code()
                    .expect("expected valid source code")
                    .read_span(label.inner(), 0, 0)
                    .expect("expected to be able to read span");

                write!(
                    f,
                    "\n     {} [{}] [{}]",
                    label.label().unwrap_or(""),
                    span.name().expect("expected source file name"),
                    from_utf8(span.data()).expect("expected valid utf-8 string"),
                )?;
            }
        }
        if !errors.is_empty() {
            write!(f, "\n  ")?;
        }
        writeln!(f, "],")?;

        Ok(())
    }
}

fn new_updater<'a>(
    received_errors: &'a RefCell<Vec<DiagnosticUpdate>>,
    received_test_cases: &'a RefCell<Vec<TestCallables>>,
) -> CompilationStateUpdater<'a> {
    let diagnostic_receiver = move |update: DiagnosticUpdate| {
        let mut v = received_errors.borrow_mut();
        v.push(update);
    };

    let test_callable_receiver = move |update: TestCallables| {
        let mut v = received_test_cases.borrow_mut();
        v.push(update);
    };

    CompilationStateUpdater::new(
        Rc::new(RefCell::new(CompilationState::default())),
        diagnostic_receiver,
        test_callable_receiver,
        TestProjectHost {
            fs: TEST_FS.with(Clone::clone),
        },
        Encoding::Utf8,
    )
}

fn new_updater_with_file_system<'a>(
    received_errors: &'a RefCell<Vec<DiagnosticUpdate>>,
    received_test_cases: &'a RefCell<Vec<TestCallables>>,
    fs: &Rc<RefCell<FsNode>>,
) -> CompilationStateUpdater<'a> {
    let diagnostic_receiver = move |update: DiagnosticUpdate| {
        let mut v = received_errors.borrow_mut();
        v.push(update);
    };

    let test_callable_receiver = move |update: TestCallables| {
        let mut v = received_test_cases.borrow_mut();
        v.push(update);
    };

    CompilationStateUpdater::new(
        Rc::new(RefCell::new(CompilationState::default())),
        diagnostic_receiver,
        test_callable_receiver,
        TestProjectHost { fs: fs.clone() },
        Encoding::Utf8,
    )
}

fn expect_errors(updates: &RefCell<Vec<DiagnosticUpdate>>, expected: &Expect) {
    let mut buf = String::new();
    let _ = buf.write_str("[");
    for update in updates.borrow().iter() {
        let _ = write!(buf, "\n  {update}");
    }
    let _ = buf.write_str("]");

    expected.assert_eq(&buf);

    // reset accumulated errors after each check
    updates.borrow_mut().clear();
}

fn assert_compilation_sources(updater: &CompilationStateUpdater<'_>, expected: &Expect) {
    let state = updater.state.try_borrow().expect("borrow should succeed");

    let compilation_sources =
        state
            .compilations
            .iter()
            .fold(String::new(), |mut output, (name, compilation)| {
                let _ = writeln!(output, "{name}: [");
                for source in compilation.0.user_unit().sources.iter() {
                    let _ = writeln!(output, "  {:?}: {:?},", source.name, source.contents);
                }
                let _ = writeln!(output, "],");
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
    received_diag_updates: &RefCell<Vec<DiagnosticUpdate>>,
    expected_open_documents: &Expect,
    expected_compilation_sources: &Expect,
    expected_errors: &Expect,
) {
    assert_open_documents(updater, expected_open_documents);
    assert_compilation_sources(updater, expected_compilation_sources);
    expect_errors(received_diag_updates, expected_errors);
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
