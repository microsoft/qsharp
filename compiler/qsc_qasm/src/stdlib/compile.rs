// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Report;

use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use qsc_frontend::{
    compile::{compile, CompileUnit, PackageStore, SourceContents, SourceMap, SourceName},
    error::WithSource,
};
use qsc_hir::hir::PackageId;
use qsc_passes::{run_core_passes, run_default_passes, PackageType};

#[test]
fn compiles_with_base_profile() {
    let _ = package_store_with_qasm(TargetCapabilityFlags::empty());
}

#[must_use]
pub fn package_store_with_qasm(
    capabilities: TargetCapabilityFlags,
) -> (
    qsc_hir::hir::PackageId,
    qsc_hir::hir::PackageId,
    PackageStore,
) {
    let (std_package_id, mut store) = package_store_with_stdlib(capabilities);
    let mut unit = compile_qasm_std(&store, std_package_id, capabilities);

    let pass_errors = run_default_passes(store.core(), &mut unit, PackageType::Lib);
    if pass_errors.is_empty() {
        //unit.expose();
        let package_id = store.insert(unit);
        (std_package_id, package_id, store)
    } else {
        for error in pass_errors {
            let report = Report::new(WithSource::from_map(&unit.sources, error));
            eprintln!("{report:?}");
        }

        panic!("could not compile qasm standard library")
    }
}

pub const STD_LIB: &[(&str, &str)] = &[
    (
        "openqasm-library-source:QasmStd/Angle.qs",
        include_str!("QasmStd/src/QasmStd/Angle.qs"),
    ),
    (
        "openqasm-library-source:QasmStd/Convert.qs",
        include_str!("QasmStd/src/QasmStd/Convert.qs"),
    ),
    (
        "openqasm-library-source:QasmStd/Intrinsic.qs",
        include_str!("QasmStd/src/QasmStd/Intrinsic.qs"),
    ),
];

/// Compiles the standard library.
///
/// # Panics
///
/// Panics if the standard library does not compile without errors.
#[must_use]
pub fn compile_qasm_std(
    store: &PackageStore,
    std_id: PackageId,
    capabilities: TargetCapabilityFlags,
) -> CompileUnit {
    let std: Vec<(SourceName, SourceContents)> = STD_LIB
        .iter()
        .map(|(name, contents)| ((*name).into(), (*contents).into()))
        .collect();
    let sources = SourceMap::new(std, None);

    let mut unit = compile(
        store,
        &[(PackageId::CORE, None), (std_id, None)],
        sources,
        capabilities,
        LanguageFeatures::default(),
    );
    assert_no_errors(&unit.sources, &mut unit.errors);
    unit
}

fn assert_no_errors(sources: &SourceMap, errors: &mut Vec<qsc_frontend::compile::Error>) {
    if !errors.is_empty() {
        for error in errors.drain(..) {
            eprintln!("{:?}", Report::new(WithSource::from_map(sources, error)));
        }

        panic!("could not compile package");
    }
}

#[must_use]
pub fn package_store_with_stdlib(
    capabilities: TargetCapabilityFlags,
) -> (qsc_hir::hir::PackageId, PackageStore) {
    let mut store = PackageStore::new(core());
    let std_id = store.insert(std(&store, capabilities));
    (std_id, store)
}

/// Compiles the core library.
///
/// # Panics
///
/// Panics if the core library compiles with errors.
#[must_use]
fn core() -> CompileUnit {
    let mut unit = qsc_frontend::compile::core();
    let pass_errors = run_core_passes(&mut unit);
    if pass_errors.is_empty() {
        unit
    } else {
        for error in pass_errors {
            let report = Report::new(WithSource::from_map(&unit.sources, error));
            eprintln!("{report:?}");
        }

        panic!("could not compile core library")
    }
}

/// Compiles the standard library.
///
/// # Panics
///
/// Panics if the standard library does not compile without errors.
#[must_use]
fn std(store: &PackageStore, capabilities: TargetCapabilityFlags) -> CompileUnit {
    let mut unit = qsc_frontend::compile::std(store, capabilities);
    let pass_errors = run_default_passes(store.core(), &mut unit, PackageType::Lib);
    if pass_errors.is_empty() {
        unit
    } else {
        for error in pass_errors {
            let report = Report::new(WithSource::from_map(&unit.sources, error));
            eprintln!("{report:?}");
        }

        panic!("could not compile standard library")
    }
}
