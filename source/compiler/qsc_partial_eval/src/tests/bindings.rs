// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(
    clippy::needless_raw_string_hashes,
    clippy::similar_names,
    clippy::too_many_lines
)]

use super::{assert_block_instructions, assert_blocks, assert_callable, get_rir_program};
use expect_test::expect;
use indoc::indoc;
use qsc_rir::rir::{BlockId, CallableId};

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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[109-119]
                Call id(2), args( Result(0), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[113-123]
                Call id(2), args( Result(0), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[107-117]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[107-124]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[107-124]
                Variable(2, Boolean) = Store Variable(1, Boolean) !dbg package_id=2 span=[103-104]
                Variable(3, Boolean) = Store Variable(2, Boolean) !dbg package_id=2 span=[134-135]
                Call id(3), args( Variable(3, Boolean), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[111-121]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[111-128]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[111-128]
                Variable(2, Boolean) = Store Variable(1, Boolean) !dbg package_id=2 span=[107-108]
                Variable(3, Boolean) = Store Variable(2, Boolean) !dbg package_id=2 span=[138-139]
                Call id(3), args( Variable(3, Boolean), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[106-116]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[106-123]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[106-123]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[106-123]
            Block 1:Block:
                Variable(3, Integer) = Store Variable(2, Integer) !dbg package_id=2 span=[102-103]
                Variable(4, Integer) = Store Variable(3, Integer) !dbg package_id=2 span=[141-142]
                Call id(3), args( Variable(4, Integer), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0) !dbg package_id=2 span=[126-127]
                Jump(1) !dbg package_id=2 span=[126-127]
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1) !dbg package_id=2 span=[130-131]
                Jump(1) !dbg package_id=2 span=[130-131]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[110-120]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[110-127]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[110-127]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[110-127]
            Block 1:Block:
                Variable(3, Integer) = Store Variable(2, Integer) !dbg package_id=2 span=[106-107]
                Variable(4, Integer) = Store Variable(3, Integer) !dbg package_id=2 span=[145-146]
                Call id(3), args( Variable(4, Integer), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0) !dbg package_id=2 span=[130-131]
                Jump(1) !dbg package_id=2 span=[130-131]
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1) !dbg package_id=2 span=[134-135]
                Jump(1) !dbg package_id=2 span=[134-135]"#]],
    );
}

#[test]
fn mutable_variable_in_outer_scope_set_to_mutable_from_inner_scope() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                use q = Qubit();
                mutable i = 0;
                if MResetZ(q) == One {
                    mutable j = 1;
                    set i = j;
                }
                else {
                    set i = 2;
                }
                return i;
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
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[106-107]
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[124-134]
                Variable(1, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[124-141]
                Variable(2, Boolean) = Store Variable(1, Boolean) !dbg package_id=2 span=[124-141]
                Branch Variable(2, Boolean), 2, 3 !dbg package_id=2 span=[124-141]
            Block 1:Block:
                Variable(4, Integer) = Store Variable(0, Integer) !dbg package_id=2 span=[267-268]
                Call id(3), args( Variable(4, Integer), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 2:Block:
                Variable(3, Integer) = Store Integer(1) !dbg package_id=2 span=[164-165]
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[187-188]
                Jump(1) !dbg package_id=2 span=[142-203]
            Block 3:Block:
                Variable(0, Integer) = Store Integer(2) !dbg package_id=2 span=[235-236]
                Jump(1) !dbg package_id=2 span=[212-251]"#]],
    );
}

#[test]
fn mutable_double_binding_does_generate_store_instruction() {
    let program = get_rir_program(indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Double {
                use q = Qubit();
                mutable d = MResetZ(q) == One ? 0.1 | 1.1;
                d
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
                name: __quantum__rt__double_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Double
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[113-123]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[113-130]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[113-130]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[113-130]
            Block 1:Block:
                Variable(3, Double) = Store Variable(2, Double) !dbg package_id=2 span=[109-110]
                Variable(4, Double) = Store Variable(3, Double) !dbg package_id=2 span=[152-153]
                Call id(3), args( Variable(4, Double), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 2:Block:
                Variable(2, Double) = Store Double(0.1) !dbg package_id=2 span=[133-136]
                Jump(1) !dbg package_id=2 span=[133-136]
            Block 3:Block:
                Variable(2, Double) = Store Double(1.1) !dbg package_id=2 span=[139-142]
                Jump(1) !dbg package_id=2 span=[139-142]"#]],
    );
}
