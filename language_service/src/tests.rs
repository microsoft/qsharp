// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{protocol::WorkspaceConfigurationUpdate, LanguageService};
use expect_test::{expect, Expect};
use qsc::{compile, PackageType, TargetProfile};
use std::cell::RefCell;

#[test]
fn no_error() {
    let errors = RefCell::new(Vec::new());
    let mut ls = new_language_service(&errors);

    ls.update_document(
        "foo.qs",
        1,
        "namespace Foo { @EntryPoint() operation Main() : Unit {} }",
    );

    expect_errors(
        &errors,
        &expect![[r#"
        [
            (
                "foo.qs",
                1,
                [],
            ),
        ]
    "#]],
    );
}

#[test]
fn compile_error() {
    let errors = RefCell::new(Vec::new());
    let mut ls = new_language_service(&errors);

    ls.update_document("foo.qs", 1, "badsyntax");

    expect_errors(
        &errors,
        &expect![[r#"
        [
            (
                "foo.qs",
                1,
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

#[test]
fn package_type_update_causes_error() {
    let errors = RefCell::new(Vec::new());
    let mut ls = new_language_service(&errors);

    ls.update_configuration(&WorkspaceConfigurationUpdate {
        target_profile: None,
        package_type: Some(PackageType::Lib),
    });

    ls.update_document("foo.qs", 1, "namespace Foo { operation Main() : Unit {} }");

    expect_errors(
        &errors,
        &expect![[r#"
        [
            (
                "foo.qs",
                1,
                [],
            ),
        ]
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
                    1,
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

#[test]
fn target_profile_update_fixes_error() {
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
    );

    expect_errors(
        &errors,
        &expect![[r#"
    [
        (
            "foo.qs",
            1,
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
                1,
                [],
            ),
        ]
    "#]],
    );
}

fn new_language_service(
    received: &RefCell<Vec<(String, u32, Vec<compile::Error>)>>,
) -> LanguageService<'_> {
    LanguageService::new(|uri: &str, version: u32, errors: &[compile::Error]| {
        let mut v = received.borrow_mut();
        v.push((uri.to_string(), version, errors.to_vec()));
    })
}

fn expect_errors(errors: &RefCell<Vec<(String, u32, Vec<compile::Error>)>>, expected: &Expect) {
    expected.assert_debug_eq(&errors.borrow());
    errors.borrow_mut().clear();
}
