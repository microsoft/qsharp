// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes, clippy::similar_names)]

pub mod test_utils;

use indoc::indoc;
use qsc_rir::rir::{
    BlockId, Callable, CallableId, CallableType, Instruction, Literal, Operand, Ty, Variable,
};
use test_utils::{
    assert_block_instructions, assert_block_last_instruction, assert_callable,
    compile_and_partially_evaluate, mresetz_callable, read_result_callable,
};

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

fn single_qubit_intrinsic_op_c() -> Callable {
    Callable {
        name: "opC".to_string(),
        input_type: vec![Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

fn single_qubit_intrinsic_op_d() -> Callable {
    Callable {
        name: "opD".to_string(),
        input_type: vec![Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

#[test]
fn if_expression_with_true_condition() {
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
fn if_expression_with_false_condition() {
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
fn if_else_expression_with_true_condition() {
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
fn if_else_expression_with_false_condition() {
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
    let op_b_callable_id = CallableId(1);
    assert_callable(&program, op_b_callable_id, &single_qubit_intrinsic_op_b());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_b_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn if_elif_else_expression_with_true_elif_condition() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            operation opC(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                if false {
                    opA(q);
                } elif true {
                    opB(q);
                } else {
                    opC(q);
                }
            }
        }
        "#,
    });
    let op_b_callable_id = CallableId(1);
    assert_callable(&program, op_b_callable_id, &single_qubit_intrinsic_op_b());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_b_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn if_expression_with_dynamic_condition() {
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

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz_callable());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result_callable());
    let op_a_callable_id = CallableId(3);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());

    // Set the IDs of the blocks we want to verify.
    let initial_block_id = BlockId(0);
    let continuation_block_id = BlockId(1);
    let if_block_id = BlockId(2);

    // Verify the branch instruction in the initial-block.
    let condition_var = Variable {
        variable_id: 1.into(),
        ty: Ty::Boolean,
    };
    let branch_inst = Instruction::Branch(condition_var, if_block_id, continuation_block_id);
    assert_block_last_instruction(&program, initial_block_id, &branch_inst);

    // Verify the instructions in the if-block.
    assert_block_instructions(
        &program,
        if_block_id,
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions in the continuation-block.
    assert_block_instructions(&program, continuation_block_id, &[Instruction::Return]);
}

#[test]
fn if_else_expression_with_dynamic_condition() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == One {
                    opA(q);
                } else {
                    opB(q);
                }
            }
        }
        "#,
    });

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz_callable());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result_callable());
    let op_a_callable_id = CallableId(3);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());
    let op_b_callable_id = CallableId(4);
    assert_callable(&program, op_b_callable_id, &single_qubit_intrinsic_op_b());

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

    // Verify the instructions in the if-block.
    assert_block_instructions(
        &program,
        if_block_id,
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions in the else-block.
    assert_block_instructions(
        &program,
        else_block_id,
        &[
            Instruction::Call(
                op_b_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions in the continuation-block.
    assert_block_instructions(&program, continuation_block_id, &[Instruction::Return]);
}

#[test]
fn if_elif_else_expression_with_dynamic_condition() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            operation opC(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1, q2) = (Qubit(), Qubit(), Qubit());
                let r0 = QIR.Intrinsic.__quantum__qis__mresetz__body(q0);
                let r1 = QIR.Intrinsic.__quantum__qis__mresetz__body(q1);
                if r0 == One {
                    opA(q2);
                } elif r1 == One {
                    opB(q2);
                } else {
                    opC(q2);
                }
            }
        }
        "#,
    });

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz_callable());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result_callable());
    let op_a_callable_id = CallableId(3);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());
    let op_b_callable_id = CallableId(4);
    assert_callable(&program, op_b_callable_id, &single_qubit_intrinsic_op_b());
    let op_c_callable_id = CallableId(5);
    assert_callable(&program, op_c_callable_id, &single_qubit_intrinsic_op_c());

    // Set the IDs of the blocks we want to verify.
    let initial_block_id = BlockId(0);
    let continuation_block_id = BlockId(1);
    let if_block_id = BlockId(2);
    let else_block_id = BlockId(3);
    let nested_continuation_block_id = BlockId(4);
    let nested_if_block_id = BlockId(5);
    let nested_else_block_id = BlockId(6);

    // Verify the branch instruction in the initial-block.
    let condition_var = Variable {
        variable_id: 1.into(),
        ty: Ty::Boolean,
    };
    let branch_inst = Instruction::Branch(condition_var, if_block_id, else_block_id);
    assert_block_last_instruction(&program, initial_block_id, &branch_inst);

    // Verify the instructions in the if-block.
    assert_block_instructions(
        &program,
        if_block_id,
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(2))],
                None,
            ),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the branch instruction in the else-block.
    let nested_condition_var = Variable {
        variable_id: 3.into(),
        ty: Ty::Boolean,
    };
    let nested_branch_inst = Instruction::Branch(
        nested_condition_var,
        nested_if_block_id,
        nested_else_block_id,
    );
    assert_block_last_instruction(&program, else_block_id, &nested_branch_inst);

    // Verify the instructions in the nested-if-block.
    assert_block_instructions(
        &program,
        nested_if_block_id,
        &[
            Instruction::Call(
                op_b_callable_id,
                vec![Operand::Literal(Literal::Qubit(2))],
                None,
            ),
            Instruction::Jump(nested_continuation_block_id),
        ],
    );

    // Verify the instructions in the nested-else-block.
    assert_block_instructions(
        &program,
        nested_else_block_id,
        &[
            Instruction::Call(
                op_c_callable_id,
                vec![Operand::Literal(Literal::Qubit(2))],
                None,
            ),
            Instruction::Jump(nested_continuation_block_id),
        ],
    );

    // Verify the instructions in the continuation-block.
    assert_block_instructions(&program, continuation_block_id, &[Instruction::Return]);
}

