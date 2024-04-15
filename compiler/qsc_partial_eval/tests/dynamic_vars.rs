// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes, clippy::similar_names)]

pub mod test_utils;

use indoc::indoc;
use qsc_rir::rir::{BlockId, CallableId, Instruction, Literal, Operand, Ty, Variable};
use test_utils::{
    assert_block_instructions, assert_block_last_instruction, assert_callable,
    compile_and_partially_evaluate, mresetz_callable, read_result_callable,
};

#[test]
fn dynamic_int_from_if_expression_with_single_measurement_comparison_and_classical_blocks() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                let b = if r == Zero { 0 } else { 1 };
            }
        }
        "#,
    });
    println!("{program}");

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz_callable());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result_callable());

    // Set the IDs of the blocks we want to verify.
    let initial_block_id = BlockId(0);
    let continuation_block_id = BlockId(1);
    let if_block_id = BlockId(2);
    let else_block_id = BlockId(3);

    // Verify the branch instruction in the initial-block.
    let condition_var = Variable {
        variable_id: 1.into(),
        ty: Ty::Boolean,
    };
    let branch_inst = Instruction::Branch(condition_var, if_block_id, else_block_id);
    assert_block_last_instruction(&program, initial_block_id, &branch_inst);

    // Create the expected variable that will hold the dynamic integer value.
    let dynamic_int_var = Variable {
        variable_id: 2.into(),
        ty: Ty::Integer,
    };

    // Verify the instructions in the if-block.
    assert_block_instructions(
        &program,
        if_block_id,
        &[
            Instruction::Store(Operand::Literal(Literal::Integer(0)), dynamic_int_var),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions in the else-block.
    assert_block_instructions(
        &program,
        else_block_id,
        &[
            Instruction::Store(Operand::Literal(Literal::Integer(1)), dynamic_int_var),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions in the continuation-block.
    assert_block_instructions(&program, continuation_block_id, &[Instruction::Return]);
}

#[test]
#[should_panic(expected = "() cannot be mapped to a RIR operand")]
fn dynamic_int_from_if_expression_with_single_measurement_comparison_and_non_classical_blocks() {
    let _ = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1) = (Qubit(), Qubit());
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q0);
                let b = if r == Zero {
                    opA(q1);
                    0
                } else {
                    opB(q1);
                    1
                };
            }
        }
        "#,
    });
}
