// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};
use qsc_hir::{mut_visit::MutVisitor, validate::Validator, visit::Visitor};

use crate::index_assignment::ConvertToWSlash;

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
    ConvertToWSlash {
        assigner: &mut unit.assigner,
    }
    .visit_package(&mut unit.package);
    Validator::default().visit_package(&unit.package);
    expect.assert_eq(&unit.package.to_string());
}

#[test]
fn convert_array_of_array_assign() {
    check(
        indoc! {r#"
        operation Main(arr : Int[]) : Unit {
            mutable arr = [[0, 1], [2, 3]];
            arr[0][1] = 3;
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-93] (Public):
                    Namespace (Ident 24 [0-93] "test"): Item 1
                Item 1 [0-93] (Internal):
                    Parent: 0
                    Callable 0 [0-93] (operation):
                        name: Ident 1 [10-14] "Main"
                        input: Pat 2 [15-26] [Type Int[]]: Bind: Ident 3 [15-18] "arr"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 4 [0-93]: Impl:
                            Block 5 [35-93] [Type Unit]:
                                Stmt 6 [41-72]: Local (Mutable):
                                    Pat 7 [49-52] [Type Int[][]]: Bind: Ident 8 [49-52] "arr"
                                    Expr 9 [55-71] [Type Int[][]]: Array:
                                        Expr 10 [56-62] [Type Int[]]: Array:
                                            Expr 11 [57-58] [Type Int]: Lit: Int(0)
                                            Expr 12 [60-61] [Type Int]: Lit: Int(1)
                                        Expr 13 [64-70] [Type Int[]]: Array:
                                            Expr 14 [65-66] [Type Int]: Lit: Int(2)
                                            Expr 15 [68-69] [Type Int]: Lit: Int(3)
                                Stmt 16 [77-91]: Semi: Expr 17 [77-90] [Type Unit]: AssignIndex:
                                    Expr 19 [77-83] [Type Int[]]: Index:
                                        Expr 20 [77-80] [Type Int[][]]: Var: Local 8
                                        Expr 21 [81-82] [Type Int]: Lit: Int(0)
                                    Expr 22 [84-85] [Type Int]: Lit: Int(1)
                                    Expr 23 [89-90] [Type Int]: Lit: Int(3)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}
