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
                Variable(2, Boolean) = Store Variable(1, Boolean)
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
                name: __quantum__rt__int_record_output
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
                Variable(0, Integer) = Store Integer(0)
                Call id(1), args( Qubit(0), Result(0), )
                Variable(1, Boolean) = Call id(2), args( Result(0), )
                Variable(2, Boolean) = Store Variable(1, Boolean)
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
                Variable(2, Boolean) = Store Variable(1, Boolean)
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

#[test]
fn array_of_results_replace_element_at_index_with_dynamic_content() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result[] {
                use (q0, q1) = (Qubit(), Qubit());
                mutable arr = [MResetZ(q0), Zero];
                set arr w/= 1 <- MResetZ(q1);
                arr
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
    let array_output_recording_callable_id = CallableId(2);
    assert_callable(
        &program,
        array_output_recording_callable_id,
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
    let result_output_recording_callable_id = CallableId(3);
    assert_callable(
        &program,
        result_output_recording_callable_id,
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
                Call id(2), args( Integer(2), Pointer, )
                Call id(3), args( Result(0), Pointer, )
                Call id(3), args( Result(1), Pointer, )
                Return"#]],
    );
}

#[test]
fn array_of_bools_replace_element_at_index_with_dynamic_content() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool[] {
                use (q0, q1) = (Qubit(), Qubit());
                mutable arr = [MResetZ(q0) == Zero, true];
                set arr w/= 1 <- MResetZ(q1) == One;
                arr
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
    let array_output_recording_callable_id = CallableId(3);
    assert_callable(
        &program,
        array_output_recording_callable_id,
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
    let bool_output_recording_callable_id = CallableId(4);
    assert_callable(
        &program,
        bool_output_recording_callable_id,
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
                Call id(1), args( Qubit(1), Result(1), )
                Variable(2, Boolean) = Call id(2), args( Result(1), )
                Variable(3, Boolean) = Store Variable(2, Boolean)
                Call id(3), args( Integer(2), Pointer, )
                Call id(4), args( Variable(1, Boolean), Pointer, )
                Call id(4), args( Variable(3, Boolean), Pointer, )
                Return"#]],
    );
}

#[test]
fn empty_array_of_results_in_place_concatenation() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result[] {
                use (q0, q1) = (Qubit(), Qubit());
                mutable results = [];
                set results += [MResetZ(q0)];
                set results += [MResetZ(q1)];
                results
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
    let array_output_recording_callable_id = CallableId(2);
    assert_callable(
        &program,
        array_output_recording_callable_id,
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
    let result_output_recording_callable_id = CallableId(3);
    assert_callable(
        &program,
        result_output_recording_callable_id,
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
                Call id(2), args( Integer(2), Pointer, )
                Call id(3), args( Result(0), Pointer, )
                Call id(3), args( Result(1), Pointer, )
                Return"#]],
    );
}

#[test]
fn non_empty_array_of_results_in_place_concatenation() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result[] {
                use (q0, q1) = (Qubit(), Qubit());
                mutable results = [MResetZ(q0)];
                set results += [MResetZ(q1)];
                results
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
    let array_output_recording_callable_id = CallableId(2);
    assert_callable(
        &program,
        array_output_recording_callable_id,
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
    let result_output_recording_callable_id = CallableId(3);
    assert_callable(
        &program,
        result_output_recording_callable_id,
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
                Call id(2), args( Integer(2), Pointer, )
                Call id(3), args( Result(0), Pointer, )
                Call id(3), args( Result(1), Pointer, )
                Return"#]],
    );
}

#[test]
fn logical_and_assign_with_lhs_classical_true_is_optimized_as_store() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let a = MResetZ(q) == One;
                mutable b = true;
                set b and= a;
                b
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
                Variable(2, Boolean) = Store Bool(true)
                Variable(2, Boolean) = Store Variable(1, Boolean)
                Call id(3), args( Variable(2, Boolean), Pointer, )
                Return"#]],
    );
}

#[test]
fn logical_and_assign_with_lhs_classical_false_short_circuits_evaluation() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let a = MResetZ(q) == One;
                mutable b = false;
                set b and= a;
                b
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
                Variable(2, Boolean) = Store Bool(false)
                Variable(2, Boolean) = Store Bool(false)
                Call id(3), args( Bool(false), Pointer, )
                Return"#]],
    );
}

