// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::test_utils::{
    check_callable_compute_properties, write_compute_properties_to_files, write_fir_store_to_files,
};
use crate::Analyzer;
use expect_test::expect;
use qsc::incremental::Compiler;
use qsc_eval::{debug::map_hir_package_to_fir, lower::Lowerer};
use qsc_fir::fir::PackageStore;
use qsc_frontend::compile::{RuntimeCapabilityFlags, SourceMap};
use qsc_passes::PackageType;

#[test]
fn qubit_allocation_intrinsics_analysis_is_correct() {
    let compiler = Compiler::new(
        false,
        SourceMap::default(),
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
    write_fir_store_to_files(&fir_store); // TODO (cesarzc): for debugging purposes only.
    let analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
    write_compute_properties_to_files(analyzer.get_package_store_compute_properties()); // TODO (cesarzc): for debugging purposes only.
    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__rt__qubit_allocate",
        &expect![
            r#"
        Callable: CallableComputeProperties:
            body: ApplicationsTable:
                inherent: ComputeProperties:
                    runtime_capabilities: RuntimeCapabilityFlags(0x0)
                dynamic_params_properties:
            adj: <none>
            ctl: <none>
            ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__rt__qubit_release",
        &expect![
            r#"
        Callable: CallableComputeProperties:
            body: ApplicationsTable:
                inherent: ComputeProperties:
                    runtime_capabilities: RuntimeCapabilityFlags(0x0)
                dynamic_params_properties:
                    [0]: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
            adj: <none>
            ctl: <none>
            ctl-adj: <none>"#
        ],
    );
}
