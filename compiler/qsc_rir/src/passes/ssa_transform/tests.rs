// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines, clippy::needless_raw_string_hashes)]

use expect_test::expect;

use crate::{
    builder::{bell_program, new_program},
    passes::{build_dominator_graph, check_ssa_form, check_unreachable_code, remap_block_ids},
    rir::{
        Block, BlockId, Callable, CallableId, CallableType, Instruction, Operand, Program, Ty,
        Variable, VariableId,
    },
    utils::build_predecessors_map,
};

use super::transform_to_ssa;

fn transform_program(program: &mut Program) {
    check_unreachable_code(program);
    remap_block_ids(program);
    let preds = build_predecessors_map(program);
    transform_to_ssa(program, &preds);
    let doms = build_dominator_graph(program, &preds);
    check_ssa_form(program, &preds, &doms);
}

#[test]
fn ssa_transform_leaves_program_without_store_instruction_unchanged() {
    let mut program = bell_program();
    let program_string_orignal = program.to_string();

    transform_program(&mut program);

    assert_eq!(program_string_orignal, program.to_string());
}

#[test]
fn ssa_transform_leaves_branching_program_without_store_instruction_unchanged() {
    let mut program = bell_program();
    let program_string_orignal = program.to_string();

    transform_program(&mut program);

    assert_eq!(program_string_orignal, program.to_string());
}

#[test]
fn ssa_transform_removes_store_in_single_block_program() {
    let mut program = new_program();
    program.callables.insert(
        CallableId(1),
        Callable {
            name: "dynamic_bool".to_string(),
            input_type: Vec::new(),
            output_type: Some(Ty::Boolean),
            body: None,
            call_type: CallableType::Regular,
        },
    );

    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(
                CallableId(1),
                Vec::new(),
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Variable(1, Boolean) = Store Variable(0, Boolean)
                    Variable(2, Boolean) = LogicalNot Variable(1, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());

    // After
    transform_program(&mut program);
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Variable(2, Boolean) = LogicalNot Variable(0, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());
}

#[test]
fn ssa_transform_removes_multiple_stores_in_single_block_program() {
    let mut program = new_program();
    program.callables.insert(
        CallableId(1),
        Callable {
            name: "dynamic_bool".to_string(),
            input_type: Vec::new(),
            output_type: Some(Ty::Boolean),
            body: None,
            call_type: CallableType::Regular,
        },
    );

    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(
                CallableId(1),
                Vec::new(),
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(3),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(3),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(4),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Variable(1, Boolean) = Store Variable(0, Boolean)
                    Variable(2, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(2, Boolean)
                    Variable(3, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(3, Boolean)
                    Variable(4, Boolean) = LogicalNot Variable(1, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());

    // After
    transform_program(&mut program);
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Variable(2, Boolean) = LogicalNot Variable(0, Boolean)
                    Variable(3, Boolean) = LogicalNot Variable(2, Boolean)
                    Variable(4, Boolean) = LogicalNot Variable(3, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());
}

#[test]
fn ssa_transform_store_dominating_usage_propagates_to_successor_blocks() {
    let mut program = new_program();
    program.callables.insert(
        CallableId(1),
        Callable {
            name: "dynamic_bool".to_string(),
            input_type: Vec::new(),
            output_type: Some(Ty::Boolean),
            body: None,
            call_type: CallableType::Regular,
        },
    );

    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(
                CallableId(1),
                Vec::new(),
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Branch(
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
                BlockId(1),
                BlockId(2),
            ),
        ]),
    );
    program.blocks.insert(
        BlockId(1),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(3)),
        ]),
    );
    program.blocks.insert(
        BlockId(2),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(3),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(3)),
        ]),
    );
    program.blocks.insert(
        BlockId(3),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(4),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Variable(1, Boolean) = Store Variable(0, Boolean)
                    Branch Variable(1, Boolean), 1, 2
                Block 1: Block:
                    Variable(2, Boolean) = LogicalNot Variable(1, Boolean)
                    Jump(3)
                Block 2: Block:
                    Variable(3, Boolean) = LogicalNot Variable(1, Boolean)
                    Jump(3)
                Block 3: Block:
                    Variable(4, Boolean) = LogicalNot Variable(1, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());

    // After
    transform_program(&mut program);
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Branch Variable(0, Boolean), 1, 2
                Block 1: Block:
                    Variable(2, Boolean) = LogicalNot Variable(0, Boolean)
                    Jump(3)
                Block 2: Block:
                    Variable(3, Boolean) = LogicalNot Variable(0, Boolean)
                    Jump(3)
                Block 3: Block:
                    Variable(4, Boolean) = LogicalNot Variable(0, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());
}