#[test]
fn logical_and_assign_with_dynamic_lhs_and_dynamic_rhs_raises_error() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                mutable b = MResetZ(q) == Zero;
                set b and= MResetZ(q) == One;
                b
            }
        }
        "#,
    });
    // This error message will no longer happen once Boolean operations with a dynamic LHS are supported.
    assert_error(
        &error,
        &expect![[
            r#"Unimplemented("bool binary operation with dynamic LHS", Span { lo: 139, hi: 167 })"#
        ]],
    );
}

#[test]
fn logical_or_assign_with_lhs_classical_true_short_circuits_evaluation() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let a = MResetZ(q) == One;
                mutable b = true;
                set b or= a;
                b
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
                Variable(2, Boolean) = Store Bool(true)
                Variable(2, Boolean) = Store Bool(true)
                Call id(3), args( Bool(true), Pointer, )
                Return"#]],
    );
}

#[test]
fn logical_or_assign_with_lhs_classical_false_is_optimized_as_store() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let a = MResetZ(q) == One;
                mutable b = false;
                set b or= a;
                b
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
                Variable(2, Boolean) = Store Bool(false)
                Variable(2, Boolean) = Store Variable(1, Boolean)
                Call id(3), args( Variable(2, Boolean), Pointer, )
                Return"#]],
    );
}

#[test]
fn logical_or_assign_with_dynamic_lhs_and_dynamic_rhs_raises_error() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                mutable b = MResetZ(q) == Zero;
                set b or= MResetZ(q) == One;
                b
            }
        }
        "#,
    });
    // This error message will no longer happen once Boolean operations with a dynamic LHS are supported.
    assert_error(
        &error,
        &expect![[
            r#"Unimplemented("bool binary operation with dynamic LHS", Span { lo: 139, hi: 166 })"#
        ]],
    );
}

