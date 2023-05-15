// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::error::WithSource;
use miette::{Diagnostic, Report};
use qsc_frontend::compile::{CompileUnit, PackageStore, SourceMap};
use qsc_hir::{global, hir::PackageId};
use qsc_passes::run_default_passes;
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
) -> (CompileUnit, Vec<Error>) {
    let mut unit = qsc_frontend::compile::compile(store, dependencies, sources);
    let mut errors = Vec::new();
    for error in unit.errors.drain(..) {
        errors.push(error.into());
    }

    if errors.is_empty() {
        for error in run_default_passes(store.core(), &mut unit) {
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
    let table = global::iter_package(None, &unit.package).collect();
    let pass_errors = run_default_passes(&table, &mut unit);
    if pass_errors.is_empty() {
        unit
    } else {
        for error in pass_errors {
            let report = Report::new(WithSource::from_map(&unit.sources, error, None));
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
pub fn std(store: &PackageStore) -> CompileUnit {
    let mut unit = qsc_frontend::compile::std(store);
    let pass_errors = run_default_passes(store.core(), &mut unit);
    if pass_errors.is_empty() {
        unit
    } else {
        for error in pass_errors {
            let report = Report::new(WithSource::from_map(&unit.sources, error, None));
            eprintln!("{report:?}");
        }

        panic!("could not compile standard library")
    }
}
