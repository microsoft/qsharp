// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::error;
use miette::Report;
use qsc_frontend::compile::{CompileUnit, PackageStore, SourceMap};
use qsc_hir::hir::PackageId;
use qsc_passes::run_default_passes;

pub fn compile(
    store: &PackageStore,
    dependencies: impl IntoIterator<Item = PackageId>,
    sources: SourceMap,
) -> (CompileUnit, Vec<Report>) {
    let mut unit = qsc_frontend::compile::compile(store, dependencies, sources);
    let pass_errors = run_default_passes(&mut unit);

    let mut reports = Vec::new();
    if !unit.errors.is_empty() || !pass_errors.is_empty() {
        for error in unit.errors.drain(..) {
            reports.push(error::report(&unit.sources, error));
        }
        for error in pass_errors {
            reports.push(error::report(&unit.sources, error));
        }
    }

    (unit, reports)
}

pub fn std() -> CompileUnit {
    let mut unit = qsc_frontend::compile::std();
    let pass_errors = run_default_passes(&mut unit);

    if !unit.errors.is_empty() || !pass_errors.is_empty() {
        for error in unit.errors {
            eprintln!("{:?}", error::report(&unit.sources, error));
        }
        for error in pass_errors {
            eprintln!("{:?}", error::report(&unit.sources, error));
        }
        panic!("could not compile standard library")
    }

    unit
}
