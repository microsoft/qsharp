// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::{protocol::DiagnosticUpdate, Encoding, JSFileEntry, LanguageService, UpdateWorker};
use expect_test::{expect, Expect};
use qsc::{
    compile::{self, ErrorKind},
    line_column::Position,
};
use qsc_project::{EntryType, Manifest, ManifestDescriptor};
use std::{cell::RefCell, future::ready, sync::Arc};

#[tokio::test]
async fn single_document() {
    let received_errors = RefCell::new(Vec::new());
    let mut ls = LanguageService::new(Encoding::Utf8);
    let mut worker = create_update_worker(&mut ls, &received_errors);

    ls.update_document("foo.qs", 1, "namespace Foo { }");

    worker.apply_pending().await;

    check_errors_and_compilation(
        &ls,
        &mut received_errors.borrow_mut(),
        "foo.qs",
        &(expect![[r#"
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
        "#]]),
        &(expect![[r#"
            SourceMap {
                sources: [
                    Source {
                        name: "foo.qs",
                        contents: "namespace Foo { }",
                        offset: 0,
                    },
                ],
                entry: None,
            }
        "#]]),
    );
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn single_document_update() {
    let received_errors = RefCell::new(Vec::new());
    let mut ls = LanguageService::new(Encoding::Utf8);
    let mut worker = create_update_worker(&mut ls, &received_errors);

    ls.update_document("foo.qs", 1, "namespace Foo { }");

    worker.apply_pending().await;

    check_errors_and_compilation(
        &ls,
        &mut received_errors.borrow_mut(),
        "foo.qs",
        &(expect![[r#"
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
        "#]]),
        &(expect![[r#"
            SourceMap {
                sources: [
                    Source {
                        name: "foo.qs",
                        contents: "namespace Foo { }",
                        offset: 0,
                    },
                ],
                entry: None,
            }
        "#]]),
    );

    // UPDATE 2
    ls.update_document(
        "foo.qs",
        1,
        "namespace Foo { @EntryPoint() operation Bar() : Unit {} }",
    );

    worker.apply_pending().await;

    check_errors_and_compilation(
        &ls,
        &mut received_errors.borrow_mut(),
        "foo.qs",
        &(expect![[r#"
            [
                (
                    "foo.qs",
                    Some(
                        1,
                    ),
                    [],
                ),
            ]
        "#]]),
        &(expect![[r#"
            SourceMap {
                sources: [
                    Source {
                        name: "foo.qs",
                        contents: "namespace Foo { @EntryPoint() operation Bar() : Unit {} }",
                        offset: 0,
                    },
                ],
                entry: None,
            }
        "#]]),
    );
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn document_in_project() {
    let received_errors = RefCell::new(Vec::new());
    let mut ls = LanguageService::new(Encoding::Utf8);
    let mut worker = create_update_worker(&mut ls, &received_errors);

    ls.update_document("this_file.qs", 1, "namespace Foo { }");

    check_errors_and_no_compilation(
        &ls,
        &mut received_errors.borrow_mut(),
        "this_file.qs",
        &(expect![[r#"
            []
        "#]]),
    );

    // now process background work
    worker.apply_pending().await;

    check_errors_and_compilation(
        &ls,
        &mut received_errors.borrow_mut(),
        "this_file.qs",
        &expect![[r#"
            [
                (
                    "./qsharp.json",
                    None,
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
        &expect![[r#"
            SourceMap {
                sources: [
                    Source {
                        name: "other_file.qs",
                        contents: "namespace OtherFile { operation Other() : Unit {} }",
                        offset: 0,
                    },
                    Source {
                        name: "this_file.qs",
                        contents: "namespace Foo { }",
                        offset: 52,
                    },
                ],
                entry: None,
            }
        "#]],
    );
}

// the below tests test the asynchronous behavior of the language service.
// we use `get_completions` as a rough analog for all document operations, as
// they all go through the same `document_op` infrastructure.
#[tokio::test]
async fn completions_requested_before_document_load() {
    let errors = RefCell::new(Vec::new());
    let mut ls = LanguageService::new(Encoding::Utf8);
    let _worker = create_update_worker(&mut ls, &errors);

    ls.update_document(
        "foo.qs",
        1,
        "namespace Foo { open Microsoft.Quantum.Diagnostics; @EntryPoint() operation Main() : Unit { DumpMachine() } }",
    );

    // we intentionally don't await work to test how LSP features function when
    // a document hasn't fully loaded

    // this should be empty, because the doc hasn't loaded
    assert!(ls
        .get_completions(
            "foo.qs",
            Position {
                line: 0,
                column: 76
            }
        )
        .items
        .is_empty());
}

#[tokio::test]
async fn completions_requested_after_document_load() {
    let errors = RefCell::new(Vec::new());
    let mut ls = LanguageService::new(Encoding::Utf8);
    let mut worker = create_update_worker(&mut ls, &errors);

    // this test is a contrast to `completions_requested_before_document_load`
    // we want to ensure that completions load when the update_document call has been awaited
    ls.update_document(
        "foo.qs",
        1,
        "namespace Foo { open Microsoft.Quantum.Diagnostics; @EntryPoint() operation Main() : Unit { DumpMachine() } }",
    );

    worker.apply_pending().await;

    // this should be empty, because the doc hasn't loaded
    assert_eq!(
        ls.get_completions(
            "foo.qs",
            Position {
                line: 0,
                column: 76
            }
        )
        .items
        .len(),
        13
    );
}

fn check_errors_and_compilation(
    ls: &LanguageService,
    received_errors: &mut Vec<(String, Option<u32>, Vec<ErrorKind>)>,
    uri: &str,
    expected_errors: &Expect,
    expected_compilation: &Expect,
) {
    expected_errors.assert_debug_eq(received_errors);
    assert_compilation(ls, uri, expected_compilation);
    received_errors.clear();
}

fn check_errors_and_no_compilation(
    ls: &LanguageService,
    received_errors: &mut Vec<(String, Option<u32>, Vec<ErrorKind>)>,
    uri: &str,
    expected_errors: &Expect,
) {
    expected_errors.assert_debug_eq(received_errors);
    received_errors.clear();

    let state = ls.state.try_borrow().expect("borrow should succeed");
    assert!(state.get_compilation(uri).is_none());
}

fn assert_compilation(ls: &LanguageService, uri: &str, expected: &Expect) {
    let state = ls.state.try_borrow().expect("borrow should succeed");
    let compilation = state
        .get_compilation(uri)
        .expect("compilation should exist");
    expected.assert_debug_eq(&compilation.user_unit().sources);
}

type ErrorInfo = (String, Option<u32>, Vec<compile::ErrorKind>);

fn create_update_worker<'a>(
    ls: &mut LanguageService,
    received_errors: &'a RefCell<Vec<ErrorInfo>>,
) -> UpdateWorker<'a> {
    let worker = ls.create_update_worker(
        |update: DiagnosticUpdate| {
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
        },
        |file| {
            Box::pin(async {
                tokio::spawn(ready(match file.as_str() {
                    "other_file.qs" => (
                        Arc::from(file),
                        Arc::from("namespace OtherFile { operation Other() : Unit {} }"),
                    ),
                    "this_file.qs" => (Arc::from(file), Arc::from("namespace Foo { }")),
                    _ => panic!("unknown file"),
                }))
                .await
                .expect("spawn should not fail")
            })
        },
        |dir_name| {
            Box::pin(async move {
                tokio::spawn(ready(vec![
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
                ]))
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
    );
    worker
}
