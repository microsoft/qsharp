// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

mod test_utils;

use indoc::indoc;
use qsc_rir::rir::{
    BlockId, Callable, CallableId, CallableType, Instruction, Literal, Operand, Ty,
};
use test_utils::{assert_block_instructions, assert_callable, compile_and_partially_evaluate};

fn single_qubit_intrinsic_op_a() -> Callable {
    Callable {
        name: "opA".to_string(),
        input_type: vec![Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

fn single_qubit_intrinsic_op_b() -> Callable {
    Callable {
        name: "opB".to_string(),
        input_type: vec![Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

#[test]
fn if_expression_with_classical_condition_evaluates_true_branch() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                if true {
                    opA(q);
                }
            }
        }
        "#,
    });
    let op_a_callable_id = CallableId(1);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn if_expression_with_classical_condition_does_not_evaluate_true_branch() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                if false {
                    opA(q);
                }
            }
        }
        "#,
    });
    // This program is expected to just have the entry-point callable, whose block only has a return
    // intruction.
    assert_eq!(program.callables.iter().count(), 1);
    assert_block_instructions(&program, BlockId(0), &[Instruction::Return]);
}

#[test]
fn if_expression_with_classical_condition_evaluates_true_branch_and_not_false_branch() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                if true {
                    opA(q);
                } else {
                    opB(q);
                }
            }
        }
        "#,
    });
    let op_a_callable_id = CallableId(1);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn if_expression_with_classical_condition_evaluates_false_branch_and_not_true_branch() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                if false {
                    opA(q);
                } else {
                    opB(q);
                }
            }
        }
        "#,
    });
    let op_a_callable_id = CallableId(1);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_b());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}
