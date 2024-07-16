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
        &"basic_manifest".into(),
        &expect![[r#"
            Project {
                name: "basic_manifest",
                path: "basic_manifest/qsharp.json",
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
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
                        language_features: LanguageFeatures(
                            0,
                        ),
                        dependencies: {},
                        package_type: None,
                    },
                    packages: {},
                },
                lints: [],
                errors: [],
            }"#]],
    );
}

#[test]
fn circular_imports() {
    check(
        &"circular_imports".into(),
        &expect![[r#"
            Project {
                name: "circular_imports",
                path: "circular_imports/qsharp.json",
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
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
                        language_features: LanguageFeatures(
                            0,
                        ),
                        dependencies: {},
                        package_type: None,
                    },
                    packages: {},
                },
                lints: [],
                errors: [],
            }"#]],
    );
}

#[test]
fn different_files_same_manifest() {
    check(
        &"different_files_same_manifest".into(),
        &expect![[r#"
            Project {
                name: "different_files_same_manifest",
                path: "different_files_same_manifest/qsharp.json",
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
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
                        language_features: LanguageFeatures(
                            0,
                        ),
                        dependencies: {},
                        package_type: None,
                    },
                    packages: {},
                },
                lints: [],
                errors: [],
            }"#]],
    );
}

#[test]
fn empty_manifest() {
    check(
        &"empty_manifest".into(),
        &expect![[r#"
            Project {
                name: "empty_manifest",
                path: "empty_manifest/qsharp.json",
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
                        sources: [
                            (
                                "empty_manifest/src/Main.qs",
                                "namespace Main {\n    @EntryPoint()\n    operation Main() : String {\n        \"12345\"\n    }\n}\n",
                            ),
                        ],
                        language_features: LanguageFeatures(
                            0,
                        ),
                        dependencies: {},
                        package_type: None,
                    },
                    packages: {},
                },
                lints: [],
                errors: [],
            }"#]],
    );
}

#[test]
fn folder_structure() {
    check(
        &"folder_structure".into(),
        &expect![[r#"
            Project {
                name: "folder_structure",
                path: "folder_structure/qsharp.json",
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
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
                        language_features: LanguageFeatures(
                            0,
                        ),
                        dependencies: {},
                        package_type: None,
                    },
                    packages: {},
                },
                lints: [],
                errors: [],
            }"#]],
    );
}
#[test]
fn hidden_files() {
    check(
        &"hidden_files".into(),
        &expect![[r#"
            Project {
                name: "hidden_files",
                path: "hidden_files/qsharp.json",
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
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
                        language_features: LanguageFeatures(
                            0,
                        ),
                        dependencies: {},
                        package_type: None,
                    },
                    packages: {},
                },
                lints: [],
                errors: [],
            }"#]],
    );
}
#[test]
fn peer_file() {
    check(
        &"peer_file".into(),
        &expect![[r#"
            Project {
                name: "peer_file",
                path: "peer_file/qsharp.json",
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
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
                        language_features: LanguageFeatures(
                            0,
                        ),
                        dependencies: {},
                        package_type: None,
                    },
                    packages: {},
                },
                lints: [],
                errors: [],
            }"#]],
    );
}

#[test]
fn language_feature() {
    check(
        &"language_feature".into(),
        &expect![[r#"
            Project {
                name: "language_feature",
                path: "language_feature/qsharp.json",
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
                        sources: [
                            (
                                "language_feature/src/Project.qs",
                                "namespace Project {\n    @EntryPoint()\n    operation Entry() : Unit {\n        use qs = Qubit[2] { };\n    }\n}\n",
                            ),
                        ],
                        language_features: LanguageFeatures(
                            1,
                        ),
                        dependencies: {},
                        package_type: None,
                    },
                    packages: {},
                },
                lints: [],
                errors: [],
            }"#]],
    );
}

