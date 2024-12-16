// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};
use qsc_hir::{validate::Validator, visit::Visitor};

use crate::test_attribute::validate_test_attributes;

fn check(file: &str, expect: &Expect) {
    let store = PackageStore::new(compile::core());
    let sources = SourceMap::new([("test".into(), file.into())], None);
    let mut unit = compile(
        &store,
        &[],
        sources,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);

    let errors = validate_test_attributes(&mut unit.package);
    Validator::default().visit_package(&unit.package);
    if errors.is_empty() {
        expect.assert_eq(&unit.package.to_string());
    } else {
        expect.assert_debug_eq(&errors);
    }
}

#[test]
fn callable_cant_have_params() {
    check(
        indoc! {"
        namespace test {
            @Test()
            operation A(q : Qubit) : Unit {

            }
        }
        "},
        &expect![[r#"
            [
                CallableHasParameters(
                    Span {
                        lo: 33,
                        hi: 71,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn callable_cant_have_type_params() {
    check(
        indoc! {"
        namespace test {
            @Test()
            operation A<'T>() : Unit {

            }
        }
        "},
        &expect![[r#"
            [
                CallableHaSTypeParameters(
                    Span {
                        lo: 33,
                        hi: 66,
                    },
                ),
            ]
        "#]],
    );
}