#[test]
fn if_expression_with_dynamic_condition_and_nested_if_expression_with_true_condition() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == Zero {
                    if true {
                        opA(q);
                    }
                }
            }
        }
        "#,
    });

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz_callable());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result_callable());
    let op_a_callable_id = CallableId(3);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());

    // Set the IDs of the blocks we want to verify.
    let initial_block_id = BlockId(0);
    let continuation_block_id = BlockId(1);
    let if_block_id = BlockId(2);

    // Verify the branch instruction in the initial-block.
    let condition_var = Variable {
        variable_id: 1.into(),
        ty: Ty::Boolean,
    };
    let branch_inst = Instruction::Branch(condition_var, if_block_id, continuation_block_id);
    assert_block_last_instruction(&program, initial_block_id, &branch_inst);

    // Verify the instructions in the if-block.
    assert_block_instructions(
        &program,
        if_block_id,
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions in the continuation-block.
    assert_block_instructions(&program, continuation_block_id, &[Instruction::Return]);
}

#[test]
fn if_expression_with_dynamic_condition_and_nested_if_expression_with_false_condition() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == Zero {
                    if false {
                        opA(q);
                    }
                }
            }
        }
        "#,
    });

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz_callable());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result_callable());

    // Set the IDs of the blocks we want to verify.
    let initial_block_id = BlockId(0);
    let continuation_block_id = BlockId(1);
    let if_block_id = BlockId(2);

    // Verify the branch instruction in the initial-block.
    let condition_var = Variable {
        variable_id: 1.into(),
        ty: Ty::Boolean,
    };
    let branch_inst = Instruction::Branch(condition_var, if_block_id, continuation_block_id);
    assert_block_last_instruction(&program, initial_block_id, &branch_inst);

    // Verify the instructions in the if-block.
    assert_block_instructions(
        &program,
        if_block_id,
        &[Instruction::Jump(continuation_block_id)],
    );

    // Verify the instructions in the continuation-block.
    assert_block_instructions(&program, continuation_block_id, &[Instruction::Return]);
}

#[test]
fn if_else_expression_with_dynamic_condition_and_nested_if_expression_with_true_condition() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == One {
                    opA(q);
                } else {
                    if true {
                        opB(q);
                    }
                }
            }
        }
        "#,
    });

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz_callable());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result_callable());
    let op_a_callable_id = CallableId(3);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());
    let op_b_callable_id = CallableId(4);
    assert_callable(&program, op_b_callable_id, &single_qubit_intrinsic_op_b());

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

    // Verify the instructions in the if-block.
    assert_block_instructions(
        &program,
        if_block_id,
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions in the else-block.
    assert_block_instructions(
        &program,
        else_block_id,
        &[
            Instruction::Call(
                op_b_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions in the continuation-block.
    assert_block_instructions(&program, continuation_block_id, &[Instruction::Return]);
}

#[test]
fn if_else_expression_with_dynamic_condition_and_nested_if_expression_with_false_condition() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == One {
                    opA(q);
                } else {
                    if false {
                        opB(q);
                    }
                }
            }
        }
        "#,
    });

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz_callable());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result_callable());
    let op_a_callable_id = CallableId(3);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());

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

    // Verify the instructions in the if-block.
    assert_block_instructions(
        &program,
        if_block_id,
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions in the else-block.
    assert_block_instructions(
        &program,
        else_block_id,
        &[Instruction::Jump(continuation_block_id)],
    );

    // Verify the instructions in the continuation-block.
    assert_block_instructions(&program, continuation_block_id, &[Instruction::Return]);
}

