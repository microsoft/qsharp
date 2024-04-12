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
fn unitary_call_within_an_if_with_classical_condition_within_a_for_loop() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                for idx in 0..5 {
                    if idx % 2 == 0 {
                        op(q);
                    }
                }
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
fn unitary_call_within_an_if_with_classical_condition_within_a_while_loop() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable idx = 0;
                while idx <= 5 {
                    if idx % 2 == 0 {
                        op(q);
                    }
                    set idx += 1;
                }
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
fn unitary_call_within_an_if_with_classical_condition_within_a_repeat_until_loop() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable idx = 0;
                repeat {
                    if idx % 2 == 0 {
                        op(q);
                    }
                    set idx += 1;
                } until idx > 5;
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
