// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use expect_test::expect;
use indoc::indoc;
use qsc_rir::rir::{BlockId, CallableId};
use test_utils::{assert_block_instructions, assert_callable, get_rir_program};

#[test]
fn result_ids_are_correct_for_measuring_and_resetting_one_qubit() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                QIR.Intrinsic.__quantum__qis__mresetz__body(q)
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    let result_record_id = CallableId(2);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__mresetz__body
            call_type: Measurement
            input_type:
                [0]: Qubit
                [1]: Result
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_callable(
        &program,
        result_record_id,
        &expect![[r#"
        Callable:
            name: __quantum__rt__result_record_output
            call_type: OutputRecording
            input_type:
                [0]: Result
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Result(0), )
                Call id(2), args( Result(0), Pointer, )
                Return"#]],
    );
    assert_eq!(program.num_qubits, 1);
    assert_eq!(program.num_results, 1);
}

#[test]
fn result_ids_are_correct_for_measuring_one_qubit() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                QIR.Intrinsic.__quantum__qis__m__body(q)
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    let result_record_id = CallableId(2);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__mz__body
            call_type: Measurement
            input_type:
                [0]: Qubit
                [1]: Result
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_callable(
        &program,
        result_record_id,
        &expect![[r#"
        Callable:
            name: __quantum__rt__result_record_output
            call_type: OutputRecording
            input_type:
                [0]: Result
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Result(0), )
                Call id(2), args( Result(0), Pointer, )
                Return"#]],
    );
    assert_eq!(program.num_qubits, 1);
    assert_eq!(program.num_results, 1);
}

