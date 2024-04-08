// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

mod test_utils;

use expect_test::expect;
use indoc::indoc;
use test_utils::check_rir;

#[test]
fn check_partial_eval_for_call_to_operation_using_literal() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                __quantum__qis__rx__body(1.0, q);
            }
        }
        "#},
        &expect![[r#"
            Program:
                entry: 0
                callables:
                    Callable 0: Callable:
                        name: main
                        call_type: Regular
                        input_type: <VOID>
                        output_type: <VOID>
                        body: 0
                    Callable 1: Callable:
                        name: __quantum__qis__rx__body
                        call_type: Regular
                        input_type:
                            [0]: Double
                            [1]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Double(1), Qubit(0), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_calls_to_operations_using_inline_expressions() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open Microsoft.Quantum.Math;
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                __quantum__qis__ry__body(PI() * 0.0, q);
                __quantum__qis__ry__body(PI() / PI(), q);
            }
        }
        "#},
        &expect![[r#"
            Program:
                entry: 0
                callables:
                    Callable 0: Callable:
                        name: main
                        call_type: Regular
                        input_type: <VOID>
                        output_type: <VOID>
                        body: 0
                    Callable 1: Callable:
                        name: __quantum__qis__ry__body
                        call_type: Regular
                        input_type:
                            [0]: Double
                            [1]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Double(0), Qubit(0), )
                        Call id(1), args( Double(1), Qubit(0), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_calls_to_operations_using_variables() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open Microsoft.Quantum.Math;
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let pi_over_two = 4.0 / 2.0;
                __quantum__qis__rz__body(pi_over_two, q);
                mutable some_angle = ArcSin(0.0);
                __quantum__qis__rz__body(some_angle, q);
                set some_angle = ArcCos(-1.0) / PI();
                __quantum__qis__rz__body(some_angle, q);
            }
        }
        "#},
        &expect![[r#"
            Program:
                entry: 0
                callables:
                    Callable 0: Callable:
                        name: main
                        call_type: Regular
                        input_type: <VOID>
                        output_type: <VOID>
                        body: 0
                    Callable 1: Callable:
                        name: __quantum__qis__rz__body
                        call_type: Regular
                        input_type:
                            [0]: Double
                            [1]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Double(2), Qubit(0), )
                        Call id(1), args( Double(0), Qubit(0), )
                        Call id(1), args( Double(1), Qubit(0), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}
