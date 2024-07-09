// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::{interpret::Interpreter, packages::BuildableProgram};
use indoc::indoc;
use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use qsc_frontend::compile::SourceMap;
use qsc_passes::PackageType;
use qsc_project::{PackageGraphSources, PackageInfo};
use rustc_hash::FxHashMap;

#[test]
fn import_and_call_reexport() {
    let pkg_graph: PackageGraphSources = PackageGraphSources {
        root: PackageInfo {
            sources: vec![(
                "PackageB.qs".into(),
                indoc! {"
                import Foo.DependencyA.Foo;
                function Main() : Unit {
      Foo([1, 2]); 
      Foo.DependencyA.MagicFunction();
  }"}
                .into(),
            )],
            language_features: LanguageFeatures::default(),
            dependencies: [("Foo".into(), "PackageAKey".into())].into_iter().collect(),
            package_type: None,
        },
        packages: [(
            "PackageAKey".into(),
            PackageInfo {
                sources: vec![(
                    "Foo.qs".into(),
                    r#"
namespace DependencyA {
    function MagicFunction() : Unit {
        Message("hello from dependency A!");
    }
    export MagicFunction, Microsoft.Quantum.Core.Length as Foo;
}
                    "#
                    .into(),
                )],
                language_features: LanguageFeatures::default(),
                dependencies: FxHashMap::default(),
                package_type: None,
            },
        )]
        .into_iter()
        .collect(),
    };

    // This builds all the dependencies
    let buildable_program = BuildableProgram::new(TargetCapabilityFlags::all(), pkg_graph);

    assert!(
        buildable_program.dependency_errors.is_empty(),
        "dependencies should be built without errors"
    );

    let BuildableProgram {
        store,
        user_code,
        user_code_dependencies,
        ..
    } = buildable_program;

    let user_code = SourceMap::new(user_code.sources, None);

    let _ = Interpreter::new(
        user_code,
        PackageType::Exe,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
        store,
        &user_code_dependencies,
    )
    .expect("interpreter creation should succeed");
}

#[test]
fn directly_call_reexport() {
    let pkg_graph: PackageGraphSources = PackageGraphSources {
        root: PackageInfo {
            sources: vec![(
                "PackageB.qs".into(),
                indoc! {"
                function Main() : Unit {
      Foo.DependencyA.Foo([1, 2]);
      Foo.DependencyA.MagicFunction();
  }"}
                .into(),
            )],
            language_features: LanguageFeatures::default(),
            dependencies: [("Foo".into(), "PackageAKey".into())].into_iter().collect(),
            package_type: None,
        },
        packages: [(
            "PackageAKey".into(),
            PackageInfo {
                sources: vec![(
                    "Foo.qs".into(),
                    r#"
namespace DependencyA {
    function MagicFunction() : Unit {
        Message("hello from dependency A!");
    }
    export MagicFunction, Microsoft.Quantum.Core.Length as Foo;
}
                    "#
                    .into(),
                )],
                language_features: LanguageFeatures::default(),
                dependencies: FxHashMap::default(),
                package_type: None,
            },
        )]
        .into_iter()
        .collect(),
    };

    // This builds all the dependencies
    let buildable_program = BuildableProgram::new(TargetCapabilityFlags::all(), pkg_graph);

    assert!(
        buildable_program.dependency_errors.is_empty(),
        "dependencies should be built without errors"
    );

    let BuildableProgram {
        store,
        user_code,
        user_code_dependencies,
        ..
    } = buildable_program;

    let user_code = SourceMap::new(user_code.sources, None);

    let _ = Interpreter::new(
        user_code,
        PackageType::Exe,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
        store,
        &user_code_dependencies,
    )
    .expect("interpreter creation should succeed");
}
