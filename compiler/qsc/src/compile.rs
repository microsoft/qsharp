// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::{Diagnostic, Report};
use qsc_frontend::{
    compile::{CompileUnit, PackageStore, SourceMap, TargetProfile},
    error::WithSource,
};
use qsc_hir::hir::PackageId;
use qsc_passes::{run_core_passes, run_default_passes, PackageType};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub enum Error {
    Frontend(#[from] qsc_frontend::compile::Error),
    Pass(#[from] qsc_passes::Error),
}

#[must_use]
pub fn compile(
    store: &PackageStore,
    dependencies: &[PackageId],
    sources: SourceMap,
    package_type: PackageType,
    target: TargetProfile,
) -> (CompileUnit, Vec<Error>) {
    let mut unit = qsc_frontend::compile::compile(store, dependencies, sources, target);
    let mut errors = Vec::new();
    for error in unit.errors.drain(..) {
        errors.push(error.into());
    }

    if errors.is_empty() {
        for error in run_default_passes(store.core(), &mut unit, package_type, target) {
            errors.push(error.into());
        }
    }

    (unit, errors)
}

/// Compiles the core library.
///
/// # Panics
///
/// Panics if the core library does not compile without errors.
#[must_use]
pub fn core() -> CompileUnit {
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
pub fn std(store: &PackageStore, target: TargetProfile) -> CompileUnit {
    let mut unit = qsc_frontend::compile::std(store, target);
    let pass_errors = run_default_passes(store.core(), &mut unit, PackageType::Lib, target);
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
