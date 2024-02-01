// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod common;

use common::{
    check_callable_compute_properties, write_compute_properties_to_files, write_fir_store_to_files,
    CompilationContext,
};
use expect_test::expect;

#[test]
fn check_rca_for_parameterless_one_function_cycle() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo() : Unit {
            Foo();
        }"#,
    );

    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_parameterless_two_functions_cycle() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo() : Unit {
            Bar();
        }
        function Bar() : Unit {
            Foo();
        }"#,
    );

    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );

    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Bar",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_parameterless_three_functions_cycle() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo() : Unit {
            Bar();
        }
        function Bar() : Unit {
            Baz();
        }
        function Baz() : Unit {
            Foo();
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Bar",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Baz",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_indirect_function_cycle() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
            let f = Foo;
            f(i)
        }"#,
    );

    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_indirect_chain_function_cycle() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
            let a = Foo;
            let b = a;
            let c = b;
            c(i)
        }"#,
    );

    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_indirect_closure_function_cycle() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
            let f = () -> Foo(0);
            f()
        }"#,
    );

    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_indirect_partial_appplication_function_cycle() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(b: Bool, i: Int) : Int {
            let f = Foo(false, _);
            f(0)
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                        [1]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_function_cycle_within_if() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
            if (i > 0) {
                Foo(i - 1)
            } else {
                0
            }
        }"#,
    );
    write_fir_store_to_files(&compilation_context.fir_store); // TODO (cesarzc): Remove.
    write_compute_properties_to_files(&compilation_context.compute_properties); // TODO (cesarzc): Remove.
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_result_param_recursive_function() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(r : Result) : Unit {
            Foo(r);
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_result_inout_recursive_function() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(r : Result) : Result {
            Foo(r)
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_bool_param_recursive_function() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(b : Bool) : Unit {
            Foo(b);
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_bool_inout_recursive_function() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(b : Bool) : Bool {
            Foo(b)
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_int_param_recursive_function() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i : Int) : Unit {
            Foo(i);
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_int_inout_recursive_function() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i : Int) : Int {
            Foo(i)
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_double_param_recursive_function() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(d : Double) : Unit {
            Foo(d);
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_double_inout_recursive_function() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(d : Double) : Double {
            Foo(d)
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_multi_param_recursive_function() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(p : Pauli, s: String[], t: (Range, BigInt)) : Unit {
            Foo(p, s, t);
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                        [1]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                        [2]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[test]
fn check_rca_for_multi_param_result_out_recursive_function() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(p : Pauli, s: String[], t: (Range, BigInt)) : Result {
            Foo(p, s, t)
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsTable:
                    inherent: ComputeProperties:
                        runtime_features: RuntimeFeatureFlags(0x0)
                    dynamic_params_properties:
                        [0]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                        [1]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                        [2]: ComputeProperties:
                            runtime_features: RuntimeFeatureFlags(CycledFunctionApplicationUsesDynamicArg)
                            dynamism_sources: [Assumed]
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_operation_adj_recursion() {
    let mut _compilation_context = CompilationContext::new();
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_operation_ctl_recursion() {
    let mut _compilation_context = CompilationContext::new();
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_operation_ctl_adj_recursion() {
    let mut _compilation_context = CompilationContext::new();
}
