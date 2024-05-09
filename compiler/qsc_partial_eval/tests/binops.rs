// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use expect_test::expect;
use indoc::indoc;
use qsc_rir::rir::{BlockId, CallableId};
use test_utils::{assert_block_instructions, assert_callable, get_rir_program};

#[test]
fn comparing_measurement_results_for_equality_adds_read_result_and_comparison_instructions() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use (q0, q1) = (Qubit(), Qubit());
                let r0 = MResetZ(q0);
                let r1 = MResetZ(q1);
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
                name: __quantum__qis__mresetz__body
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
                let r0 = MResetZ(q0);
                let r1 = MResetZ(q1);
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
            name: __quantum__qis__mresetz__body
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
                let r = MResetZ(q);
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
            name: __quantum__qis__mresetz__body
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
                let r = MResetZ(q);
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
            name: __quantum__qis__mresetz__body
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

#[test]
fn comparing_lhs_classical_boolean_against_rhs_dynamic_boolean_for_equality() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let b = MResetZ(q) == One;
                true == b
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
                name: __quantum__qis__mresetz__body
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
    let output_record_id = CallableId(3);
    assert_callable(
        &program,
        output_record_id,
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
                Variable(2, Boolean) = Icmp Eq, Bool(true), Variable(1, Boolean)
                Call id(3), args( Variable(2, Boolean), Pointer, )
                Return"#]],
    );
}

#[test]
fn comparing_lhs_classical_boolean_against_rhs_dynamic_boolean_for_inequality() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let b = MResetZ(q) == One;
                true != b
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
                name: __quantum__qis__mresetz__body
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
    let output_record_id = CallableId(3);
    assert_callable(
        &program,
        output_record_id,
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
                Variable(2, Boolean) = Icmp Ne, Bool(true), Variable(1, Boolean)
                Call id(3), args( Variable(2, Boolean), Pointer, )
                Return"#]],
    );
}

#[test]
fn logical_and_with_lhs_classical_true_generates_boolean_instruction() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let b = MResetZ(q) == One;
                true and b
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
                name: __quantum__qis__mresetz__body
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
    let output_record_id = CallableId(3);
    assert_callable(
        &program,
        output_record_id,
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
                Variable(2, Boolean) = LogicalAnd Bool(true), Variable(1, Boolean)
                Call id(3), args( Variable(2, Boolean), Pointer, )
                Return"#]],
    );
}

#[test]
fn logical_and_with_lhs_classical_false_short_circuits_evaluation() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let b = MResetZ(q) == One;
                false and (true == b)
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
                name: __quantum__qis__mresetz__body
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
    let output_record_id = CallableId(3);
    assert_callable(
        &program,
        output_record_id,
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
                Call id(3), args( Bool(false), Pointer, )
                Return"#]],
    );
}

#[test]
fn logical_or_with_lhs_classical_true_short_circuits_evaluation() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let b = MResetZ(q) == One;
                true or (false != b)
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
                name: __quantum__qis__mresetz__body
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
    let output_record_id = CallableId(3);
    assert_callable(
        &program,
        output_record_id,
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
                Call id(3), args( Bool(true), Pointer, )
                Return"#]],
    );
}

#[test]
fn logical_or_with_lhs_classical_false_generates_boolean_instruction() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let b = MResetZ(q) == One;
                false or b
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
                name: __quantum__qis__mresetz__body
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
    let output_record_id = CallableId(3);
    assert_callable(
        &program,
        output_record_id,
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
                Variable(2, Boolean) = LogicalOr Bool(false), Variable(1, Boolean)
                Call id(3), args( Variable(2, Boolean), Pointer, )
                Return"#]],
    );
}
