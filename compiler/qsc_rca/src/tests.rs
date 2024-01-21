// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::test_utils::{
    check_callable_compute_properties, lower_hir_package_store, write_compute_properties_to_files,
    write_fir_store_to_files,
};
use crate::Analyzer;
use expect_test::expect;
use qsc::incremental::Compiler;
use qsc_eval::debug::map_hir_package_to_fir;
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
    let fir_store = lower_hir_package_store(compiler.package_store());
    let analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
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

#[ignore = "work in progress"]
#[test]
fn core_lib_functions_analysis_is_correct() {
    let compiler = Compiler::new(
        false,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "Length",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "Repeated",
        &expect![r#""#],
    );
}

#[test]
fn std_lib_measurement_intrisics_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__m__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                        quantum_source: Intrinsic
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__mresetz__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                        quantum_source: Intrinsic
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn std_lib_convert_intrisics_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "IntAsDouble",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(IntegerComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "IntAsBigInt",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(IntegerComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn std_lib_diagnostics_intrisics_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "DumpMachine",
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
        "CheckZero",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                        quantum_source: Intrinsic
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn std_lib_available_by_default_intrisics_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "Message",
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

#[test]
fn std_lib_math_intrisics_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "ArcCos",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "ArcSin",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "ArcTan",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "ArcTan2",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "Cos",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "Cosh",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "Sin",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "Sinh",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "Tan",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "Tanh",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "Sqrt",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "Log",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "Truncate",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn std_qir_intrisics_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__ccx__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                        [2]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__cx__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__cy__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__cz__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__rx__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__rxx__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                        [2]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__ry__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__ryy__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                        [2]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__rz__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__rzz__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                        [2]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__h__body",
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

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__s__body",
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

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__s__adj",
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

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__t__body",
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

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__t__adj",
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

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__x__body",
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

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__y__body",
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

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__z__body",
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

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__swap__body",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "__quantum__qis__reset__body",
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

#[test]
fn std_random_intrisics_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "DrawRandomInt",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                        quantum_source: Intrinsic
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(IntegerComputations)
                            quantum_source: Intrinsic
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(IntegerComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "DrawRandomDouble",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                        quantum_source: Intrinsic
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(FloatingPointComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn std_re_intrisics_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "BeginEstimateCaching",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                            quantum_source: Intrinsic
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(IntegerComputations)
                            quantum_source: Intrinsic
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "EndEstimateCaching",
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
        "AccountForEstimatesInternal",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                        [1]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(IntegerComputations)
                        [2]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(HigherLevelConstructs)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "BeginRepeatEstimatesInternal",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_capabilities: RuntimeCapabilityFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_capabilities: RuntimeCapabilityFlags(IntegerComputations)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &fir_store,
        &analyzer.compute_properties,
        "EndRepeatEstimatesInternal",
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
}

#[ignore = "work in progress"]
#[test]
fn static_qubit_allocation_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    write_fir_store_to_files(&fir_store); // TODO (cesarzc): for debugging purposes only.
    let analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
    write_compute_properties_to_files(analyzer.get_package_store_compute_properties());
    // TODO (cesarzc): for debugging purposes only.
}

#[ignore = "work in progress"]
#[test]
fn dynamic_qubit_allocation_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_qubit_array_allocation_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_qubit_array_allocation_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_results_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_results_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn mixed_results_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_bools_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_bools_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn mixed_bools_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_integers_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_integers_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn mixed_integers_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_paulis_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_paulis_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn mixed_paulis_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_ranges_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_ranges_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn mixed_ranges_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_doubles_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_doubles_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn mixed_doubles_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_big_integers_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_big_integers_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn mixed_big_integers_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_strings_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_strings_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn mixed_strings_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_arrays_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_arrays_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn mixed_arrays_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_tuples_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_tuples_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn mixed_tuples_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_udts_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_udts_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn mixed_udts_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn static_arrows_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn dynamic_arrows_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn mixed_arrows_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn functions_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn function_calls_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn operations_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn operation_calls_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn closure_functions_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn closure_function_calls_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn closure_operations_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn closure_operation_calls_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn ifs_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn loops_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}

#[ignore = "work in progress"]
#[test]
fn generics_analysis_is_correct() {
    let compiler = Compiler::new(
        true,
        SourceMap::default(),
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
    )
    .expect("should be able to create a new compiler");
    let fir_store = lower_hir_package_store(compiler.package_store());
    let _analyzer = Analyzer::new(&fir_store, map_hir_package_to_fir(compiler.package_id()));
}
