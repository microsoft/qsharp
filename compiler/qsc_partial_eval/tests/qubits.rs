// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use expect_test::expect;
use indoc::indoc;
use qsc_rir::rir::{BlockId, CallableId};
use test_utils::{assert_block_instructions, assert_callable, compile_and_partially_evaluate};

#[test]
fn qubit_ids_are_correct_for_allocate_use_release_one_qubit() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                let q = QIR.Runtime.__quantum__rt__qubit_allocate();
                op(q);
                QIR.Runtime.__quantum__rt__qubit_release(q);
            }
        }
        "#,
    });
    expect![[r#"
        Callable:
            name: op
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]]
    .assert_eq(&program.get_callable(CallableId(1)).to_string());
    expect![[r#"
        Callable:
            name: __quantum__rt__tuple_record_output
            call_type: OutputRecording
            input_type:
                [0]: Integer
                [1]: Pointer
            output_type: <VOID>
            body: <NONE>"#]]
    .assert_eq(&program.get_callable(CallableId(2)).to_string());
    expect![[r#"
        Block:
            Call id(1), args( Qubit(0), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());
}

#[test]
fn qubit_ids_are_correct_for_allocate_use_release_multiple_qubits() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                let q0 = QIR.Runtime.__quantum__rt__qubit_allocate();
                let q1 = QIR.Runtime.__quantum__rt__qubit_allocate();
                let q2 = QIR.Runtime.__quantum__rt__qubit_allocate();
                op(q0);
                op(q1);
                op(q2);
                QIR.Runtime.__quantum__rt__qubit_release(q2);
                QIR.Runtime.__quantum__rt__qubit_release(q1);
                QIR.Runtime.__quantum__rt__qubit_release(q0);
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: op
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let tuple_callable_id = CallableId(2);
    assert_callable(
        &program,
        tuple_callable_id,
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
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
        Block:
            Call id(1), args( Qubit(0), )
            Call id(1), args( Qubit(1), )
            Call id(1), args( Qubit(2), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
    );
    assert_eq!(program.num_qubits, 3);
    assert_eq!(program.num_results, 0);
}

#[test]
fn qubit_ids_are_correct_for_allocate_use_release_one_qubit_multiple_times() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                let q0 = QIR.Runtime.__quantum__rt__qubit_allocate();
                op(q0);
                QIR.Runtime.__quantum__rt__qubit_release(q0);
                let q1 = QIR.Runtime.__quantum__rt__qubit_allocate();
                op(q1);
                QIR.Runtime.__quantum__rt__qubit_release(q1);
                let q2 = QIR.Runtime.__quantum__rt__qubit_allocate();
                op(q2);
                QIR.Runtime.__quantum__rt__qubit_release(q2);
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: op
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let tuple_callable_id = CallableId(2);
    assert_callable(
        &program,
        tuple_callable_id,
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
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
        Block:
            Call id(1), args( Qubit(0), )
            Call id(1), args( Qubit(0), )
            Call id(1), args( Qubit(0), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
    );
    assert_eq!(program.num_qubits, 1);
    assert_eq!(program.num_results, 0);
}

#[test]
fn qubit_ids_are_correct_for_allocate_use_release_multiple_qubits_interleaved() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                let q0 = QIR.Runtime.__quantum__rt__qubit_allocate();
                op(q0);
                let q1 = QIR.Runtime.__quantum__rt__qubit_allocate();
                op(q1);
                let q2 = QIR.Runtime.__quantum__rt__qubit_allocate();
                op(q2);
                QIR.Runtime.__quantum__rt__qubit_release(q2);
                let q3 = QIR.Runtime.__quantum__rt__qubit_allocate();
                let q4 = QIR.Runtime.__quantum__rt__qubit_allocate();
                op(q3);
                op(q4);
                QIR.Runtime.__quantum__rt__qubit_release(q4);
                QIR.Runtime.__quantum__rt__qubit_release(q3);
                QIR.Runtime.__quantum__rt__qubit_release(q1);
                QIR.Runtime.__quantum__rt__qubit_release(q0);
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: op
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let tuple_callable_id = CallableId(2);
    assert_callable(
        &program,
        tuple_callable_id,
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
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
        Block:
            Call id(1), args( Qubit(0), )
            Call id(1), args( Qubit(1), )
            Call id(1), args( Qubit(2), )
            Call id(1), args( Qubit(2), )
            Call id(1), args( Qubit(3), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
    );
    assert_eq!(program.num_qubits, 4);
    assert_eq!(program.num_results, 0);
}
