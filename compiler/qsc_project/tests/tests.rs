// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! These tests exist solely to test project structure and contents.
//! They are not intended to test the qsc runtime or compilation process,
//! so we are only asserting file names and contents.

mod harness;

use expect_test::expect;
use harness::check;

#[test]
fn basic_manifest() {
    check(
        "basic_manifest".into(),
        &expect![[r#"
            Project {
                sources: [
                    (
                        "basic_manifest/Dependency1.qs",
                        "namespace Dependency1 {\n    function First() : String {\n        \"123\"\n    }\n}\n",
                    ),
                    (
                        "basic_manifest/Dependency2.qs",
                        "namespace Dependency2 {\n    function Second() : String {\n        \"45\"\n    }\n}\n",
                    ),
                    (
                        "basic_manifest/Main.qs",
                        "namespace Main {\n    open Dependency1;\n    open Dependency2;\n    @EntryPoint()\n    operation Main() : String {\n        First() + Second()\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: Some(
                        "Microsoft",
                    ),
                    license: None,
                    exclude_regexes: [],
                    exclude_files: [],
                },
            }"#]],
    )
}

#[test]
fn circular_imports() {
    check(
        "circular_imports".into(),
        &expect![[r#"
            Project {
                sources: [
                    (
                        "circular_imports/Evens.qs",
                        "namespace Evens {\n    open Odds;\n    function Two() : String {\n        \"2\"\n    }\n    function Four() : String {\n        \"4\"\n    }\n    function Twelve() : String {\n        One_() + Two()\n    }\n}\n",
                    ),
                    (
                        "circular_imports/Main.qs",
                        "namespace Main {\n    open Evens;\n    open Odds;\n\n    @EntryPoint()\n    operation Main() : String {\n        Twelve() + Three() + FortyFive()\n    }\n}\n",
                    ),
                    (
                        "circular_imports/Odds.qs",
                        "namespace Odds {\n    open Evens;\n    function One_() : String {\n        \"1\"\n    }\n    function Three() : String {\n        \"3\"\n    }\n    function Five() : String {\n        \"5\"\n    }\n    function FortyFive() : String {\n        Four() + Five()\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: Some(
                        "Microsoft",
                    ),
                    license: None,
                    exclude_regexes: [],
                    exclude_files: [],
                },
            }"#]],
    )
}

#[test]
fn different_files_same_manifest() {
    check(
        "different_files_same_manifest".into(),
        &expect![[r#"
            Project {
                sources: [
                    (
                        "different_files_same_manifest/Dependency1.qs",
                        "namespace Dependency {\n    function First() : String {\n        \"123\"\n    }\n}\n",
                    ),
                    (
                        "different_files_same_manifest/Dependency2.qs",
                        "namespace Dependency {\n    function Second() : String {\n        \"45\"\n    }\n}\n",
                    ),
                    (
                        "different_files_same_manifest/Main.qs",
                        "namespace Main {\n    open Dependency;\n    @EntryPoint()\n    operation Main() : String {\n        First() + Second()\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: Some(
                        "Microsoft",
                    ),
                    license: None,
                    exclude_regexes: [],
                    exclude_files: [],
                },
            }"#]],
    )
}

#[test]
fn empty_manifest() {
    check(
        "empty_manifest".into(),
        &expect![[r#"
            Project {
                sources: [
                    (
                        "empty_manifest/Main.qs",
                        "namespace Main {\n    @EntryPoint()\n    operation Main() : String {\n        \"12345\"\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: None,
                    license: None,
                    exclude_regexes: [],
                    exclude_files: [],
                },
            }"#]],
    )
}

#[test]
fn exclude_blobs() {
    check(
        "exclude_blobs".into(),
        &expect![[r#"
            Project {
                sources: [
                    (
                        "exclude_blobs/Main.qs",
                        "namespace Main {\n    @EntryPoint()\n    operation Main() : String {\n        Numbers.Numbers()\n    }\n}\n",
                    ),
                    (
                        "exclude_blobs/to_include/Numbers.qs",
                        "namespace Numbers {\n    operation Numbers() : String {\n        \"12345\"\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: None,
                    license: None,
                    exclude_regexes: [
                        ".*to_exclude.*",
                    ],
                    exclude_files: [],
                },
            }"#]],
    )
}

#[test]
fn exclude_files() {
    check(
        "exclude_files".into(),
        &expect![[r#"
            Project {
                sources: [
                    (
                        "exclude_files/Main.qs",
                        "namespace Main {\n    @EntryPoint()\n    operation Main() : String {\n        \"12345\"\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: None,
                    license: None,
                    exclude_regexes: [
                        ".*\\.exclude\\.qs",
                    ],
                    exclude_files: [],
                },
            }"#]],
    )
}

#[test]
fn exclude_files_with_regex() {
    check(
        "exclude_files_with_regex".into(),
        &expect![[r#"
            Project {
                sources: [
                    (
                        "exclude_files/Main.qs",
                        "namespace Main {\n    @EntryPoint()\n    operation Main() : String {\n        \"12345\"\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: None,
                    license: None,
                    exclude_regexes: [
                        ".*\\.exclude\\.qs",
                    ],
                    exclude_files: [],
                },
            }"#]],
    )
}

#[test]
fn exclude_list() {
    check(
        "exclude_list".into(),
        &expect![[r#"
            Project {
                sources: [
                    (
                        "exclude_list/Included.qs",
                        "namespace Numbers {\n    operation OneTwoThreeFourFive() : String {\n        \"12345\"\n    }\n}\n",
                    ),
                    (
                        "exclude_list/Main.qs",
                        "namespace Main {\n    open Numbers;\n    @EntryPoint()\n    operation Main() : String {\n        OneTwoThreeFourFive()\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: None,
                    license: None,
                    exclude_regexes: [
                        ".*Excluded.qs",
                    ],
                    exclude_files: [],
                },
            }"#]],
    )
}

#[test]
fn folder_structure() {
    check(
        "folder_structure".into(),
        &expect![[r#"
            Project {
                sources: [
                    (
                        "folder_structure/Project.qs",
                        "namespace Project {\n    @EntryPoint()\n    operation Entry() : String {\n        Strings.Concat(\"12\", $\"{(Math.Subtract(346, 1))}\")\n    }\n}\n",
                    ),
                    (
                        "folder_structure/utils/ops/Add.qs",
                        "namespace Math {\n    function Add(a: Int, b: Int) : Int {\n        a + b\n    }\n}\n",
                    ),
                    (
                        "folder_structure/utils/ops/Subtract.qs",
                        "namespace Math {\n    function Subtract(a: Int, b: Int) : Int {\n        a - b\n    }\n}\n",
                    ),
                    (
                        "folder_structure/utils/strings/Concat.qs",
                        "namespace Strings {\n    function Concat(a: String, b: String) : String {\n        a + b\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: None,
                    license: None,
                    exclude_regexes: [],
                    exclude_files: [],
                },
            }"#]],
    )
}
