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
fn check_rca_for_direct_function_cycle() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
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

#[test]
fn check_rca_for_function_cycle_within_binding() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
            let out = Foo(i);
            return out;
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
fn check_rca_for_function_cycle_within_assignment() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
            mutable out = 0;
            set out = Foo(i);
            return out;
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
fn check_rca_for_function_cycle_within_return() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
            return Foo(i);
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
fn check_rca_for_function_cycle_within_tuple() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
            let (a, b) = (Foo(0), Foo(1));
            return a + b;
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
fn check_rca_for_function_cycle_within_call_input() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Arrays;
        function MySorted<'T>(comparison : (('T, 'T) -> Bool), array : 'T[]) : 'T[] {
            if Length(array) <= 1 {
                return array;
            }
            let pivotIndex = Length(array) / 2;
            let left = array[...pivotIndex - 1];
            let right = array[pivotIndex...];
            MySortedMerged(
                comparison,
                MySorted(comparison, left),
                MySorted(comparison, right)
            )
        }
        internal function MySortedMerged<'T>(comparison : (('T, 'T) -> Bool), left : 'T[], right : 'T[]) : 'T[] {
            mutable output = [];
            mutable remainingLeft = left;
            mutable remainingRight = right;
            while (not IsEmpty(remainingLeft)) and (not IsEmpty(remainingRight)) {
                if comparison(Head(remainingLeft), Head(remainingRight)) {
                    set output += [Head(remainingLeft)];
                    set remainingLeft = Rest(remainingLeft);
                } else {
                    set output += [Head(remainingRight)];
                    set remainingRight = Rest(remainingRight);
                }
            }
            output + remainingLeft + remainingRight
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "MySorted",
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

#[test]
fn check_rca_for_function_cycle_within_if_block() {
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
fn check_rca_for_function_cycle_within_if_condition() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
            if (Foo(i) > 0) {
                1
            } else {
                0
            }
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
fn check_rca_for_function_cycle_within_for_block() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
            for _ in 0..10 {
                Foo(i);
            }
            0
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
fn check_rca_for_function_cycle_within_while_block() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
            while true {
                Foo(i);
            }
            0
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
fn check_rca_for_function_cycle_within_while_condition() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(i: Int) : Int {
            while Foo(i) > 0{
            }
            0
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
    let compilation_context = CompilationContext::new();
    write_fir_store_to_files(&compilation_context.fir_store); // TODO (cesarzc): Remove.
    write_compute_properties_to_files(&compilation_context.compute_properties); // TODO (cesarzc): Remove.
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
