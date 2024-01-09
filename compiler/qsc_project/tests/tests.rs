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
                        "basic_manifest/src/Dependency1.qs",
                        "namespace Dependency1 {\n    function First() : String {\n        \"123\"\n    }\n}\n",
                    ),
                    (
                        "basic_manifest/src/Dependency2.qs",
                        "namespace Dependency2 {\n    function Second() : String {\n        \"45\"\n    }\n}\n",
                    ),
                    (
                        "basic_manifest/src/Main.qs",
                        "namespace Main {\n    open Dependency1;\n    open Dependency2;\n    @EntryPoint()\n    operation Main() : String {\n        First() + Second()\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: Some(
                        "Microsoft",
                    ),
                    license: None,
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
                        "circular_imports/src/Evens.qs",
                        "namespace Evens {\n    open Odds;\n    function Two() : String {\n        \"2\"\n    }\n    function Four() : String {\n        \"4\"\n    }\n    function Twelve() : String {\n        One_() + Two()\n    }\n}\n",
                    ),
                    (
                        "circular_imports/src/Main.qs",
                        "namespace Main {\n    open Evens;\n    open Odds;\n\n    @EntryPoint()\n    operation Main() : String {\n        Twelve() + Three() + FortyFive()\n    }\n}\n",
                    ),
                    (
                        "circular_imports/src/Odds.qs",
                        "namespace Odds {\n    open Evens;\n    function One_() : String {\n        \"1\"\n    }\n    function Three() : String {\n        \"3\"\n    }\n    function Five() : String {\n        \"5\"\n    }\n    function FortyFive() : String {\n        Four() + Five()\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: Some(
                        "Microsoft",
                    ),
                    license: None,
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
                        "different_files_same_manifest/src/Dependency1.qs",
                        "namespace Dependency {\n    function First() : String {\n        \"123\"\n    }\n}\n",
                    ),
                    (
                        "different_files_same_manifest/src/Dependency2.qs",
                        "namespace Dependency {\n    function Second() : String {\n        \"45\"\n    }\n}\n",
                    ),
                    (
                        "different_files_same_manifest/src/Main.qs",
                        "namespace Main {\n    open Dependency;\n    @EntryPoint()\n    operation Main() : String {\n        First() + Second()\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: Some(
                        "Microsoft",
                    ),
                    license: None,
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
                        "empty_manifest/src/Main.qs",
                        "namespace Main {\n    @EntryPoint()\n    operation Main() : String {\n        \"12345\"\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: None,
                    license: None,
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
                        "folder_structure/src/Project.qs",
                        "namespace Project {\n    @EntryPoint()\n    operation Entry() : String {\n        Strings.Concat(\"12\", $\"{(Math.Subtract(346, 1))}\")\n    }\n}\n",
                    ),
                    (
                        "folder_structure/src/utils/ops/Add.qs",
                        "namespace Math {\n    function Add(a: Int, b: Int) : Int {\n        a + b\n    }\n}\n",
                    ),
                    (
                        "folder_structure/src/utils/ops/Subtract.qs",
                        "namespace Math {\n    function Subtract(a: Int, b: Int) : Int {\n        a - b\n    }\n}\n",
                    ),
                    (
                        "folder_structure/src/utils/strings/Concat.qs",
                        "namespace Strings {\n    function Concat(a: String, b: String) : String {\n        a + b\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: None,
                    license: None,
                },
            }"#]],
    )
}
#[test]
fn hidden_files() {
    check(
        "hidden_files".into(),
        &expect![[r#"
            Project {
                sources: [
                    (
                        "hidden_files/src/Project.qs",
                        "namespace Project {\n    @EntryPoint()\n    operation Entry() : String {\n        Strings.Concat(\"12\", $\"{(Math.Subtract(346, 1))}\")\n    }\n}\n",
                    ),
                    (
                        "hidden_files/src/utils/ops/Add.qs",
                        "namespace Math {\n    function Add(a: Int, b: Int) : Int {\n        a + b\n    }\n}\n",
                    ),
                    (
                        "hidden_files/src/utils/ops/Subtract.qs",
                        "namespace Math {\n    function Subtract(a: Int, b: Int) : Int {\n        a - b\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: None,
                    license: None,
                },
            }"#]],
    )
}
#[test]
fn peer_file() {
    check(
        "peer_file".into(),
        &expect![[r#"
            Project {
                sources: [
                    (
                        "peer_file/src/Project.qs",
                        "namespace Project {\n    @EntryPoint()\n    operation Entry() : String {\n        Strings.Concat(\"12\", $\"{(Math.Subtract(346, 1))}\")\n    }\n}\n",
                    ),
                    (
                        "peer_file/src/utils/ops/Add.qs",
                        "namespace Math {\n    function Add(a: Int, b: Int) : Int {\n        a + b\n    }\n}\n",
                    ),
                    (
                        "peer_file/src/utils/ops/Subtract.qs",
                        "namespace Math {\n    function Subtract(a: Int, b: Int) : Int {\n        a - b\n    }\n}\n",
                    ),
                    (
                        "peer_file/src/utils/strings/Concat.qs",
                        "namespace Strings {\n    function Concat(a: String, b: String) : String {\n        a + b\n    }\n}\n",
                    ),
                ],
                manifest: Manifest {
                    author: None,
                    license: None,
                },
            }"#]],
    )
}
