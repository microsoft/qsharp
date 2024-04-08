// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

mod test_utils;

use expect_test::expect;
use indoc::indoc;
use test_utils::check_rir;

#[test]
fn check_partial_eval_for_calls_to_single_qubit_operations() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                __quantum__qis__h__body(q);
                __quantum__qis__s__body(q);
                __quantum__qis__s__adj(q);
                __quantum__qis__t__body(q);
                __quantum__qis__t__adj(q);
                __quantum__qis__x__body(q);
                __quantum__qis__y__body(q);
                __quantum__qis__x__body(q);
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
                        name: __quantum__qis__h__body
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 2: Callable:
                        name: __quantum__qis__s__body
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 3: Callable:
                        name: __quantum__qis__s__adj
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 4: Callable:
                        name: __quantum__qis__t__body
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 5: Callable:
                        name: __quantum__qis__t__adj
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 6: Callable:
                        name: __quantum__qis__x__body
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 7: Callable:
                        name: __quantum__qis__y__body
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), )
                        Call id(2), args( Qubit(0), )
                        Call id(3), args( Qubit(0), )
                        Call id(4), args( Qubit(0), )
                        Call id(5), args( Qubit(0), )
                        Call id(6), args( Qubit(0), )
                        Call id(7), args( Qubit(0), )
                        Call id(6), args( Qubit(0), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_calls_to_two_qubit_operations() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1) = (Qubit(), Qubit());
                __quantum__qis__swap__body(q0, q1);
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
                        name: __quantum__qis__swap__body
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                            [1]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), Qubit(1), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_calls_to_controlled_operations() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use (ctl0, ctl1, target) = (Qubit(), Qubit(), Qubit());
                __quantum__qis__ccx__body(ctl0, ctl1, target);
                __quantum__qis__cx__body(ctl0, target);
                __quantum__qis__cy__body(ctl0, target);
                __quantum__qis__cz__body(ctl0, target);
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
                        name: __quantum__qis__ccx__body
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                            [1]: Qubit
                            [2]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 2: Callable:
                        name: __quantum__qis__cx__body
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                            [1]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 3: Callable:
                        name: __quantum__qis__cy__body
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                            [1]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 4: Callable:
                        name: __quantum__qis__cz__body
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                            [1]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), Qubit(1), Qubit(2), )
                        Call id(2), args( Qubit(0), Qubit(2), )
                        Call id(3), args( Qubit(0), Qubit(2), )
                        Call id(4), args( Qubit(0), Qubit(2), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_calls_to_rotation_operations() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use (target0, target1) = (Qubit(), Qubit());
                __quantum__qis__rx__body(0.0, target0);
                __quantum__qis__rxx__body(0.0, target0, target1);
                __quantum__qis__ry__body(0.0, target0);
                __quantum__qis__ryy__body(0.0, target0, target1);
                __quantum__qis__rz__body(0.0, target0);
                __quantum__qis__rzz__body(0.0, target0, target1);
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
                    Callable 2: Callable:
                        name: __quantum__qis__rxx__body
                        call_type: Regular
                        input_type:
                            [0]: Double
                            [1]: Qubit
                            [2]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 3: Callable:
                        name: __quantum__qis__ry__body
                        call_type: Regular
                        input_type:
                            [0]: Double
                            [1]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 4: Callable:
                        name: __quantum__qis__ryy__body
                        call_type: Regular
                        input_type:
                            [0]: Double
                            [1]: Qubit
                            [2]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 5: Callable:
                        name: __quantum__qis__rz__body
                        call_type: Regular
                        input_type:
                            [0]: Double
                            [1]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 6: Callable:
                        name: __quantum__qis__rzz__body
                        call_type: Regular
                        input_type:
                            [0]: Double
                            [1]: Qubit
                            [2]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Double(0), Qubit(0), )
                        Call id(2), args( Double(0), Qubit(0), Qubit(1), )
                        Call id(3), args( Double(0), Qubit(0), )
                        Call id(4), args( Double(0), Qubit(0), Qubit(1), )
                        Call id(5), args( Double(0), Qubit(0), )
                        Call id(6), args( Double(0), Qubit(0), Qubit(1), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_call_to_reset() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                __quantum__qis__reset__body(q);
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
                        name: __quantum__qis__reset__body
                        call_type: Reset
                        input_type:
                            [0]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}
