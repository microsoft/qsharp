// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_rir::rir;

use super::{check_unreachable_blocks, check_unreachable_callable, check_unreachable_instrs};

#[test]
#[should_panic(expected = "BlockId(0) does not end with a terminator instruction")]
fn test_check_unreachable_instrs_panics_on_missing_terminator() {
    let mut program = rir::Program::new();
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![rir::Instruction::BitwiseNot(
            rir::Value::Literal(rir::Literal::Bool(true)),
            rir::Variable {
                variable_id: rir::VariableId(0),
                ty: rir::Ty::Boolean,
            },
        )]),
    );
    check_unreachable_instrs(&program);
}

#[test]
fn test_check_unreachable_instrs_succeeds_on_terminator() {
    let mut program = rir::Program::new();
    program
        .blocks
        .insert(rir::BlockId(0), rir::Block(vec![rir::Instruction::Return]));
    check_unreachable_instrs(&program);
}

#[test]
fn test_check_unreachable_instrs_succeeds_on_terminator_after_other_instrs() {
    let mut program = rir::Program::new();
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![
            rir::Instruction::BitwiseNot(
                rir::Value::Literal(rir::Literal::Bool(true)),
                rir::Variable {
                    variable_id: rir::VariableId(0),
                    ty: rir::Ty::Boolean,
                },
            ),
            rir::Instruction::Return,
        ]),
    );
    check_unreachable_instrs(&program);
}

#[test]
#[should_panic(expected = "BlockId(0) has unreachable instructions after a terminator")]
fn test_check_unreachable_instrs_panics_on_unreachable_instrs_after_terminator() {
    let mut program = rir::Program::new();
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![
            rir::Instruction::Return,
            rir::Instruction::BitwiseNot(
                rir::Value::Literal(rir::Literal::Bool(true)),
                rir::Variable {
                    variable_id: rir::VariableId(0),
                    ty: rir::Ty::Boolean,
                },
            ),
        ]),
    );
    check_unreachable_instrs(&program);
}

#[test]
fn test_check_unreachable_blocks_succeeds_on_no_unreachable_blocks() {
    let mut program = rir::Program::new();
    program.callables.insert(
        rir::CallableId(0),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
            call_type: rir::CallableType::Other,
        },
    );
    program
        .blocks
        .insert(rir::BlockId(0), rir::Block(vec![rir::Instruction::Return]));
    check_unreachable_blocks(&program);
}

#[test]
#[should_panic(expected = "Unreachable blocks found: [BlockId(1)]")]
fn test_check_unreachable_blocks_panics_on_unreachable_block() {
    let mut program = rir::Program::new();
    program.callables.insert(
        rir::CallableId(0),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
            call_type: rir::CallableType::Other,
        },
    );
    program
        .blocks
        .insert(rir::BlockId(0), rir::Block(vec![rir::Instruction::Return]));
    program
        .blocks
        .insert(rir::BlockId(1), rir::Block(vec![rir::Instruction::Return]));
    check_unreachable_blocks(&program);
}

#[test]
fn test_check_unreachable_blocks_succeeds_on_no_unreachable_blocks_with_branch() {
    let mut program = rir::Program::new();
    program.callables.insert(
        rir::CallableId(0),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
            call_type: rir::CallableType::Other,
        },
    );
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![rir::Instruction::Branch(
            rir::Value::Literal(rir::Literal::Bool(true)),
            rir::BlockId(1),
            rir::BlockId(2),
        )]),
    );
    program
        .blocks
        .insert(rir::BlockId(1), rir::Block(vec![rir::Instruction::Return]));
    program
        .blocks
        .insert(rir::BlockId(2), rir::Block(vec![rir::Instruction::Return]));
    check_unreachable_blocks(&program);
}

#[test]
fn test_check_unreachable_blocks_succeeds_on_no_unreachable_blocks_with_jump() {
    let mut program = rir::Program::new();
    program.callables.insert(
        rir::CallableId(0),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
            call_type: rir::CallableType::Other,
        },
    );
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![
            rir::Instruction::Jump(rir::BlockId(1)),
            rir::Instruction::Return,
        ]),
    );
    program
        .blocks
        .insert(rir::BlockId(1), rir::Block(vec![rir::Instruction::Return]));
    check_unreachable_blocks(&program);
}

#[test]
#[should_panic(expected = "Unreachable blocks found: [BlockId(2)]")]
fn test_check_unreachable_blocks_panics_on_unreachable_block_with_branch() {
    let mut program = rir::Program::new();
    program.callables.insert(
        rir::CallableId(0),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
            call_type: rir::CallableType::Other,
        },
    );
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![rir::Instruction::Branch(
            rir::Value::Literal(rir::Literal::Bool(true)),
            rir::BlockId(1),
            rir::BlockId(1),
        )]),
    );
    program
        .blocks
        .insert(rir::BlockId(1), rir::Block(vec![rir::Instruction::Return]));
    program
        .blocks
        .insert(rir::BlockId(2), rir::Block(vec![rir::Instruction::Return]));
    check_unreachable_blocks(&program);
}

