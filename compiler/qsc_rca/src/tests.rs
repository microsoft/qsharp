// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::Analyzer;
use qsc::incremental::Compiler;
use qsc_eval::{debug::map_hir_package_to_fir, lower::Lowerer};
use qsc_fir::fir::PackageStore;
use qsc_frontend::compile::{RuntimeCapabilityFlags, SourceMap};
use qsc_passes::PackageType;

#[test]
fn core_library_analysis_is_correct() {
    let compiler = Compiler::new(
        false,
        SourceMap::new(vec![], None),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let mut lowerer = Lowerer::new();
    let mut fir_store = PackageStore::new();
    for (id, unit) in compiler.package_store() {
        fir_store.insert(
            map_hir_package_to_fir(id),
            lowerer.lower_package(&unit.package),
        );
    }
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}