#[test]
fn ssa_transform_store_dominating_usage_propagates_to_successor_blocks_without_intermediate_usage()
{
    let mut program = new_program();
    program.callables.insert(
        CallableId(1),
        Callable {
            name: "dynamic_bool".to_string(),
            input_type: Vec::new(),
            output_type: Some(Ty::Boolean),
            body: None,
            call_type: CallableType::Regular,
        },
    );

    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(
                CallableId(1),
                Vec::new(),
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Branch(
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
                BlockId(1),
                BlockId(2),
            ),
        ]),
    );
    program
        .blocks
        .insert(BlockId(1), Block(vec![Instruction::Jump(BlockId(3))]));
    program
        .blocks
        .insert(BlockId(2), Block(vec![Instruction::Jump(BlockId(3))]));
    program.blocks.insert(
        BlockId(3),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(4),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Variable(1, Boolean) = Store Variable(0, Boolean)
                    Branch Variable(1, Boolean), 1, 2
                Block 1: Block:
                    Jump(3)
                Block 2: Block:
                    Jump(3)
                Block 3: Block:
                    Variable(4, Boolean) = LogicalNot Variable(1, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());

    // After
    transform_program(&mut program);
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Branch Variable(0, Boolean), 1, 2
                Block 1: Block:
                    Jump(3)
                Block 2: Block:
                    Jump(3)
                Block 3: Block:
                    Variable(4, Boolean) = LogicalNot Variable(0, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());
}

#[test]
fn ssa_transform_inserts_phi_for_store_not_dominating_usage() {
    let mut program = new_program();
    program.callables.insert(
        CallableId(1),
        Callable {
            name: "dynamic_bool".to_string(),
            input_type: Vec::new(),
            output_type: Some(Ty::Boolean),
            body: None,
            call_type: CallableType::Regular,
        },
    );

    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(
                CallableId(1),
                Vec::new(),
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Branch(
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
                BlockId(1),
                BlockId(2),
            ),
        ]),
    );
    program.blocks.insert(
        BlockId(1),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(3)),
        ]),
    );
    program.blocks.insert(
        BlockId(2),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(3),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(3),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(3)),
        ]),
    );
    program.blocks.insert(
        BlockId(3),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(4),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Variable(1, Boolean) = Store Variable(0, Boolean)
                    Branch Variable(1, Boolean), 1, 2
                Block 1: Block:
                    Variable(2, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(2, Boolean)
                    Jump(3)
                Block 2: Block:
                    Variable(3, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(3, Boolean)
                    Jump(3)
                Block 3: Block:
                    Variable(4, Boolean) = LogicalNot Variable(1, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());

    // After
    transform_program(&mut program);
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Branch Variable(0, Boolean), 1, 2
                Block 1: Block:
                    Variable(2, Boolean) = LogicalNot Variable(0, Boolean)
                    Jump(3)
                Block 2: Block:
                    Variable(3, Boolean) = LogicalNot Variable(0, Boolean)
                    Jump(3)
                Block 3: Block:
                    Variable(5, Boolean) = Phi ( [Variable(2, Boolean), 1], [Variable(3, Boolean), 2], )
                    Variable(4, Boolean) = LogicalNot Variable(5, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]].assert_eq(&program.to_string());
}

#[test]
fn ssa_transform_inserts_phi_for_store_not_dominating_usage_in_one_branch() {
    let mut program = new_program();
    program.callables.insert(
        CallableId(1),
        Callable {
            name: "dynamic_bool".to_string(),
            input_type: Vec::new(),
            output_type: Some(Ty::Boolean),
            body: None,
            call_type: CallableType::Regular,
        },
    );

    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(
                CallableId(1),
                Vec::new(),
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Branch(
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
                BlockId(1),
                BlockId(2),
            ),
        ]),
    );
    program.blocks.insert(
        BlockId(1),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(3)),
        ]),
    );
    program
        .blocks
        .insert(BlockId(2), Block(vec![Instruction::Jump(BlockId(3))]));
    program.blocks.insert(
        BlockId(3),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(4),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Variable(1, Boolean) = Store Variable(0, Boolean)
                    Branch Variable(1, Boolean), 1, 2
                Block 1: Block:
                    Variable(2, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(2, Boolean)
                    Jump(3)
                Block 2: Block:
                    Jump(3)
                Block 3: Block:
                    Variable(4, Boolean) = LogicalNot Variable(1, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());

    // After
    transform_program(&mut program);
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Branch Variable(0, Boolean), 1, 2
                Block 1: Block:
                    Variable(2, Boolean) = LogicalNot Variable(0, Boolean)
                    Jump(3)
                Block 2: Block:
                    Jump(3)
                Block 3: Block:
                    Variable(5, Boolean) = Phi ( [Variable(2, Boolean), 1], [Variable(0, Boolean), 2], )
                    Variable(4, Boolean) = LogicalNot Variable(5, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]].assert_eq(&program.to_string());
}

