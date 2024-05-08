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
use qsc::TargetCapabilityFlags;
use qsc_rir::rir::{BlockId, CallableId};
use test_utils::{
    assert_block_instructions, assert_blocks, assert_callable, assert_error,
    get_partial_evaluation_error_with_capabilities, get_rir_program,
    get_rir_program_with_capabilities,
};

#[test]
fn immutable_result_binding_does_not_generate_store_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                let r = MResetZ(q);
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
fn mutable_result_binding_does_not_generate_store_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                mutable r = MResetZ(q);
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
fn immutable_bool_binding_does_not_generate_store_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                let b = MResetZ(q) == One;
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
    let read_result_callable_id = CallableId(2);
    assert_callable(
        &program,
        read_result_callable_id,
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
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(true)
                Call id(3), args( Variable(1, Boolean), Pointer, )
                Return"#]],
    );
}

#[test]
fn mutable_bool_binding_generates_store_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Bool {
                use q = Qubit();
                mutable b = MResetZ(q) == One;
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
    let read_result_callable_id = CallableId(2);
    assert_callable(
        &program,
        read_result_callable_id,
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
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(true)
                Variable(2, Boolean) = Store Variable(1, Boolean)
                Call id(3), args( Variable(2, Boolean), Pointer, )
                Return"#]],
    );
}

#[test]
fn immutable_int_binding_does_not_generate_store_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                let i = MResetZ(q) == One ? 0 | 1;
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
    let read_result_callable_id = CallableId(2);
    assert_callable(
        &program,
        read_result_callable_id,
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
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(true)
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
fn mutable_int_binding_does_generate_store_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = MResetZ(q) == One ? 0 | 1;
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
    let read_result_callable_id = CallableId(2);
    assert_callable(
        &program,
        read_result_callable_id,
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
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(true)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Variable(3, Integer) = Store Variable(2, Integer)
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
fn classical_mutable_int_binding_does_not_generate_store_instruction_without_capability() {
    let program = get_rir_program_with_capabilities(
        indoc! {r#"
            namespace Test {
                @EntryPoint()
                operation Main() : Int {
                    mutable i = 0;
                    set i = i + 1;
                    i
                }
            }
        "#},
        TargetCapabilityFlags::Adaptive,
    );

    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Call id(1), args( Integer(1), Pointer, )
                Return"#]],
    );
}

#[test]
fn classical_mutable_int_binding_does_generate_store_instruction_with_capability() {
    let program = get_rir_program_with_capabilities(
        indoc! {r#"
            namespace Test {
                @EntryPoint()
                operation Main() : Int {
                    mutable i = 0;
                    set i = i + 1;
                    i
                }
            }
        "#},
        TargetCapabilityFlags::Adaptive | TargetCapabilityFlags::IntegerComputations,
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
fn dynamic_mutable_int_binding_fails_without_capability() {
    let error = get_partial_evaluation_error_with_capabilities(
        indoc! {r#"
        namespace Test {
            function UpdateFunc() : Result -> Int {
                r -> if r == One { 0 } else { 1 }
            }
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = 0;
                let f = UpdateFunc();
                set i = f(MResetZ(q));
                i
            }
        }
    "#},
        TargetCapabilityFlags::Adaptive,
    );
    assert_error(
        &error,
        &expect![[r#"MissingCapabilities("Integer Computations", Span { lo: 75, hi: 103 })"#]],
    );
}

#[test]
fn dynamic_mutable_int_binding_succeeds_with_capability() {
    let program = get_rir_program_with_capabilities(
        indoc! {r#"
        namespace Test {
            function UpdateFunc() : Result -> Int {
                r -> if r == One { 0 } else { 1 }
            }
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = 0;
                let f = UpdateFunc();
                set i = f(MResetZ(q));
                i
            }
        }
    "#},
        TargetCapabilityFlags::Adaptive | TargetCapabilityFlags::IntegerComputations,
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
                Variable(3, Integer) = Store Integer(0)
                Jump(1)
            Block 3:Block:
                Variable(3, Integer) = Store Integer(1)
                Jump(1)"#]],
    );
}