#[test]
fn integer_assign_add_with_lhs_classical_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = 0;
                set i += MResetZ(q) == Zero ? 0 | 1;
                i
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
                Variable(0, Integer) = Store Integer(0)
                Call id(1), args( Qubit(0), Result(0), )
                Variable(1, Boolean) = Call id(2), args( Result(0), )
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(false)
                Branch Variable(2, Boolean), 2, 3
            Block 1:Block:
                Variable(4, Integer) = Add Integer(0), Variable(3, Integer)
                Variable(0, Integer) = Store Variable(4, Integer)
                Call id(3), args( Variable(0, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(3, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(3, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn integer_assign_sub_with_lhs_dynamic_integer_and_rhs_classical_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = MResetZ(q) == Zero ? 0 | 1;
                set i -= 1;
                i
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
            Variable(3, Integer) = Store Variable(2, Integer)
            Variable(4, Integer) = Sub Variable(3, Integer), Integer(1)
            Variable(3, Integer) = Store Variable(4, Integer)
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
fn integer_assign_mul_with_lhs_dynamic_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = MResetZ(q) == Zero ? 0 | 1;
                set i *= MResetZ(q) == Zero ? 1 | 0;
                i
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
            Variable(3, Integer) = Store Variable(2, Integer)
            Call id(1), args( Qubit(0), Result(1), )
            Variable(4, Boolean) = Call id(2), args( Result(1), )
            Variable(5, Boolean) = Icmp Eq, Variable(4, Boolean), Bool(false)
            Branch Variable(5, Boolean), 5, 6
        Block 2:Block:
            Variable(2, Integer) = Store Integer(0)
            Jump(1)
        Block 3:Block:
            Variable(2, Integer) = Store Integer(1)
            Jump(1)
        Block 4:Block:
            Variable(7, Integer) = Mul Variable(3, Integer), Variable(6, Integer)
            Variable(3, Integer) = Store Variable(7, Integer)
            Call id(3), args( Variable(3, Integer), Pointer, )
            Return
        Block 5:Block:
            Variable(6, Integer) = Store Integer(1)
            Jump(4)
        Block 6:Block:
            Variable(6, Integer) = Store Integer(0)
            Jump(4)"#]],
    );
}

#[test]
fn integer_assign_div_with_lhs_classical_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = 0;
                set i /= MResetZ(q) == Zero ? 0 | 1;
                i
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
                Variable(0, Integer) = Store Integer(0)
                Call id(1), args( Qubit(0), Result(0), )
                Variable(1, Boolean) = Call id(2), args( Result(0), )
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(false)
                Branch Variable(2, Boolean), 2, 3
            Block 1:Block:
                Variable(4, Integer) = Sdiv Integer(0), Variable(3, Integer)
                Variable(0, Integer) = Store Variable(4, Integer)
                Call id(3), args( Variable(0, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(3, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(3, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}

#[test]
fn integer_assign_mod_with_lhs_dynamic_integer_and_rhs_classical_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = MResetZ(q) == Zero ? 0 | 1;
                set i %= 1;
                i
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
            Variable(3, Integer) = Store Variable(2, Integer)
            Variable(4, Integer) = Srem Variable(3, Integer), Integer(1)
            Variable(3, Integer) = Store Variable(4, Integer)
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
fn integer_assign_exp_with_lhs_classical_integer_and_rhs_dynamic_integer_raises_error() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = 0;
                set i ^= MResetZ(q) == Zero ? 0 | 1;
                i
            }
        }
        "#,
    });
    assert_error(
        &error,
        &expect![[
            r#"Unexpected("exponent must be a classical integer", Span { lo: 121, hi: 156 })"#
        ]],
    );
}

#[test]
fn integer_assign_exp_with_lhs_classical_integer_and_rhs_classical_negative_integer_raises_error() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = 0;
                set i ^= -1;
                i
            }
        }
        "#,
    });
    assert_error(
        &error,
        &expect![[
            r#"EvaluationFailed("negative integers cannot be used here: -1", Span { lo: 130, hi: 132 })"#
        ]],
    );
}

#[test]
fn integer_assign_exp_with_lhs_dynamic_integer_and_rhs_classical_zero_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = MResetZ(q) == Zero ? 0 | 1;
                set i ^= 0;
                i
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
                Variable(3, Integer) = Store Variable(2, Integer)
                Variable(4, Integer) = Store Integer(1)
                Variable(3, Integer) = Store Variable(4, Integer)
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
fn integer_assign_exp_with_lhs_dynamic_integer_and_rhs_classical_positive_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = MResetZ(q) == Zero ? 0 | 1;
                set i ^= 3;
                i
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
                Variable(3, Integer) = Store Variable(2, Integer)
                Variable(4, Integer) = Store Integer(1)
                Variable(5, Integer) = Mul Variable(4, Integer), Variable(3, Integer)
                Variable(6, Integer) = Mul Variable(5, Integer), Variable(3, Integer)
                Variable(7, Integer) = Mul Variable(6, Integer), Variable(3, Integer)
                Variable(3, Integer) = Store Variable(7, Integer)
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
fn integer_assign_exp_with_lhs_dynamic_integer_and_rhs_dynamic_integer_raises_error() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = MResetZ(q) == Zero ? 0 | 1;
                set i ^= MResetZ(q) == Zero ? 1 | 0;
                i
            }
        }
        "#,
    });
    assert_error(
        &error,
        &expect![[
            r#"Unexpected("exponent must be a classical integer", Span { lo: 146, hi: 181 })"#
        ]],
    );
}