#[test]
fn test_check_unreachable_callable_succeeds_on_no_unreachable_callables() {
    let mut program = rir::Program::new();
    program.entry = rir::CallableId(0);
    program.callables.insert(
        rir::CallableId(0),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
            call_type: rir::CallableType::Other,
        },
    );
    program
        .blocks
        .insert(rir::BlockId(0), rir::Block(vec![rir::Instruction::Return]));
    check_unreachable_callable(&program);
}

#[test]
#[should_panic(expected = "Unreachable callables found: [CallableId(1)]")]
fn test_check_unreachable_callable_panics_on_unreachable_callable() {
    let mut program = rir::Program::new();
    program.entry = rir::CallableId(0);
    program.callables.insert(
        rir::CallableId(0),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
            call_type: rir::CallableType::Other,
        },
    );
    program.callables.insert(
        rir::CallableId(1),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(1)),
            call_type: rir::CallableType::Other,
        },
    );
    program
        .blocks
        .insert(rir::BlockId(0), rir::Block(vec![rir::Instruction::Return]));
    program
        .blocks
        .insert(rir::BlockId(1), rir::Block(vec![rir::Instruction::Return]));
    check_unreachable_callable(&program);
}

#[test]
fn test_check_unreachable_callable_succeeds_on_no_unreachable_callables_with_call() {
    let mut program = rir::Program::new();
    program.entry = rir::CallableId(0);
    program.callables.insert(
        rir::CallableId(0),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
            call_type: rir::CallableType::Other,
        },
    );
    program.callables.insert(
        rir::CallableId(1),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(1)),
            call_type: rir::CallableType::Other,
        },
    );
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![rir::Instruction::Call(
            rir::CallableId(1),
            Vec::new(),
            None,
        )]),
    );
    program
        .blocks
        .insert(rir::BlockId(1), rir::Block(vec![rir::Instruction::Return]));
    check_unreachable_callable(&program);
}

#[test]
fn test_check_unreachable_callable_succeeds_on_no_unreachable_callables_with_nested_call() {
    let mut program = rir::Program::new();
    program.entry = rir::CallableId(0);
    program.callables.insert(
        rir::CallableId(0),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
            call_type: rir::CallableType::Other,
        },
    );
    program.callables.insert(
        rir::CallableId(1),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(1)),
            call_type: rir::CallableType::Other,
        },
    );
    program.callables.insert(
        rir::CallableId(2),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: None,
            call_type: rir::CallableType::Other,
        },
    );
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![rir::Instruction::Call(
            rir::CallableId(1),
            Vec::new(),
            None,
        )]),
    );
    program.blocks.insert(
        rir::BlockId(1),
        rir::Block(vec![rir::Instruction::Call(
            rir::CallableId(2),
            Vec::new(),
            None,
        )]),
    );
    check_unreachable_callable(&program);
}

#[test]
#[should_panic(expected = "Unreachable callables found: [CallableId(2)]")]
fn test_check_unreachable_callable_panics_on_unreachable_callable_with_nested_call() {
    let mut program = rir::Program::new();
    program.entry = rir::CallableId(0);
    program.callables.insert(
        rir::CallableId(0),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
            call_type: rir::CallableType::Other,
        },
    );
    program.callables.insert(
        rir::CallableId(1),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(1)),
            call_type: rir::CallableType::Other,
        },
    );
    program.callables.insert(
        rir::CallableId(2),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: None,
            call_type: rir::CallableType::Other,
        },
    );
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![rir::Instruction::Call(
            rir::CallableId(1),
            Vec::new(),
            None,
        )]),
    );
    program
        .blocks
        .insert(rir::BlockId(1), rir::Block(vec![rir::Instruction::Return]));
    check_unreachable_callable(&program);
}

#[test]
fn test_check_unreachable_callable_succeeds_on_no_unreachable_callables_with_call_in_successor_block(
) {
    let mut program = rir::Program::new();
    program.entry = rir::CallableId(0);
    program.callables.insert(
        rir::CallableId(0),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
            call_type: rir::CallableType::Other,
        },
    );
    program.callables.insert(
        rir::CallableId(1),
        rir::Callable {
            name: "test".to_string(),
            input_type: vec![],
            output_type: None,
            body: None,
            call_type: rir::CallableType::Other,
        },
    );
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![rir::Instruction::Jump(rir::BlockId(1))]),
    );
    program.blocks.insert(
        rir::BlockId(1),
        rir::Block(vec![rir::Instruction::Call(
            rir::CallableId(1),
            Vec::new(),
            None,
        )]),
    );
    check_unreachable_callable(&program);
}