#[test]
fn if_expression_with_dynamic_condition_and_nested_if_expression_with_dynamic_condition() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1, q2) = (Qubit(), Qubit(), Qubit());
                let r0 = QIR.Intrinsic.__quantum__qis__mresetz__body(q0);
                let r1 = QIR.Intrinsic.__quantum__qis__mresetz__body(q1);
                if r0 == Zero {
                    if r1 == One {
                        opA(q2);
                    }
                }
            }
        }
        "#,
    });

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz_callable());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result_callable());
    let op_a_callable_id = CallableId(3);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());

    // Set the IDs of the blocks we want to verify.
    let initial_block_id = BlockId(0);
    let continuation_block_id = BlockId(1);
    let if_block_id = BlockId(2);
    let nested_continuation_block_id = BlockId(3);
    let nested_if_block_id = BlockId(4);

    // Verify the branch instruction in the initial block.
    let condition_var = Variable {
        variable_id: 1.into(),
        ty: Ty::Boolean,
    };
    let branch_inst = Instruction::Branch(condition_var, if_block_id, continuation_block_id);
    assert_block_last_instruction(&program, initial_block_id, &branch_inst);

    // Verify the branch instruction in the if-block.
    let nested_condition_var = Variable {
        variable_id: 3.into(),
        ty: Ty::Boolean,
    };
    let nested_branch_inst = Instruction::Branch(
        nested_condition_var,
        nested_if_block_id,
        nested_continuation_block_id,
    );
    assert_block_last_instruction(&program, if_block_id, &nested_branch_inst);

    // Verify the instructions in the nested-if-block.
    assert_block_instructions(
        &program,
        nested_if_block_id,
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(2))],
                None,
            ),
            Instruction::Jump(nested_continuation_block_id),
        ],
    );

    // Verify the instructions in the nested-continuation-block.
    assert_block_instructions(
        &program,
        nested_continuation_block_id,
        &[Instruction::Jump(continuation_block_id)],
    );

    // Verify the instructions in the continuation-block.
    assert_block_instructions(&program, continuation_block_id, &[Instruction::Return]);
}

#[allow(clippy::too_many_lines)]
#[test]
fn doubly_nested_if_else_expressions_with_dynamic_conditions() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            operation opC(q : Qubit) : Unit { body intrinsic; }
            operation opD(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1, q2) = (Qubit(), Qubit(), Qubit());
                let r0 = QIR.Intrinsic.__quantum__qis__mresetz__body(q0);
                let r1 = QIR.Intrinsic.__quantum__qis__mresetz__body(q1);
                if r0 == Zero {
                    if r1 == Zero {
                        opA(q2);
                    } else {
                        opB(q2);
                    }
                } else {
                    if r1 == One{
                        opC(q2);
                    } else {
                        opD(q2);
                    }
                }
            }
        }
        "#,
    });

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz_callable());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result_callable());
    let op_a_callable_id = CallableId(3);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());
    let op_b_callable_id = CallableId(4);
    assert_callable(&program, op_b_callable_id, &single_qubit_intrinsic_op_b());
    let op_c_callable_id = CallableId(5);
    assert_callable(&program, op_c_callable_id, &single_qubit_intrinsic_op_c());
    let op_d_callable_id = CallableId(6);
    assert_callable(&program, op_d_callable_id, &single_qubit_intrinsic_op_d());

    // Set the IDs of the blocks we want to verify.
    let initial_block_id = BlockId(0);
    let continuation_block_id = BlockId(1);
    let if_block_id = BlockId(2);
    let else_block_id = BlockId(6);
    let first_nested_continuation_block_id = BlockId(3);
    let first_nested_if_block_id = BlockId(4);
    let first_nested_else_block_id = BlockId(5);
    let second_nested_continuation_block_id = BlockId(7);
    let second_nested_if_block_id = BlockId(8);
    let second_nested_else_block_id = BlockId(9);

    // Verify the branch instruction in the initial-block.
    let condition_var = Variable {
        variable_id: 1.into(),
        ty: Ty::Boolean,
    };
    let branch_inst = Instruction::Branch(condition_var, if_block_id, else_block_id);
    assert_block_last_instruction(&program, initial_block_id, &branch_inst);

    // Verify the branch instruction in the if-block.
    let first_nested_condition_var = Variable {
        variable_id: 3.into(),
        ty: Ty::Boolean,
    };
    let first_nested_branch_inst = Instruction::Branch(
        first_nested_condition_var,
        first_nested_if_block_id,
        first_nested_else_block_id,
    );
    assert_block_last_instruction(&program, if_block_id, &first_nested_branch_inst);

    // Verify the instructions in the first nested if-block.
    assert_block_instructions(
        &program,
        first_nested_if_block_id,
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(2))],
                None,
            ),
            Instruction::Jump(first_nested_continuation_block_id),
        ],
    );

    // Verify the instructions in the first nested else-block.
    assert_block_instructions(
        &program,
        first_nested_else_block_id,
        &[
            Instruction::Call(
                op_b_callable_id,
                vec![Operand::Literal(Literal::Qubit(2))],
                None,
            ),
            Instruction::Jump(first_nested_continuation_block_id),
        ],
    );

    // Verify the instructions in the first nested continuation-block.
    assert_block_instructions(
        &program,
        first_nested_continuation_block_id,
        &[Instruction::Jump(continuation_block_id)],
    );

    // Verify the branch instruction in the else-block.
    let second_nested_condition_var = Variable {
        variable_id: 5.into(),
        ty: Ty::Boolean,
    };
    let second_nested_branch_inst = Instruction::Branch(
        second_nested_condition_var,
        second_nested_if_block_id,
        second_nested_else_block_id,
    );
    assert_block_last_instruction(&program, else_block_id, &second_nested_branch_inst);

    // Verify the instructions in the second nested if-block.
    assert_block_instructions(
        &program,
        second_nested_if_block_id,
        &[
            Instruction::Call(
                op_c_callable_id,
                vec![Operand::Literal(Literal::Qubit(2))],
                None,
            ),
            Instruction::Jump(second_nested_continuation_block_id),
        ],
    );

    // Verify the instructions in the second nested else-block.
    assert_block_instructions(
        &program,
        second_nested_else_block_id,
        &[
            Instruction::Call(
                op_d_callable_id,
                vec![Operand::Literal(Literal::Qubit(2))],
                None,
            ),
            Instruction::Jump(second_nested_continuation_block_id),
        ],
    );

    // Verify the instructions in the second nested continuation-block.
    assert_block_instructions(
        &program,
        second_nested_continuation_block_id,
        &[Instruction::Jump(continuation_block_id)],
    );

    // Verify the instructions in the continuation-block.
    assert_block_instructions(&program, continuation_block_id, &[Instruction::Return]);
}