#[test]
fn integer_assign_bitwise_and_with_lhs_dynamic_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = MResetZ(q) == Zero ? 0 | 1;
                set i &&&= MResetZ(q) == Zero ? 1 | 0;
                i
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
                Variable(3, Integer) = Store Variable(2, Integer)
                Call id(1), args( Qubit(0), Result(1), )
                Variable(4, Boolean) = Call id(2), args( Result(1), )
                Variable(5, Boolean) = Icmp Eq, Variable(4, Boolean), Bool(false)
                Branch Variable(5, Boolean), 5, 6
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)
            Block 4:Block:
                Variable(7, Integer) = BitwiseAnd Variable(3, Integer), Variable(6, Integer)
                Variable(3, Integer) = Store Variable(7, Integer)
                Call id(3), args( Variable(3, Integer), Pointer, )
                Return
            Block 5:Block:
                Variable(6, Integer) = Store Integer(1)
                Jump(4)
            Block 6:Block:
                Variable(6, Integer) = Store Integer(0)
                Jump(4)"#]],
    );
}

#[test]
fn integer_assign_bitwise_or_with_lhs_classical_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = 0;
                set i |||= MResetZ(q) == Zero ? 0 | 1;
                i
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
                Variable(0, Integer) = Store Integer(0)
                Call id(1), args( Qubit(0), Result(0), )
                Variable(1, Boolean) = Call id(2), args( Result(0), )
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(false)
                Branch Variable(2, Boolean), 2, 3
            Block 1:Block:
                Variable(4, Integer) = BitwiseOr Integer(0), Variable(3, Integer)
                Variable(0, Integer) = Store Variable(4, Integer)
                Call id(3), args( Variable(0, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(3, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(3, Integer) = Store Integer(1)
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
                mutable i = MResetZ(q) == Zero ? 0 | 1;
                set i ^^^= 1;
                i
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
            Variable(3, Integer) = Store Variable(2, Integer)
            Variable(4, Integer) = BitwiseXor Variable(3, Integer), Integer(1)
            Variable(3, Integer) = Store Variable(4, Integer)
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
fn integer_assign_bitwise_left_shift_with_lhs_dynamic_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = MResetZ(q) == Zero ? 0 | 1;
                set i <<<= MResetZ(q) == Zero ? 1 | 0;
                i
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
                Variable(3, Integer) = Store Variable(2, Integer)
                Call id(1), args( Qubit(0), Result(1), )
                Variable(4, Boolean) = Call id(2), args( Result(1), )
                Variable(5, Boolean) = Icmp Eq, Variable(4, Boolean), Bool(false)
                Branch Variable(5, Boolean), 5, 6
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1)
                Jump(1)
            Block 4:Block:
                Variable(7, Integer) = Shl Variable(3, Integer), Variable(6, Integer)
                Variable(3, Integer) = Store Variable(7, Integer)
                Call id(3), args( Variable(3, Integer), Pointer, )
                Return
            Block 5:Block:
                Variable(6, Integer) = Store Integer(1)
                Jump(4)
            Block 6:Block:
                Variable(6, Integer) = Store Integer(0)
                Jump(4)"#]],
    );
}

#[test]
fn integer_assign_bitwise_right_shift_with_lhs_classical_integer_and_rhs_dynamic_integer() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = 0;
                set i >>>= MResetZ(q) == Zero ? 0 | 1;
                i
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
                Variable(0, Integer) = Store Integer(0)
                Call id(1), args( Qubit(0), Result(0), )
                Variable(1, Boolean) = Call id(2), args( Result(0), )
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(false)
                Branch Variable(2, Boolean), 2, 3
            Block 1:Block:
                Variable(4, Integer) = Ashr Integer(0), Variable(3, Integer)
                Variable(0, Integer) = Store Variable(4, Integer)
                Call id(3), args( Variable(0, Integer), Pointer, )
                Return
            Block 2:Block:
                Variable(3, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(3, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}
