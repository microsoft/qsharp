// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use expect_test::expect;
use indoc::indoc;
use qsc_rir::rir::{BlockId, CallableId};
use test_utils::{
    assert_block_instructions, assert_blocks, assert_callable, assert_error,
    get_partial_evaluation_error, get_rir_program,
};

#[test]
fn leading_positive_unary_operator_does_not_generate_rir_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                +i
            }
        }
    "#});
    let mresetz_callable_id = CallableId(1);
    assert_callable(
        &program,
        mresetz_callable_id,
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
    let output_recording_callable_id = CallableId(3);
    assert_callable(
        &program,
        output_recording_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Call id(3), args( Variable(2, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn leading_negative_unary_operator_generates_rir_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                -i
            }
        }
    "#});
    let mresetz_callable_id = CallableId(1);
    assert_callable(
        &program,
        mresetz_callable_id,
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
    let output_recording_callable_id = CallableId(3);
    assert_callable(
        &program,
        output_recording_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Integer) = Mul Integer(-1), Variable(2, Integer)
                Call id(3), args( Variable(3, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn bitwise_not_unary_operator_generates_rir_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                ~~~i
            }
        }
    "#});
    let mresetz_callable_id = CallableId(1);
    assert_callable(
        &program,
        mresetz_callable_id,
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
    let output_recording_callable_id = CallableId(3);
    assert_callable(
        &program,
        output_recording_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Integer) = BitwiseNot Variable(2, Integer)
                Call id(3), args( Variable(3, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn logical_not_unary_operator_generates_logical_not_rir_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let b = MResetZ(q) == Zero;
                not b
            }
        }
    "#});
    let mresetz_callable_id = CallableId(1);
    assert_callable(
        &program,
        mresetz_callable_id,
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
    let output_recording_callable_id = CallableId(3);
    assert_callable(
        &program,
        output_recording_callable_id,
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
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Variable(2, Boolean) = LogicalNot Variable(1, Boolean)
                Call id(3), args( Variable(2, Boolean), Pointer, )
                Return"#]],
    );
}

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
                r == Zero
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
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
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
                r != One
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
                Variable(1, Boolean) = Icmp Ne, Variable(0, Boolean), Bool(true)
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
                false == b
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
                Variable(1, Boolean) = Store Variable(0, Boolean)
                Variable(2, Boolean) = Icmp Eq, Bool(false), Variable(1, Boolean)
                Call id(3), args( Variable(2, Boolean), Pointer, )
                Return"#]],
    );
}

#[test]
fn comparing_lhs_dynamic_boolean_against_rhs_dynamic_boolean_for_equality_raises_error() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                (MResetZ(q) == Zero) == (MResetZ(q) == One)
            }
        }
        "#,
    });
    // This error message will no longer happen once Boolean operations with a dynamic LHS are supported.
    assert_error(
        &error,
        &expect![[
            r#"Unimplemented("bool binary operation with dynamic LHS", Span { lo: 99, hi: 142 })"#
        ]],
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
                Variable(1, Boolean) = Store Variable(0, Boolean)
                Variable(2, Boolean) = Icmp Ne, Bool(true), Variable(1, Boolean)
                Call id(3), args( Variable(2, Boolean), Pointer, )
                Return"#]],
    );
}

#[test]
fn comparing_lhs_dynamic_boolean_against_rhs_dynamic_boolean_for_inequality_raises_error() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                (MResetZ(q) == Zero) != (MResetZ(q) == One)
            }
        }
        "#,
    });
    // This error message will no longer happen once Boolean operations with a dynamic LHS are supported.
    assert_error(
        &error,
        &expect![[
            r#"Unimplemented("bool binary operation with dynamic LHS", Span { lo: 99, hi: 142 })"#
        ]],
    );
}

#[test]
fn logical_and_with_lhs_classical_true_is_optimized_as_store() {
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
                Variable(1, Boolean) = Store Variable(0, Boolean)
                Call id(3), args( Variable(1, Boolean), Pointer, )
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
                Variable(1, Boolean) = Store Variable(0, Boolean)
                Call id(3), args( Bool(false), Pointer, )
                Return"#]],
    );
}

#[test]
fn logical_and_with_dynamic_lhs_and_dynamic_rhs_raises_error() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                (MResetZ(q) == Zero) and (MResetZ(q) == One)
            }
        }
        "#,
    });
    // This error message will no longer happen once Boolean operations with a dynamic LHS are supported.
    assert_error(
        &error,
        &expect![[
            r#"Unimplemented("bool binary operation with dynamic LHS", Span { lo: 99, hi: 143 })"#
        ]],
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
                Variable(1, Boolean) = Store Variable(0, Boolean)
                Call id(3), args( Bool(true), Pointer, )
                Return"#]],
    );
}

