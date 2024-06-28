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
fn repro_for_alex() {
    let pkg_graph: PackageGraphSources = PackageGraphSources {
        root: PackageInfo {
            sources: vec![(
                "PackageB.qs".into(),
                indoc! {"
                    import PackageA.PackageA.Foo;
                    @EntryPoint()
                    function Main() : Unit {
                        // use qs = Qubit[2];
                        let len = Foo([]);
                    }"}
                .into(),
            )],
            language_features: LanguageFeatures::default(),
            dependencies: [("PackageA".into(), "PackageAKey".into())]
                .into_iter()
                .collect(),
        },
        packages: [(
            "PackageAKey".into(),
            PackageInfo {
                sources: vec![(
                    "PackageA.qs".into(),
                    "export Microsoft.Quantum.Core.Length as Foo;".into(),
                )],
                language_features: LanguageFeatures::default(),
                dependencies: FxHashMap::default(),
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