#[test]
fn ssa_transform_inserts_phi_for_node_with_many_predecessors() {
    let mut program = new_program();
    program.callables.insert(
        CallableId(1),
        Callable {
            name: "dynamic_bool".to_string(),
            input_type: Vec::new(),
            output_type: Some(Ty::Boolean),
            body: None,
            call_type: CallableType::Regular,
        },
    );

    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(
                CallableId(1),
                Vec::new(),
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Branch(
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
                BlockId(1),
                BlockId(2),
            ),
        ]),
    );
    program.blocks.insert(
        BlockId(1),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Branch(
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
                BlockId(3),
                BlockId(4),
            ),
        ]),
    );
    program.blocks.insert(
        BlockId(2),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(3),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(3),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Branch(
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
                BlockId(5),
                BlockId(6),
            ),
        ]),
    );
    program
        .blocks
        .insert(BlockId(3), Block(vec![Instruction::Jump(BlockId(7))]));
    program
        .blocks
        .insert(BlockId(4), Block(vec![Instruction::Jump(BlockId(7))]));
    program
        .blocks
        .insert(BlockId(5), Block(vec![Instruction::Jump(BlockId(7))]));
    program.blocks.insert(
        BlockId(6),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(4),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(4),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(7)),
        ]),
    );
    program.blocks.insert(
        BlockId(7),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(5),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Variable(1, Boolean) = Store Variable(0, Boolean)
                    Branch Variable(1, Boolean), 1, 2
                Block 1: Block:
                    Variable(2, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(2, Boolean)
                    Branch Variable(1, Boolean), 3, 4
                Block 2: Block:
                    Variable(3, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(3, Boolean)
                    Branch Variable(1, Boolean), 5, 6
                Block 3: Block:
                    Jump(7)
                Block 4: Block:
                    Jump(7)
                Block 5: Block:
                    Jump(7)
                Block 6: Block:
                    Variable(4, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(4, Boolean)
                    Jump(7)
                Block 7: Block:
                    Variable(5, Boolean) = LogicalNot Variable(1, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());

    // After
    transform_program(&mut program);
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Branch Variable(0, Boolean), 1, 2
                Block 1: Block:
                    Variable(2, Boolean) = LogicalNot Variable(0, Boolean)
                    Branch Variable(2, Boolean), 3, 4
                Block 2: Block:
                    Variable(3, Boolean) = LogicalNot Variable(0, Boolean)
                    Branch Variable(3, Boolean), 5, 6
                Block 3: Block:
                    Jump(7)
                Block 4: Block:
                    Jump(7)
                Block 5: Block:
                    Jump(7)
                Block 6: Block:
                    Variable(4, Boolean) = LogicalNot Variable(3, Boolean)
                    Jump(7)
                Block 7: Block:
                    Variable(6, Boolean) = Phi ( [Variable(2, Boolean), 3], [Variable(2, Boolean), 4], [Variable(3, Boolean), 5], [Variable(4, Boolean), 6], )
                    Variable(5, Boolean) = LogicalNot Variable(6, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]].assert_eq(&program.to_string());
}

#[test]
fn ssa_transform_inserts_phi_for_multiple_stored_values() {
    let mut program = new_program();
    program.callables.insert(
        CallableId(1),
        Callable {
            name: "dynamic_bool".to_string(),
            input_type: Vec::new(),
            output_type: Some(Ty::Boolean),
            body: None,
            call_type: CallableType::Regular,
        },
    );

    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(
                CallableId(1),
                Vec::new(),
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Branch(
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
                BlockId(1),
                BlockId(2),
            ),
        ]),
    );
    program.blocks.insert(
        BlockId(1),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(3),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(3),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(3)),
        ]),
    );
    program.blocks.insert(
        BlockId(2),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(4),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(4),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(3)),
        ]),
    );
    program.blocks.insert(
        BlockId(3),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(5),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(6),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Variable(1, Boolean) = Store Variable(0, Boolean)
                    Variable(2, Boolean) = Store Variable(0, Boolean)
                    Branch Variable(1, Boolean), 1, 2
                Block 1: Block:
                    Variable(3, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(3, Boolean)
                    Jump(3)
                Block 2: Block:
                    Variable(4, Boolean) = LogicalNot Variable(2, Boolean)
                    Variable(2, Boolean) = Store Variable(4, Boolean)
                    Jump(3)
                Block 3: Block:
                    Variable(5, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(6, Boolean) = LogicalNot Variable(2, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());

    // After
    transform_program(&mut program);
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Branch Variable(0, Boolean), 1, 2
                Block 1: Block:
                    Variable(3, Boolean) = LogicalNot Variable(0, Boolean)
                    Jump(3)
                Block 2: Block:
                    Variable(4, Boolean) = LogicalNot Variable(0, Boolean)
                    Jump(3)
                Block 3: Block:
                    Variable(8, Boolean) = Phi ( [Variable(0, Boolean), 1], [Variable(4, Boolean), 2], )
                    Variable(7, Boolean) = Phi ( [Variable(3, Boolean), 1], [Variable(0, Boolean), 2], )
                    Variable(5, Boolean) = LogicalNot Variable(7, Boolean)
                    Variable(6, Boolean) = LogicalNot Variable(8, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]].assert_eq(&program.to_string());
}

#[test]
fn ssa_transform_inserts_phi_nodes_in_successive_blocks_for_chained_branches() {
    let mut program = new_program();
    program.callables.insert(
        CallableId(1),
        Callable {
            name: "dynamic_bool".to_string(),
            input_type: Vec::new(),
            output_type: Some(Ty::Boolean),
            body: None,
            call_type: CallableType::Regular,
        },
    );

    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(
                CallableId(1),
                Vec::new(),
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Branch(
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
                BlockId(1),
                BlockId(2),
            ),
        ]),
    );
    program.blocks.insert(
        BlockId(1),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Branch(
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
                BlockId(3),
                BlockId(4),
            ),
        ]),
    );
    program.blocks.insert(
        BlockId(2),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(3),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(3),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(5)),
        ]),
    );
    program.blocks.insert(
        BlockId(3),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(4),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(4),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(6)),
        ]),
    );
    program.blocks.insert(
        BlockId(4),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(5),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(5),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(6)),
        ]),
    );
    program.blocks.insert(
        BlockId(5),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(6),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(6),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(7)),
        ]),
    );
    program.blocks.insert(
        BlockId(6),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(7),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Store(
                Operand::Variable(Variable {
                    variable_id: VariableId(7),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Jump(BlockId(7)),
        ]),
    );
    program.blocks.insert(
        BlockId(7),
        Block(vec![
            Instruction::LogicalNot(
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(8),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Variable(1, Boolean) = Store Variable(0, Boolean)
                    Branch Variable(1, Boolean), 1, 2
                Block 1: Block:
                    Variable(2, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(2, Boolean)
                    Branch Variable(1, Boolean), 3, 4
                Block 2: Block:
                    Variable(3, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(3, Boolean)
                    Jump(5)
                Block 3: Block:
                    Variable(4, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(4, Boolean)
                    Jump(6)
                Block 4: Block:
                    Variable(5, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(5, Boolean)
                    Jump(6)
                Block 5: Block:
                    Variable(6, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(6, Boolean)
                    Jump(7)
                Block 6: Block:
                    Variable(7, Boolean) = LogicalNot Variable(1, Boolean)
                    Variable(1, Boolean) = Store Variable(7, Boolean)
                    Jump(7)
                Block 7: Block:
                    Variable(8, Boolean) = LogicalNot Variable(1, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]]
    .assert_eq(&program.to_string());

    // After
    transform_program(&mut program);
    expect![[r#"
        Program:
            entry: 0
            callables:
                Callable 0: Callable:
                    name: main
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  <VOID>
                    body:  0
                Callable 1: Callable:
                    name: dynamic_bool
                    call_type: Regular
                    input_type:  <VOID>
                    output_type:  Boolean
                    body:  <NONE>
            blocks:
                Block 0: Block:
                    Variable(0, Boolean) = Call id(1), args( )
                    Branch Variable(0, Boolean), 1, 2
                Block 1: Block:
                    Variable(2, Boolean) = LogicalNot Variable(0, Boolean)
                    Branch Variable(2, Boolean), 3, 4
                Block 2: Block:
                    Variable(3, Boolean) = LogicalNot Variable(0, Boolean)
                    Jump(5)
                Block 3: Block:
                    Variable(4, Boolean) = LogicalNot Variable(2, Boolean)
                    Jump(6)
                Block 4: Block:
                    Variable(5, Boolean) = LogicalNot Variable(2, Boolean)
                    Jump(6)
                Block 5: Block:
                    Variable(6, Boolean) = LogicalNot Variable(3, Boolean)
                    Jump(7)
                Block 6: Block:
                    Variable(9, Boolean) = Phi ( [Variable(4, Boolean), 3], [Variable(5, Boolean), 4], )
                    Variable(7, Boolean) = LogicalNot Variable(9, Boolean)
                    Jump(7)
                Block 7: Block:
                    Variable(10, Boolean) = Phi ( [Variable(6, Boolean), 5], [Variable(7, Boolean), 6], )
                    Variable(8, Boolean) = LogicalNot Variable(10, Boolean)
                    Return
            config: Config:
                remap_qubits_on_reuse: false
                defer_measurements: false
            num_qubits: 0
            num_results: 0"#]].assert_eq(&program.to_string());
}