#[test]
fn logical_or_with_lhs_classical_false_is_optimized_as_store() {
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
                Variable(1, Boolean) = Store Variable(0, Boolean)
                Call id(3), args( Variable(1, Boolean), Pointer, )
                Return"#]],
    );
}

#[test]
fn logical_or_with_dynamic_lhs_and_dynamic_rhs_raises_error() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                (MResetZ(q) == Zero) or (MResetZ(q) == One)
            }
        }
        "#,
    });
    // This error message will no longer happen once Boolean operations with a dynamic LHS are supported.
    assert_error(
        &error,
        &expect![[
            r#"Unimplemented("bool binary operation with dynamic LHS", Span { lo: 99, hi: 142 })"#
        ]],
    );
}

#[test]
fn integer_add_with_lhs_classical_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                1 + i
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
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Integer) = Add Integer(1), Variable(2, Integer)
                Call id(3), args( Variable(3, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn integer_sub_with_lhs_dynamic_integer_and_rhs_classical_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                i - 1
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
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Integer) = Sub Variable(2, Integer), Integer(1)
                Call id(3), args( Variable(3, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn integer_mul_with_lhs_dynamic_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let a = MResetZ(q) == Zero ? 0 | 1;
                let b = MResetZ(q) == Zero ? 1 | 0;
                a * b
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
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Call id(1), args( Qubit(0), Result(1), )
                Variable(3, Boolean) = Call id(2), args( Result(1), )
                Variable(4, Boolean) = Icmp Eq, Variable(3, Boolean), Bool(false)
                Branch Variable(4, Boolean), 5, 6
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)
            Block 4:Block:
                Variable(6, Integer) = Mul Variable(2, Integer), Variable(5, Integer)
                Call id(3), args( Variable(6, Integer), Pointer, )
                Return
            Block 5:Block:
                Variable(5, Integer) = Store Integer(1)
                Jump(4)
            Block 6:Block:
                Variable(5, Integer) = Store Integer(0)
                Jump(4)"#]],
    );
}

#[test]
fn integer_div_with_lhs_classical_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                1 / i
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
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Integer) = Sdiv Integer(1), Variable(2, Integer)
                Call id(3), args( Variable(3, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn integer_div_with_lhs_dynamic_integer_and_rhs_zero_raises_error() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                i / 0
            }
        }
        "#,
    });
    assert_error(
        &error,
        &expect![[r#"EvaluationFailed("division by zero", Span { lo: 142, hi: 147 })"#]],
    );
}

#[test]
fn integer_mod_with_lhs_dynamic_integer_and_rhs_classical_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                i % 1
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
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Integer) = Srem Variable(2, Integer), Integer(1)
                Call id(3), args( Variable(3, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn integer_exponentiation_with_lhs_classical_integer_and_rhs_dynamic_integer() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                2 ^ i
            }
        }
        "#,
    });
    // When this binary operation is supported, this error should be different.
    assert_error(
        &error,
        &expect![[
            r#"Unimplemented("exponentiation for integer operands", Span { lo: 142, hi: 147 })"#
        ]],
    );
}

#[test]
fn integer_exponentiation_with_lhs_dynamic_integer_and_rhs_classical_integer() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                i ^ 3
            }
        }
        "#,
    });
    // When this binary operation is supported, this program should not yield an error.
    assert_error(
        &error,
        &expect![[
            r#"Unimplemented("exponentiation for integer operands", Span { lo: 142, hi: 147 })"#
        ]],
    );
}

#[test]
fn integer_exponentiation_with_lhs_dynamic_integer_and_rhs_dynamic_integer() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let a = MResetZ(q) == Zero ? 0 | 1;
                let b = MResetZ(q) == Zero ? 1 | 0;
                a ^ b
            }
        }
        "#,
    });
    // When this binary operation is supported, this error should be different.
    assert_error(
        &error,
        &expect![[
            r#"Unimplemented("exponentiation for integer operands", Span { lo: 186, hi: 191 })"#
        ]],
    );
}

#[test]
fn integer_bitwise_and_with_lhs_dynamic_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let a = MResetZ(q) == Zero ? 0 | 1;
                let b = MResetZ(q) == Zero ? 1 | 0;
                a &&& b
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
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Call id(1), args( Qubit(0), Result(1), )
                Variable(3, Boolean) = Call id(2), args( Result(1), )
                Variable(4, Boolean) = Icmp Eq, Variable(3, Boolean), Bool(false)
                Branch Variable(4, Boolean), 5, 6
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)
            Block 4:Block:
                Variable(6, Integer) = BitwiseAnd Variable(2, Integer), Variable(5, Integer)
                Call id(3), args( Variable(6, Integer), Pointer, )
                Return
            Block 5:Block:
                Variable(5, Integer) = Store Integer(1)
                Jump(4)
            Block 6:Block:
                Variable(5, Integer) = Store Integer(0)
                Jump(4)"#]],
    );
}

