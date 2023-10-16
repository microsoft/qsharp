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
                    "basic_manifest/Dependency2.qs",
                    "namespace Dependency2 {\n\tfunction Second() : String {\n\t\t\"45\"\n\t}\n}\n",
                ),
                (
                    "basic_manifest/Main.qs",
                    "namespace Main {\n\topen Dependency1;\n\topen Dependency2;\n\t@EntryPoint()\n\toperation Main() : String {\n\t\tFirst() + Second()\n\t}\n}\n",
                ),
                (
                    "basic_manifest/Dependency1.qs",
                    "namespace Dependency1 {\n\tfunction First() : String {\n\t\t\"123\"\n\t}\n}\n",
                ),
            ],
            manifest: Manifest {
                author: Some(
                    "Microsoft",
                ),
                license: None,
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
                    "circular_imports/Odds.qs",
                    "namespace Odds {\n\topen Evens;\n\tfunction One_() : String {\n\t\t\"1\"\n\t}\n\tfunction Three() : String {\n\t\t\"3\"\n\t}\n\tfunction Five() : String {\n\t\t\"5\"\n\t}\n\tfunction FortyFive() : String {\n\t\tFour() + Five()\n\t} \n}\n",
                ),
                (
                    "circular_imports/Evens.qs",
                    "namespace Evens {\n\topen Odds;\n\tfunction Two() : String {\n\t\t\"2\"\n\t}\n\tfunction Four() : String {\n\t\t\"4\"\n\t}\n\tfunction Twelve() : String {\n\t\tOne_() + Two()\n\t}\n}\n",
                ),
                (
                    "circular_imports/Main.qs",
                    "namespace Main {\n\topen Evens;\n\topen Odds;\n\n\t@EntryPoint()\n\toperation Main() : String {\n\t\tTwelve() + Three() + FortyFive()\n\t}\n}\n",
                ),
            ],
            manifest: Manifest {
                author: Some(
                    "Microsoft",
                ),
                license: None,
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
                    "different_files_same_manifest/Dependency2.qs",
                    "namespace Dependency {\n\tfunction Second() : String {\n\t\t\"45\"\n\t}\n}\n",
                ),
                (
                    "different_files_same_manifest/Main.qs",
                    "namespace Main {\n\topen Dependency;\n\t@EntryPoint()\n\toperation Main() : String {\n\t\tFirst() + Second()\n\t}\n}\n",
                ),
                (
                    "different_files_same_manifest/Dependency1.qs",
                    "namespace Dependency {\n\tfunction First() : String {\n\t\t\"123\"\n\t}\n}\n",
                ),
            ],
            manifest: Manifest {
                author: Some(
                    "Microsoft",
                ),
                license: None,
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
                    "namespace Main {\n\t@EntryPoint()\n\toperation Main() : String {\n\t\t\"12345\"\n\t}\n}\n",
                ),
            ],
            manifest: Manifest {
                author: None,
                license: None,
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
                    "exclude_blobs/to_include/Numbers.qs",
                    "namespace Numbers {\n\toperation Numbers() : String {\n\t\t\"12345\"\n\t}\n}\n",
                ),
                (
                    "exclude_blobs/Main.qs",
                    "namespace Main {\n\t@EntryPoint()\n\toperation Main() : String {\n\t\tNumbers.Numbers()\n\t}\n}\n",
                ),
            ],
            manifest: Manifest {
                author: None,
                license: None,
                exclude_files: [
                    "to_exclude/*",
                ],
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
                    "namespace Main {\n\t@EntryPoint()\n\toperation Main() : String {\n\t\t\"12345\"\n\t}\n}\n",
                ),
            ],
            manifest: Manifest {
                author: None,
                license: None,
                exclude_files: [
                    "*.exclude.qs",
                ],
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
                    "namespace Numbers {\n\toperation OneTwoThreeFourFive() : String {\n\t\t\"12345\"\n\t}\n}\n",
                ),
                (
                    "exclude_list/Main.qs",
                    "namespace Main {\n\topen Numbers;\n\t@EntryPoint()\n\toperation Main() : String {\n\t\tOneTwoThreeFourFive()\n\t}\n} \n",
                ),
            ],
            manifest: Manifest {
                author: None,
                license: None,
                exclude_files: [
                    "Excluded.qs",
                ],
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
                    "namespace Project {\n\t@EntryPoint()\n\toperation Entry() : String {\n\t\tStrings.Concat(\"12\", $\"{(Math.Subtract(346, 1))}\")\n\t}\n}\n",
                ),
                (
                    "folder_structure/utils/strings/Concat.qs",
                    "namespace Strings {\n\tfunction Concat(a: String, b: String) : String {\n\t\ta + b\n\t}\n}\n",
                ),
                (
                    "folder_structure/utils/ops/Subtract.qs",
                    "namespace Math {\n\tfunction Subtract(a: Int, b: Int) : Int {\n\t\ta - b\n\t}\n}\n",
                ),
                (
                    "folder_structure/utils/ops/Add.qs",
                    "namespace Math {\n\tfunction Add(a: Int, b: Int) : Int {\n\t\ta + b\n\t}\n}\n",
                ),
            ],
            manifest: Manifest {
                author: None,
                license: None,
                exclude_files: [],
            },
        }"#]],
    )
}
