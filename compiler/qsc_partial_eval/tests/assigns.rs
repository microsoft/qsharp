// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(
    clippy::needless_raw_string_hashes,
    clippy::similar_names,
    clippy::too_many_lines
)]

pub mod test_utils;

use expect_test::expect;
use indoc::indoc;
use qsc_rir::rir::{BlockId, CallableId};
use test_utils::{
    assert_block_instructions, assert_blocks, assert_callable, assert_error,
    get_partial_evaluation_error, get_rir_program,
};

#[test]
fn assigning_result_literal_updates_value() {
    // This test verifies that the result value is updated using the fact that a program that returns a result literal
    // value will raise an error in partial evaluation.
    let error = get_partial_evaluation_error(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                mutable r = MResetZ(q);
                set r = One;
                r
            }
        }
    "#});
    assert_error(
        &error,
        &expect!["OutputResultLiteral(Span { lo: 50, hi: 54 })"],
    );
}

#[test]
fn assigning_result_register_updates_value() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                mutable r = Zero;
                set r = MResetZ(q);
                r
            }
        }
    "#});
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
    let output_recording_callable_id = CallableId(2);
    assert_callable(
        &program,
        output_recording_callable_id,
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
}

#[test]
fn assigning_classical_bool_updates_value_and_adds_store_instructions() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit(); // Needed to make `Main` hybrid.
                mutable b = true;
                set b = false;
                b
            }
        }
    "#});
    let output_recording_callable_id = CallableId(1);
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
                Variable(0, Boolean) = Store Bool(true)
                Variable(0, Boolean) = Store Bool(false)
                Call id(1), args( Bool(false), Pointer, )
                Return"#]],
    );
}

#[test]
fn assigning_dynamic_bool_updates_value_and_adds_store_instructions() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                mutable b = false;
                set b = MResetZ(q) == One;
                b
            }
        }
    "#});
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
                Variable(0, Boolean) = Store Bool(false)
                Call id(1), args( Qubit(0), Result(0), )
                Variable(1, Boolean) = Call id(2), args( Result(0), )
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(true)
                Variable(0, Boolean) = Store Variable(2, Boolean)
                Call id(3), args( Variable(0, Boolean), Pointer, )
                Return"#]],
    );
}

#[test]
fn assigning_classical_int_updates_value_and_adds_store_instructions() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit(); // Needed to make `Main` hybrid.
                mutable i = 0;
                set i = 1;
                i
            }
        }
    "#});
    let output_recording_callable_id = CallableId(1);
    assert_callable(
        &program,
        output_recording_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__integer_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Variable(0, Integer) = Store Integer(0)
                Variable(0, Integer) = Store Integer(1)
                Call id(1), args( Integer(1), Pointer, )
                Return"#]],
    );
}

#[test]
fn assigning_dynamic_int_updates_value_and_adds_store_instructions() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = 0;
                set i = MResetZ(q) == One ? 1 | 2;
                i
            }
        }
    "#});
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
    let output_recording_callable_id = CallableId(3);
    assert_callable(
        &program,
        output_recording_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__integer_record_output
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
                Variable(0, Integer) = Store Integer(0)
                Call id(1), args( Qubit(0), Result(0), )
                Variable(1, Boolean) = Call id(2), args( Result(0), )
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(true)
                Branch Variable(2, Boolean), 2, 3
            Block 1:Block:
                Variable(0, Integer) = Store Variable(3, Integer)
                Call id(3), args( Variable(0, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(3, Integer) = Store Integer(1)
                Jump(1)
            Block 3:Block:
                Variable(3, Integer) = Store Integer(2)
                Jump(1)"#]],
    );
}

#[test]
fn assigning_classical_bool_within_dynamic_if_expression_adds_store_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                mutable b = false;
                if MResetZ(q) == One {
                    set b = true;
                }
                b
            }
        }
    "#});
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
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Variable(0, Boolean) = Store Bool(false)
                Call id(1), args( Qubit(0), Result(0), )
                Variable(1, Boolean) = Call id(2), args( Result(0), )
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(true)
                Branch Variable(2, Boolean), 2, 1
            Block 1:Block:
                Call id(3), args( Variable(0, Boolean), Pointer, )
                Return
            Block 2:Block:
                Variable(0, Boolean) = Store Bool(true)
                Jump(1)"#]],
    );
}

#[test]
fn assigning_classical_int_within_dynamic_if_else_expression_adds_store_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = 0;
                if MResetZ(q) == Zero {
                    set i = 1;
                } else {
                    set i = 2;
                }
                i
            }
        }
    "#});
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
    let output_recording_callable_id = CallableId(3);
    assert_callable(
        &program,
        output_recording_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__integer_record_output
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
                Variable(0, Integer) = Store Integer(0)
                Call id(1), args( Qubit(0), Result(0), )
                Variable(1, Boolean) = Call id(2), args( Result(0), )
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(false)
                Branch Variable(2, Boolean), 2, 3
            Block 1:Block:
                Call id(3), args( Variable(0, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(0, Integer) = Store Integer(1)
                Jump(1)
            Block 3:Block:
                Variable(0, Integer) = Store Integer(2)
                Jump(1)"#]],
    );
}

#[test]
fn assigning_result_literal_within_dynamic_if_expression_produces_error() {
    let error = get_partial_evaluation_error(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                mutable r = Zero;
                if MResetZ(q) == One {
                    set r = One;
                }
                r
            }
        }
    "#});
    assert_error(
        &error,
        &expect![[
            r#"Unexpected("re-assignment within a dynamic branch is unsupported for type Result", Span { lo: 166, hi: 167 })"#
        ]],
    );
}
