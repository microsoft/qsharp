// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

mod test_utils;

use indoc::indoc;
use qsc_rir::rir::{
    BlockId, Callable, CallableId, CallableType, Instruction, Literal, Operand, Ty,
};
use test_utils::{assert_block_instructions, assert_callable, compile_and_partially_evaluate};

fn single_qubit_intrinsic() -> Callable {
    Callable {
        name: "op".to_string(),
        input_type: vec![Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

fn single_qubit_rotation_intrinsic() -> Callable {
    Callable {
        name: "rotation".to_string(),
        input_type: vec![Ty::Double, Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

#[test]
fn unitary_call_within_a_for_loop() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                for _ in 1..3 {
                    op(q);
                }
            }
        }
        "#,
    });

    let op_callable_id = CallableId(1);
    assert_callable(&program, op_callable_id, &single_qubit_intrinsic());
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
fn unitary_call_within_a_while_loop() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable idx = 0;
                while idx < 3 {
                    op(q);
                    set idx += 1;
                }
            }
        }
        "#,
    });

    let rotation_callable_id = CallableId(1);
    assert_callable(&program, rotation_callable_id, &single_qubit_intrinsic());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                rotation_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Call(
                rotation_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Call(
                rotation_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn unitary_call_within_a_repeat_until_loop() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable idx = 0;
                repeat {
                    op(q);
                    set idx += 1;
                } until idx >= 3;
            }
        }
        "#,
    });

    let op_callable_id = CallableId(1);
    assert_callable(&program, op_callable_id, &single_qubit_intrinsic());
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
fn rotation_call_within_a_for_loop() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation rotation(theta : Double, q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                for theta in [0.0, 1.0, 2.0] {
                    rotation(theta, q);
                }
            }
        }
        "#,
    });

    let rotation_callable_id = CallableId(1);
    assert_callable(
        &program,
        rotation_callable_id,
        &single_qubit_rotation_intrinsic(),
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                rotation_callable_id,
                vec![
                    Operand::Literal(Literal::Double(0.0)),
                    Operand::Literal(Literal::Qubit(0)),
                ],
                None,
            ),
            Instruction::Call(
                rotation_callable_id,
                vec![
                    Operand::Literal(Literal::Double(1.0)),
                    Operand::Literal(Literal::Qubit(0)),
                ],
                None,
            ),
            Instruction::Call(
                rotation_callable_id,
                vec![
                    Operand::Literal(Literal::Double(2.0)),
                    Operand::Literal(Literal::Qubit(0)),
                ],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn rotation_call_within_a_while_loop() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation rotation(theta : Double, q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let angles = [0.0, 1.0, 2.0];
                mutable idx = 0;
                while idx < 3 {
                    rotation(angles[idx], q);
                    set idx += 1;
                }
            }
        }
        "#,
    });

    let op_callable_id = CallableId(1);
    assert_callable(&program, op_callable_id, &single_qubit_rotation_intrinsic());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Double(0.0)),
                    Operand::Literal(Literal::Qubit(0)),
                ],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Double(1.0)),
                    Operand::Literal(Literal::Qubit(0)),
                ],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Double(2.0)),
                    Operand::Literal(Literal::Qubit(0)),
                ],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn rotation_call_within_a_repeat_until_loop() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation rotation(theta : Double, q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let angles = [0.0, 1.0, 2.0];
                mutable idx = 0;
                repeat {
                    rotation(angles[idx], q);
                    set idx += 1;
                } until idx >= 3;
            }
        }
        "#,
    });

    let rotation_callable_id = CallableId(1);
    assert_callable(
        &program,
        rotation_callable_id,
        &single_qubit_rotation_intrinsic(),
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                rotation_callable_id,
                vec![
                    Operand::Literal(Literal::Double(0.0)),
                    Operand::Literal(Literal::Qubit(0)),
                ],
                None,
            ),
            Instruction::Call(
                rotation_callable_id,
                vec![
                    Operand::Literal(Literal::Double(1.0)),
                    Operand::Literal(Literal::Qubit(0)),
                ],
                None,
            ),
            Instruction::Call(
                rotation_callable_id,
                vec![
                    Operand::Literal(Literal::Double(2.0)),
                    Operand::Literal(Literal::Qubit(0)),
                ],
                None,
            ),
            Instruction::Return,
        ],
    );
}