#[test]
fn with_local_dep() {
    check(
        &"with_local_dep".into(),
        &expect![[r#"
            Project {
                name: "with_local_dep",
                path: "with_local_dep/qsharp.json",
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
                        sources: [
                            (
                                "with_local_dep/src/Main.qs",
                                "namespace Main {\n    @EntryPoint()\n    function Main() : Unit {\n        Dependency.LibraryFn();\n    }\n}\n",
                            ),
                        ],
                        language_features: LanguageFeatures(
                            0,
                        ),
                        dependencies: {
                            "MyDep": "{\"path\":\"local_dep\"}",
                        },
                        package_type: None,
                    },
                    packages: {
                        "{\"path\":\"local_dep\"}": PackageInfo {
                            sources: [
                                (
                                    "local_dep/src/Dependency.qs",
                                    "namespace Dependency {\n    function LibraryFn() : Unit {\n    }\n}\n",
                                ),
                            ],
                            language_features: LanguageFeatures(
                                0,
                            ),
                            dependencies: {},
                            package_type: None,
                        },
                    },
                },
                lints: [],
                errors: [],
            }"#]],
    );
}

#[test]
fn transitive_dep() {
    check(
        &"transitive_dep".into(),
        &expect![[r#"
            Project {
                name: "transitive_dep",
                path: "transitive_dep/qsharp.json",
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
                        sources: [
                            (
                                "transitive_dep/src/Main.qs",
                                "namespace Main {\n    @EntryPoint()\n    function Main() : Unit {\n    }\n}\n",
                            ),
                        ],
                        language_features: LanguageFeatures(
                            0,
                        ),
                        dependencies: {
                            "MyDep": "{\"path\":\"with_local_dep\"}",
                        },
                        package_type: None,
                    },
                    packages: {
                        "{\"path\":\"local_dep\"}": PackageInfo {
                            sources: [
                                (
                                    "local_dep/src/Dependency.qs",
                                    "namespace Dependency {\n    function LibraryFn() : Unit {\n    }\n}\n",
                                ),
                            ],
                            language_features: LanguageFeatures(
                                0,
                            ),
                            dependencies: {},
                            package_type: None,
                        },
                        "{\"path\":\"with_local_dep\"}": PackageInfo {
                            sources: [
                                (
                                    "with_local_dep/src/Main.qs",
                                    "namespace Main {\n    @EntryPoint()\n    function Main() : Unit {\n        Dependency.LibraryFn();\n    }\n}\n",
                                ),
                            ],
                            language_features: LanguageFeatures(
                                0,
                            ),
                            dependencies: {
                                "MyDep": "{\"path\":\"local_dep\"}",
                            },
                            package_type: None,
                        },
                    },
                },
                lints: [],
                errors: [],
            }"#]],
    );
}

#[test]
fn explicit_files_list() {
    check(
        &"explicit_files_list".into(),
        &expect![[r#"
            Project {
                name: "explicit_files_list",
                path: "explicit_files_list/qsharp.json",
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
                        sources: [
                            (
                                "explicit_files_list/src/Main.qs",
                                "namespace Dependency {\n    function LibraryFn() : Unit {\n    }\n}\n",
                            ),
                        ],
                        language_features: LanguageFeatures(
                            0,
                        ),
                        dependencies: {},
                        package_type: None,
                    },
                    packages: {},
                },
                lints: [],
                errors: [],
            }"#]],
    );
}

#[test]
fn circular_dep() {
    check(
        &"circular_dep".into(),
        &expect![[r#"
            Project {
                name: "circular_dep",
                path: "circular_dep/qsharp.json",
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
                        sources: [
                            (
                                "circular_dep/src/Main.qs",
                                "namespace Main {\n    @EntryPoint()\n    function Main() : Unit {}\n}\n",
                            ),
                        ],
                        language_features: LanguageFeatures(
                            0,
                        ),
                        dependencies: {
                            "MyCircularDep": "{\"path\":\"circular_dep\"}",
                        },
                        package_type: None,
                    },
                    packages: {},
                },
                lints: [],
                errors: [
                    Circular(
                        "REPLACED",
                        "REPLACED",
                    ),
                ],
            }"#]],
    );
}
