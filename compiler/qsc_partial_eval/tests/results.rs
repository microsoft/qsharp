// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

mod test_utils;

use expect_test::expect;
use indoc::indoc;
use test_utils::check_rir;

#[test]
fn check_partial_eval_for_measuring_and_resetting_one_qubit() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                __quantum__qis__mresetz__body(q);
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
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                    Callable 1: Callable:
                        name: __quantum__qis__mresetz__body
                        call_type: Measurement
                        input_type: 
                            [0]: Qubit
                            [1]: Result
                        output_type:  <VOID>
                        body:  <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), Result(0), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_measuring_one_qubit() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                __quantum__qis__m__body(q);
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
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                    Callable 1: Callable:
                        name: __quantum__qis__mz__body
                        call_type: Measurement
                        input_type: 
                            [0]: Qubit
                            [1]: Result
                        output_type:  <VOID>
                        body:  <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), Result(0), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_measuring_one_qubit_multiple_times() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                __quantum__qis__m__body(q);
                __quantum__qis__m__body(q);
                __quantum__qis__m__body(q);
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
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                    Callable 1: Callable:
                        name: __quantum__qis__mz__body
                        call_type: Measurement
                        input_type: 
                            [0]: Qubit
                            [1]: Result
                        output_type:  <VOID>
                        body:  <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), Result(0), )
                        Call id(1), args( Qubit(0), Result(1), )
                        Call id(1), args( Qubit(0), Result(2), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_measuring_multiple_qubits() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1, q2) = (Qubit(), Qubit(), Qubit());
                __quantum__qis__m__body(q0);
                __quantum__qis__m__body(q1);
                __quantum__qis__m__body(q2);
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
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                    Callable 1: Callable:
                        name: __quantum__qis__mz__body
                        call_type: Measurement
                        input_type: 
                            [0]: Qubit
                            [1]: Result
                        output_type:  <VOID>
                        body:  <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), Result(0), )
                        Call id(1), args( Qubit(1), Result(1), )
                        Call id(1), args( Qubit(2), Result(2), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_comparing_measurement_results() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1) = (Qubit(), Qubit());
                let r0 = __quantum__qis__m__body(q0);
                let r1 = __quantum__qis__mresetz__body(q1);
                let b = r0 == r1;
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
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                    Callable 1: Callable:
                        name: __quantum__qis__mz__body
                        call_type: Measurement
                        input_type: 
                            [0]: Qubit
                            [1]: Result
                        output_type:  <VOID>
                        body:  <NONE>
                    Callable 2: Callable:
                        name: __quantum__qis__mresetz__body
                        call_type: Measurement
                        input_type: 
                            [0]: Qubit
                            [1]: Result
                        output_type:  <VOID>
                        body:  <NONE>
                    Callable 3: Callable:
                        name: __quantum__rt__read_result__body
                        call_type: Readout
                        input_type: 
                            [0]: Result
                        output_type:  Boolean
                        body:  <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), Result(0), )
                        Call id(2), args( Qubit(1), Result(1), )
                        Variable(0, Boolean) = Call id(3), args( Result(0), )
                        Variable(1, Boolean) = Call id(3), args( Result(1), )
                        Variable(2, Boolean) = Icmp Eq, Variable(0, Boolean), Variable(1, Boolean)
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn check_partial_eval_for_comparing_measurement_result_to_result_literal() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = __quantum__qis__m__body(q);
                let b = r == One;
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
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                    Callable 1: Callable:
                        name: __quantum__qis__mz__body
                        call_type: Measurement
                        input_type: 
                            [0]: Qubit
                            [1]: Result
                        output_type:  <VOID>
                        body:  <NONE>
                    Callable 2: Callable:
                        name: __quantum__rt__read_result__body
                        call_type: Readout
                        input_type: 
                            [0]: Result
                        output_type:  Boolean
                        body:  <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), Result(0), )
                        Variable(0, Boolean) = Call id(2), args( Result(0), )
                        Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(true)
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}