#[test]
fn integer_bitwise_or_with_lhs_classical_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                1 ||| i
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
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Integer) = BitwiseOr Integer(1), Variable(2, Integer)
                Call id(3), args( Variable(3, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn integer_bitwise_xor_with_lhs_dynamic_integer_and_rhs_classical_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                i ^^^ 1
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
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Integer) = BitwiseXor Variable(2, Integer), Integer(1)
                Call id(3), args( Variable(3, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn integer_bitwise_left_shif_with_lhs_dynamic_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let a = MResetZ(q) == Zero ? 0 | 1;
                let b = MResetZ(q) == Zero ? 1 | 0;
                a <<< b
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
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Call id(1), args( Qubit(0), Result(1), )
                Variable(3, Boolean) = Call id(2), args( Result(1), )
                Variable(4, Boolean) = Icmp Eq, Variable(3, Boolean), Bool(false)
                Branch Variable(4, Boolean), 5, 6
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)
            Block 4:Block:
                Variable(6, Integer) = Shl Variable(2, Integer), Variable(5, Integer)
                Call id(3), args( Variable(6, Integer), Pointer, )
                Return
            Block 5:Block:
                Variable(5, Integer) = Store Integer(1)
                Jump(4)
            Block 6:Block:
                Variable(5, Integer) = Store Integer(0)
                Jump(4)"#]],
    );
}

#[test]
fn integer_bitwise_right_shift_with_lhs_classical_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                1 >>> i
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
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Integer) = Ashr Integer(1), Variable(2, Integer)
                Call id(3), args( Variable(3, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn integer_equality_comparison_with_lhs_dynamic_integer_and_rhs_classical_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                i == 1
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
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Boolean) = Icmp Eq, Variable(2, Integer), Integer(1)
                Call id(3), args( Variable(3, Boolean), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn integer_inequality_comparison_with_lhs_dynamic_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let a = MResetZ(q) == Zero ? 0 | 1;
                let b = MResetZ(q) == Zero ? 1 | 0;
                a != b
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
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Call id(1), args( Qubit(0), Result(1), )
                Variable(3, Boolean) = Call id(2), args( Result(1), )
                Variable(4, Boolean) = Icmp Eq, Variable(3, Boolean), Bool(false)
                Branch Variable(4, Boolean), 5, 6
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)
            Block 4:Block:
                Variable(6, Boolean) = Icmp Ne, Variable(2, Integer), Variable(5, Integer)
                Call id(3), args( Variable(6, Boolean), Pointer, )
                Return
            Block 5:Block:
                Variable(5, Integer) = Store Integer(1)
                Jump(4)
            Block 6:Block:
                Variable(5, Integer) = Store Integer(0)
                Jump(4)"#]],
    );
}

#[test]
fn integer_greater_than_comparison_with_lhs_classical_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                1 > i
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
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Boolean) = Icmp Sgt, Integer(1), Variable(2, Integer)
                Call id(3), args( Variable(3, Boolean), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn integer_greater_or_equal_than_comparison_with_lhs_dynamic_integer_and_rhs_classical_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                i >= 1
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
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Boolean) = Icmp Sge, Variable(2, Integer), Integer(1)
                Call id(3), args( Variable(3, Boolean), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn integer_less_than_comparison_with_lhs_dynamic_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let a = MResetZ(q) == Zero ? 0 | 1;
                let b = MResetZ(q) == Zero ? 1 | 0;
                a < b
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
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Call id(1), args( Qubit(0), Result(1), )
                Variable(3, Boolean) = Call id(2), args( Result(1), )
                Variable(4, Boolean) = Icmp Eq, Variable(3, Boolean), Bool(false)
                Branch Variable(4, Boolean), 5, 6
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)
            Block 4:Block:
                Variable(6, Boolean) = Icmp Slt, Variable(2, Integer), Variable(5, Integer)
                Call id(3), args( Variable(6, Boolean), Pointer, )
                Return
            Block 5:Block:
                Variable(5, Integer) = Store Integer(1)
                Jump(4)
            Block 6:Block:
                Variable(5, Integer) = Store Integer(0)
                Jump(4)"#]],
    );
}

#[test]
fn integer_less_or_equal_than_comparison_with_lhs_classical_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let i = MResetZ(q) == Zero ? 0 | 1;
                1 <= i
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
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Boolean) = Icmp Sle, Integer(1), Variable(2, Integer)
                Call id(3), args( Variable(3, Boolean), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}
