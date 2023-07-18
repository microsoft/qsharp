// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::error::WithSource;
use miette::{Diagnostic, Report};
use qsc_frontend::compile::{CompileUnit, PackageStore, SourceMap};
use qsc_hir::hir::PackageId;
use qsc_passes::{entry_point::generate_entry_expr, run_core_passes, run_default_passes};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub enum Error {
    Frontend(#[from] qsc_frontend::compile::Error),
    Pass(#[from] qsc_passes::Error),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CheckEntry {
    Required,
    Optional,
}

#[must_use]
pub fn compile(
    store: &PackageStore,
    dependencies: &[PackageId],
    sources: SourceMap,
    entry: CheckEntry,
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
        if entry == CheckEntry::Required {
            for error in generate_entry_expr(&mut unit) {
                errors.push(error.into());
            }
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
