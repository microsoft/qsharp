// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

mod test_utils;

use expect_test::expect;
use indoc::indoc;
use test_utils::check_rir;

#[test]
fn check_partial_eval_for_allocate_use_release_one_qubit() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            open QIR.Runtime;
            @EntryPoint()
            operation Main() : Unit {
                let q = __quantum__rt__qubit_allocate();
                __quantum__qis__h__body(q);
                __quantum__rt__qubit_release(q);
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

#[test]
fn check_partial_eval_for_allocate_use_release_multiple_qubits() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            open QIR.Runtime;
            @EntryPoint()
            operation Main() : Unit {
                let q0 = __quantum__rt__qubit_allocate();
                let q1 = __quantum__rt__qubit_allocate();
                let q2 = __quantum__rt__qubit_allocate();
                __quantum__qis__h__body(q0);
                __quantum__qis__h__body(q1);
                __quantum__qis__h__body(q2);
                __quantum__rt__qubit_release(q2);
                __quantum__rt__qubit_release(q1);
                __quantum__rt__qubit_release(q0);
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
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(1), )
                        Call id(1), args( Qubit(2), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_allocate_use_release_one_qubit_multiple_times() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            open QIR.Runtime;
            @EntryPoint()
            operation Main() : Unit {
                let q0 = __quantum__rt__qubit_allocate();
                __quantum__qis__h__body(q0);
                __quantum__rt__qubit_release(q0);
                let q1 = __quantum__rt__qubit_allocate();
                __quantum__qis__h__body(q1);
                __quantum__rt__qubit_release(q1);
                let q2 = __quantum__rt__qubit_allocate();
                __quantum__qis__h__body(q2);
                __quantum__rt__qubit_release(q2);
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
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(0), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_allocate_use_release_multiple_qubits_interleaved() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            open QIR.Runtime;
            @EntryPoint()
            operation Main() : Unit {
                let q0 = __quantum__rt__qubit_allocate();
                __quantum__qis__h__body(q0);
                let q1 = __quantum__rt__qubit_allocate();
                __quantum__qis__h__body(q1);
                let q2 = __quantum__rt__qubit_allocate();
                __quantum__qis__h__body(q2);
                __quantum__rt__qubit_release(q2);
                let q3 = __quantum__rt__qubit_allocate();
                let q4 = __quantum__rt__qubit_allocate();
                __quantum__qis__h__body(q3);
                __quantum__qis__h__body(q4);
                __quantum__rt__qubit_release(q4);
                __quantum__rt__qubit_release(q3);
                __quantum__rt__qubit_release(q1);
                __quantum__rt__qubit_release(q0);
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
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(1), )
                        Call id(1), args( Qubit(2), )
                        Call id(1), args( Qubit(2), )
                        Call id(1), args( Qubit(3), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}