#[test]
fn if_expression_with_dynamic_condition_and_subsequent_call_to_operation() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == Zero {
                    opA(q);
                }
                opB(q);
            }
        }
        "#,
    });

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz_callable());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result_callable());
    let op_a_callable_id = CallableId(3);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());
    let op_b_callable_id = CallableId(4);
    assert_callable(&program, op_b_callable_id, &single_qubit_intrinsic_op_b());

    // Set the IDs of the blocks we want to verify.
    let initial_block_id = BlockId(0);
    let continuation_block_id = BlockId(1);
    let if_block_id = BlockId(2);

    // Verify the branch instruction in the initial-block.
    let condition_var = Variable {
        variable_id: 1.into(),
        ty: Ty::Boolean,
    };
    let branch_inst = Instruction::Branch(condition_var, if_block_id, continuation_block_id);
    assert_block_last_instruction(&program, initial_block_id, &branch_inst);

    // Verify the instructions in the if-block.
    assert_block_instructions(
        &program,
        if_block_id,
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions in the continuation-block.
    assert_block_instructions(
        &program,
        continuation_block_id,
        &[
            Instruction::Call(
                op_b_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn if_else_expression_with_dynamic_condition_and_subsequent_call_to_operation() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            operation opC(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == One {
                    opA(q);
                } else {
                    opB(q);
                }
                opC(q);
            }
        }
        "#,
    });

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(&program, mresetz_callable_id, &mresetz_callable());
    let read_result_callable_id = CallableId(2);
    assert_callable(&program, read_result_callable_id, &read_result_callable());
    let op_a_callable_id = CallableId(3);
    assert_callable(&program, op_a_callable_id, &single_qubit_intrinsic_op_a());
    let op_b_callable_id = CallableId(4);
    assert_callable(&program, op_b_callable_id, &single_qubit_intrinsic_op_b());
    let op_c_callable_id = CallableId(5);
    assert_callable(&program, op_c_callable_id, &single_qubit_intrinsic_op_c());

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

    // Verify the instructions in the if-block.
    assert_block_instructions(
        &program,
        if_block_id,
        &[
            Instruction::Call(
                op_a_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions in the else-block.
    assert_block_instructions(
        &program,
        else_block_id,
        &[
            Instruction::Call(
                op_b_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Jump(continuation_block_id),
        ],
    );

    // Verify the instructions in the continuation-block.
    assert_block_instructions(
        &program,
        continuation_block_id,
        &[
            Instruction::Call(
                op_c_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}
