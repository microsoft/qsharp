// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

mod test_utils;

use indoc::indoc;
use qsc_rir::rir::{
    BlockId, Callable, CallableId, CallableType, Instruction, Literal, Operand, Ty,
};
use test_utils::{assert_block_instructions, assert_callable, compile_and_partially_evaluate};

fn single_qubit_intrinsic_op() -> Callable {
    Callable {
        name: "op".to_string(),
        input_type: vec![Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

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
    let op_callable_id = CallableId(1);
    assert_callable(&program, op_callable_id, &single_qubit_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
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
    assert_callable(&program, op_callable_id, &single_qubit_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(1))],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(2))],
                None,
            ),
            Instruction::Return,
        ],
    );
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
    assert_callable(&program, op_callable_id, &single_qubit_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
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
    assert_callable(&program, op_callable_id, &single_qubit_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(1))],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(2))],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(2))],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(3))],
                None,
            ),
            Instruction::Return,
        ],
    );
}
