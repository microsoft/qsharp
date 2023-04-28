// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::error::WithSource;
use miette::{Diagnostic, Report};
use qsc_frontend::compile::{CompileUnit, PackageStore, SourceMap};
use qsc_hir::hir::PackageId;
use qsc_passes::run_default_passes;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub enum Error {
    Frontend(#[from] qsc_frontend::compile::Error),
    Pass(#[from] qsc_passes::Error),
}

pub fn compile(
    store: &PackageStore,
    dependencies: impl IntoIterator<Item = PackageId>,
    sources: SourceMap,
) -> (CompileUnit, Vec<Error>) {
    let mut unit = qsc_frontend::compile::compile(store, dependencies, sources);
    let mut errors = Vec::new();
    for error in unit.errors.drain(..) {
        errors.push(error.into());
    }

    if errors.is_empty() {
        for error in run_default_passes(&mut unit) {
            errors.push(error.into());
        }
    }

    (unit, errors)
}

pub fn std() -> CompileUnit {
    let mut unit = qsc_frontend::compile::std();
    let pass_errors = run_default_passes(&mut unit);
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
