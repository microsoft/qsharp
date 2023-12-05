// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::{
    protocol::{DiagnosticUpdate, WorkspaceConfigurationUpdate},
    LanguageService,
};
use expect_test::{expect, Expect};
use qsc::{compile, PackageType, TargetProfile};
use std::{cell::RefCell, sync::Arc};

#[tokio::test]
async fn no_error() {
    let errors = RefCell::new(Vec::new());
    let mut ls = new_language_service(&errors);

    ls.update_document(
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
    let mut ls = new_language_service(&errors);

    ls.update_document("foo.qs", 1, "namespace {").await;

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

    ls.update_document(
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
                        1,
                    ),
                    [],
                ),
            ]
        "#]],
    );
}

#[tokio::test]
async fn clear_on_document_close() {
    let errors = RefCell::new(Vec::new());
    let mut ls = new_language_service(&errors);

    ls.update_document("foo.qs", 1, "namespace {").await;

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

    ls.close_document("foo.qs");

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
    let mut ls = new_language_service(&errors);

    ls.update_document("foo.qs", 1, "badsyntax").await;

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
    let mut ls = new_language_service(&errors);

    ls.update_configuration(&WorkspaceConfigurationUpdate {
        target_profile: None,
        package_type: Some(PackageType::Lib),
    });

    ls.update_document("foo.qs", 1, "namespace Foo { operation Main() : Unit {} }")
        .await;

    expect_errors(
        &errors,
        &expect![[r#"
            []
    "#]],
    );

    ls.update_configuration(&WorkspaceConfigurationUpdate {
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
    let mut ls = new_language_service(&errors);

    ls.update_configuration(&WorkspaceConfigurationUpdate {
        target_profile: Some(TargetProfile::Base),
        package_type: Some(PackageType::Lib),
    });

    ls.update_document(
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

    ls.update_configuration(&WorkspaceConfigurationUpdate {
        target_profile: Some(TargetProfile::Full),
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
    let mut ls = new_language_service(&errors);

    ls.update_document(
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

    ls.update_configuration(&WorkspaceConfigurationUpdate {
        target_profile: Some(TargetProfile::Base),
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

#[tokio::test]
async fn notebook_document_no_errors() {
    let errors = RefCell::new(Vec::new());
    let mut ls = new_language_service(&errors);

    ls.update_notebook_document(
        "notebook.ipynb",
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

#[tokio::test]
async fn notebook_document_errors() {
    let errors = RefCell::new(Vec::new());
    let mut ls = new_language_service(&errors);

    ls.update_notebook_document(
        "notebook.ipynb",
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

#[tokio::test]
async fn notebook_update_remove_cell_clears_errors() {
    let errors = RefCell::new(Vec::new());
    let mut ls = new_language_service(&errors);

    ls.update_notebook_document(
        "notebook.ipynb",
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

    ls.update_notebook_document(
        "notebook.ipynb",
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

#[tokio::test]
async fn close_notebook_clears_errors() {
    let errors = RefCell::new(Vec::new());
    let mut ls = new_language_service(&errors);

    ls.update_notebook_document(
        "notebook.ipynb",
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

    ls.close_notebook_document("notebook.ipynb", ["cell1", "cell2"].into_iter());

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

type ErrorInfo = (String, Option<u32>, Vec<compile::ErrorKind>);

fn new_language_service(received: &RefCell<Vec<ErrorInfo>>) -> LanguageService<'_> {
    let diagnostic_receiver = move |update: DiagnosticUpdate| {
        let mut v = received.borrow_mut();

        v.push((
            update.uri.to_string(),
            update.version,
            update.errors.iter().map(|e| e.error().clone()).collect(),
        ));
    };

    LanguageService::new(
        diagnostic_receiver,
        // these tests do not test project mode
        // so we provide these stubbed-out functions
        |_| Box::pin(std::future::ready((Arc::from(""), Arc::from("")))),
        |_| Box::pin(std::future::ready(vec![])),
        |_| Box::pin(std::future::ready(None)),
    )
}

// the below tests test the asynchronous behavior of the language service.
// we use `get_completions` as a rough analog for all document operations, as
// they all go through the same `document_op` infrastructure.
#[tokio::test]
async fn completions_requested_before_document_load() {
    let errors = RefCell::new(Vec::new());
    let mut ls = new_language_service(&errors);

    // we intentionally don't await this to test how LSP features function when
    // a document hasn't fully loaded
    let _ = ls.update_document(
        "foo.qs",
        1,
        "namespace Foo { open Microsoft.Quantum.Diagnostics; @EntryPoint() operation Main() : Unit { DumpMachine() } }",
    );

    // this should be empty, because the doc hasn't loaded
    assert!(ls.get_completions("foo.qs", 76).items.is_empty())
}

#[tokio::test]
async fn completions_requested_after_document_load() {
    let errors = RefCell::new(Vec::new());
    let mut ls = new_language_service(&errors);

    // this test is a contrast to `completions_requested_before_document_load`
    // we want to ensure that completions load when the update_document call has been awaited
    let _ = ls.update_document(
        "foo.qs",
        1,
        "namespace Foo { open Microsoft.Quantum.Diagnostics; @EntryPoint() operation Main() : Unit { DumpMachine() } }",
    ).await;

    // this should be empty, because the doc hasn't loaded
    assert_eq!(ls.get_completions("foo.qs", 76).items.len(), 13)
}

fn expect_errors(errors: &RefCell<Vec<ErrorInfo>>, expected: &Expect) {
    expected.assert_debug_eq(&errors.borrow());
    // reset accumulated errors after each check
    errors.borrow_mut().clear();
}
