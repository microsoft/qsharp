// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

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
    let mut reports = Vec::new();
    for error in unit.errors.drain(..) {
        reports.push(unit.sources.report(error));
    }

    if reports.is_empty() {
        for error in run_default_passes(&mut unit) {
            reports.push(unit.sources.report(error));
        }
    }

    (unit, reports)
}

pub fn std() -> CompileUnit {
    let mut unit = qsc_frontend::compile::std();
    let pass_errors = run_default_passes(&mut unit);
    if pass_errors.is_empty() {
        unit
    } else {
        for error in pass_errors {
            eprintln!("{:?}", unit.sources.report(error));
        }
        panic!("could not compile standard library")
    }
}