#[test]
fn result_ids_are_correct_for_measuring_one_qubit_multiple_times() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : (Result, Result, Result) {
                use q = Qubit();
                (QIR.Intrinsic.__quantum__qis__m__body(q),
                QIR.Intrinsic.__quantum__qis__m__body(q),
                QIR.Intrinsic.__quantum__qis__m__body(q))
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    let tuple_record_id = CallableId(2);
    let result_record_id = CallableId(3);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__mz__body
            call_type: Measurement
            input_type:
                [0]: Qubit
                [1]: Result
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_callable(
        &program,
        tuple_record_id,
        &expect![[r#"
        Callable:
            name: __quantum__rt__tuple_record_output
            call_type: OutputRecording
            input_type:
                [0]: Integer
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_callable(
        &program,
        result_record_id,
        &expect![[r#"
        Callable:
            name: __quantum__rt__result_record_output
            call_type: OutputRecording
            input_type:
                [0]: Result
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Result(0), )
                Call id(1), args( Qubit(0), Result(1), )
                Call id(1), args( Qubit(0), Result(2), )
                Call id(2), args( Integer(3), Pointer, )
                Call id(3), args( Result(0), Pointer, )
                Call id(3), args( Result(1), Pointer, )
                Call id(3), args( Result(2), Pointer, )
                Return"#]],
    );
}

#[test]
fn result_ids_are_correct_for_measuring_one_qubit_multiple_times_into_array() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result[] {
                use q = Qubit();
                [QIR.Intrinsic.__quantum__qis__m__body(q),
                QIR.Intrinsic.__quantum__qis__m__body(q),
                QIR.Intrinsic.__quantum__qis__m__body(q)]
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    let array_record_id = CallableId(2);
    let result_record_id = CallableId(3);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__mz__body
            call_type: Measurement
            input_type:
                [0]: Qubit
                [1]: Result
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_callable(
        &program,
        array_record_id,
        &expect![[r#"
        Callable:
            name: __quantum__rt__array_record_output
            call_type: OutputRecording
            input_type:
                [0]: Integer
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_callable(
        &program,
        result_record_id,
        &expect![[r#"
        Callable:
            name: __quantum__rt__result_record_output
            call_type: OutputRecording
            input_type:
                [0]: Result
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Result(0), )
                Call id(1), args( Qubit(0), Result(1), )
                Call id(1), args( Qubit(0), Result(2), )
                Call id(2), args( Integer(3), Pointer, )
                Call id(3), args( Result(0), Pointer, )
                Call id(3), args( Result(1), Pointer, )
                Call id(3), args( Result(2), Pointer, )
                Return"#]],
    );
    assert_eq!(program.num_qubits, 1);
    assert_eq!(program.num_results, 3);
}

#[test]
fn result_ids_are_correct_for_measuring_multiple_qubits() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : (Result, Result, Result) {
                use (q0, q1, q2) = (Qubit(), Qubit(), Qubit());
                (QIR.Intrinsic.__quantum__qis__m__body(q0),
                QIR.Intrinsic.__quantum__qis__m__body(q1),
                QIR.Intrinsic.__quantum__qis__m__body(q2))
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    let tuple_record_id = CallableId(2);
    let result_record_id = CallableId(3);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__mz__body
            call_type: Measurement
            input_type:
                [0]: Qubit
                [1]: Result
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_callable(
        &program,
        tuple_record_id,
        &expect![[r#"
        Callable:
            name: __quantum__rt__tuple_record_output
            call_type: OutputRecording
            input_type:
                [0]: Integer
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_callable(
        &program,
        result_record_id,
        &expect![[r#"
        Callable:
            name: __quantum__rt__result_record_output
            call_type: OutputRecording
            input_type:
                [0]: Result
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Result(0), )
                Call id(1), args( Qubit(1), Result(1), )
                Call id(1), args( Qubit(2), Result(2), )
                Call id(2), args( Integer(3), Pointer, )
                Call id(3), args( Result(0), Pointer, )
                Call id(3), args( Result(1), Pointer, )
                Call id(3), args( Result(2), Pointer, )
                Return"#]],
    );
    assert_eq!(program.num_qubits, 3);
    assert_eq!(program.num_results, 3);
}

#[test]
fn comparing_measurement_results_for_equality_adds_read_result_and_comparison_instructions() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use (q0, q1) = (Qubit(), Qubit());
                let r0 = QIR.Intrinsic.__quantum__qis__m__body(q0);
                let r1 = QIR.Intrinsic.__quantum__qis__m__body(q1);
                r0 == r1
            }
        }
        "#,
    });
    let measurement_callable_id = CallableId(1);
    assert_callable(
        &program,
        measurement_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__mz__body
            call_type: Measurement
            input_type:
                [0]: Qubit
                [1]: Result
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let readout_callable_id = CallableId(2);
    assert_callable(
        &program,
        readout_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__read_result__body
            call_type: Readout
            input_type:
                [0]: Result
            output_type: Boolean
            body: <NONE>"#]],
    );
    let bool_record_id = CallableId(3);
    assert_callable(
        &program,
        bool_record_id,
        &expect![[r#"
        Callable:
            name: __quantum__rt__bool_record_output
            call_type: OutputRecording
            input_type:
                [0]: Boolean
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Result(0), )
                Call id(1), args( Qubit(1), Result(1), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Call id(2), args( Result(1), )
                Variable(2, Boolean) = Icmp Eq, Variable(0, Boolean), Variable(1, Boolean)
                Call id(3), args( Variable(2, Boolean), Pointer, )
                Return"#]],
    );
    assert_eq!(program.num_qubits, 2);
    assert_eq!(program.num_results, 2);
}

#[test]
fn comparing_measurement_results_for_inequality_adds_read_result_and_comparison_instructions() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use (q0, q1) = (Qubit(), Qubit());
                let r0 = QIR.Intrinsic.__quantum__qis__m__body(q0);
                let r1 = QIR.Intrinsic.__quantum__qis__m__body(q1);
                r0 != r1
            }
        }
        "#,
    });
    let measurement_callable_id = CallableId(1);
    assert_callable(
        &program,
        measurement_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__mz__body
            call_type: Measurement
            input_type:
                [0]: Qubit
                [1]: Result
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let readout_callable_id = CallableId(2);
    assert_callable(
        &program,
        readout_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__read_result__body
            call_type: Readout
            input_type:
                [0]: Result
            output_type: Boolean
            body: <NONE>"#]],
    );
    let bool_record_id = CallableId(3);
    assert_callable(
        &program,
        bool_record_id,
        &expect![[r#"
        Callable:
            name: __quantum__rt__bool_record_output
            call_type: OutputRecording
            input_type:
                [0]: Boolean
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Result(0), )
                Call id(1), args( Qubit(1), Result(1), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Call id(2), args( Result(1), )
                Variable(2, Boolean) = Icmp Ne, Variable(0, Boolean), Variable(1, Boolean)
                Call id(3), args( Variable(2, Boolean), Pointer, )
                Return"#]],
    );
    assert_eq!(program.num_qubits, 2);
    assert_eq!(program.num_results, 2);
}

#[test]
fn comparing_measurement_result_against_result_literal_for_equality_adds_read_result_and_comparison_instructions(
) {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__m__body(q);
                r == One
            }
        }
        "#,
    });
    let measurement_callable_id = CallableId(1);
    assert_callable(
        &program,
        measurement_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__mz__body
            call_type: Measurement
            input_type:
                [0]: Qubit
                [1]: Result
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let readout_callable_id = CallableId(2);
    assert_callable(
        &program,
        readout_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__read_result__body
            call_type: Readout
            input_type:
                [0]: Result
            output_type: Boolean
            body: <NONE>"#]],
    );
    let bool_record_id = CallableId(3);
    assert_callable(
        &program,
        bool_record_id,
        &expect![[r#"
        Callable:
            name: __quantum__rt__bool_record_output
            call_type: OutputRecording
            input_type:
                [0]: Boolean
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(true)
                Call id(3), args( Variable(1, Boolean), Pointer, )
                Return"#]],
    );
    assert_eq!(program.num_qubits, 1);
    assert_eq!(program.num_results, 1);
}

#[test]
fn comparing_measurement_result_against_result_literal_for_inequality_adds_read_result_and_comparison_instructions(
) {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__m__body(q);
                r != Zero
            }
        }
        "#,
    });
    let measurement_callable_id = CallableId(1);
    assert_callable(
        &program,
        measurement_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__mz__body
            call_type: Measurement
            input_type:
                [0]: Qubit
                [1]: Result
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let readout_callable_id = CallableId(2);
    assert_callable(
        &program,
        readout_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__read_result__body
            call_type: Readout
            input_type:
                [0]: Result
            output_type: Boolean
            body: <NONE>"#]],
    );
    let bool_record_id = CallableId(3);
    assert_callable(
        &program,
        bool_record_id,
        &expect![[r#"
        Callable:
            name: __quantum__rt__bool_record_output
            call_type: OutputRecording
            input_type:
                [0]: Boolean
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Ne, Variable(0, Boolean), Bool(false)
                Call id(3), args( Variable(1, Boolean), Pointer, )
                Return"#]],
    );
    assert_eq!(program.num_qubits, 1);
    assert_eq!(program.num_results, 1);
}
