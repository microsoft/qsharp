// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

mod test_utils;

use indoc::indoc;
use qsc_rir::rir::{
    BlockId, Callable, CallableId, CallableType, Instruction, Literal, Operand, Ty, Variable,
};
use test_utils::{
    assert_block_instructions, assert_block_last_instruction, assert_callable,
    compile_and_partially_evaluate,
};

fn mresetz() -> Callable {
    Callable {
        name: "__quantum__qis__mresetz__body".to_string(),
        input_type: vec![Ty::Qubit, Ty::Result],
        output_type: None,
        body: None,
        call_type: CallableType::Measurement,
    }
}

fn read_result() -> Callable {
    Callable {
        name: "__quantum__rt__read_result__body".to_string(),
        input_type: vec![Ty::Result],
        output_type: Some(Ty::Boolean),
        body: None,
        call_type: CallableType::Readout,
    }
}

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
    let op_a_callable_id = CallableId(3);
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

#[test]
fn if_expression_with_dynamic_condition_evaluates_true_branch() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == Zero {
                    opA(q);
                }
            }
        }
        "#,
    });
    println!("{program}");

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result());
    let op_a_callable_id = CallableId(3);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());

    // Set the values of the block IDs we want to verify.
    let initial_block_id = BlockId(0);
    let continuation_block_id = BlockId(1);
    let if_true_block_id = BlockId(2);

    // Verify the branch instruction from the initial block.
    let condition_var = Variable {
        variable_id: 1.into(),
        ty: Ty::Boolean,
    };
    let branch_inst = Instruction::Branch(condition_var, if_true_block_id, continuation_block_id);
    assert_block_last_instruction(&program, initial_block_id, &branch_inst);

    // Verify the instructions of the if-true block.
    assert_block_instructions(
        &program,
        if_true_block_id,
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions of the continuation block.
    assert_block_instructions(&program, continuation_block_id, &[Instruction::Return]);
}
