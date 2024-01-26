// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod common;

use common::{check_callable_compute_properties, lower_hir_package_store, CompilationContext};
use expect_test::expect;
use qsc::incremental::Compiler;
use qsc_eval::{debug::map_hir_package_to_fir, lower::Lowerer};
use qsc_frontend::compile::{RuntimeCapabilityFlags, SourceMap};
use qsc_passes::PackageType;
use qsc_rca::PackageStoreComputeProperties;

#[test]
fn qubit_allocation_intrinsics_analysis_is_correct() {
    let compiler = Compiler::new(
        false,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let compute_properties = PackageStoreComputeProperties::new(&fir_store);
    check_callable_compute_properties(
        &fir_store,
        &compute_properties,
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
        &compute_properties,
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
