// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines, clippy::needless_raw_string_hashes)]

use expect_test::expect;
use qsc_data_structures::index_map::IndexMap;

use crate::{
    passes::remap_block_ids,
    rir::{
        Block, BlockId, Callable, CallableId, CallableType, Instruction, Literal, Program, Value,
    },
};

use super::build_dominator_graph;

/// Creates a new program with a single, entry callable that has block 0 as its body.
fn new_program() -> Program {
    let mut program = Program::new();
    program.entry = CallableId(0);
    program.callables.insert(
        CallableId(0),
        Callable {
            name: "main".to_string(),
            input_type: Vec::new(),
            output_type: None,
            body: Some(BlockId(0)),
            call_type: CallableType::Regular,
        },
    );
    program
}

fn display_dominator_graph(doms: &IndexMap<BlockId, BlockId>) -> String {
    let mut result = String::new();
    for (block_id, dom) in doms.iter() {
        result.push_str(&format!(
            "Block {} dominated by block {},\n",
            block_id.0, dom.0
        ));
    }
    result
}

#[test]
fn test_dominator_graph_single_block_dominates_itself() {
    let mut program = new_program();
    program
        .blocks
        .insert(BlockId(0), Block(vec![Instruction::Return]));

    remap_block_ids(&mut program);
    let doms = build_dominator_graph(&program);

    expect![[r#"
        Block 0 dominated by block 0,
    "#]]
    .assert_eq(&display_dominator_graph(&doms));
}

#[test]
fn test_dominator_graph_sequential_blocks_dominated_by_predecessor() {
    let mut program = new_program();
    program
        .blocks
        .insert(BlockId(0), Block(vec![Instruction::Jump(BlockId(1))]));
    program
        .blocks
        .insert(BlockId(1), Block(vec![Instruction::Jump(BlockId(2))]));
    program
        .blocks
        .insert(BlockId(2), Block(vec![Instruction::Return]));

    remap_block_ids(&mut program);
    let doms = build_dominator_graph(&program);

    expect![[r#"
        Block 0 dominated by block 0,
        Block 1 dominated by block 0,
        Block 2 dominated by block 1,
    "#]]
    .assert_eq(&display_dominator_graph(&doms));
}

#[test]
fn test_dominator_graph_branching_blocks_dominated_by_common_predecessor() {
    let mut program = new_program();
    program
        .blocks
        .insert(BlockId(0), Block(vec![Instruction::Jump(BlockId(1))]));
    program.blocks.insert(
        BlockId(1),
        Block(vec![Instruction::Branch(
            Value::Literal(Literal::Bool(true)),
            BlockId(2),
            BlockId(3),
        )]),
    );
    program
        .blocks
        .insert(BlockId(2), Block(vec![Instruction::Return]));
    program
        .blocks
        .insert(BlockId(3), Block(vec![Instruction::Return]));

    remap_block_ids(&mut program);
    let doms = build_dominator_graph(&program);

    expect![[r#"
        Block 0 dominated by block 0,
        Block 1 dominated by block 0,
        Block 2 dominated by block 1,
        Block 3 dominated by block 1,
    "#]]
    .assert_eq(&display_dominator_graph(&doms));
}

#[test]
fn test_dominator_graph_infinite_loop() {
    let mut program = new_program();
    program
        .blocks
        .insert(BlockId(0), Block(vec![Instruction::Jump(BlockId(1))]));
    program
        .blocks
        .insert(BlockId(1), Block(vec![Instruction::Jump(BlockId(1))]));

    remap_block_ids(&mut program);
    let doms = build_dominator_graph(&program);

    expect![[r#"
        Block 0 dominated by block 0,
        Block 1 dominated by block 0,
    "#]]
    .assert_eq(&display_dominator_graph(&doms));
}

#[test]
fn test_dominator_graph_branch_and_loop() {
    let mut program = new_program();
    program
        .blocks
        .insert(BlockId(0), Block(vec![Instruction::Jump(BlockId(1))]));
    program.blocks.insert(
        BlockId(1),
        Block(vec![Instruction::Branch(
            Value::Literal(Literal::Bool(true)),
            BlockId(2),
            BlockId(3),
        )]),
    );
    program
        .blocks
        .insert(BlockId(2), Block(vec![Instruction::Jump(BlockId(4))]));
    program
        .blocks
        .insert(BlockId(3), Block(vec![Instruction::Jump(BlockId(1))]));
    program
        .blocks
        .insert(BlockId(4), Block(vec![Instruction::Return]));

    remap_block_ids(&mut program);
    let doms = build_dominator_graph(&program);

    expect![[r#"
        Block 0 dominated by block 0,
        Block 1 dominated by block 0,
        Block 2 dominated by block 1,
        Block 3 dominated by block 1,
        Block 4 dominated by block 2,
    "#]]
    .assert_eq(&display_dominator_graph(&doms));
}

#[test]
fn test_dominator_graph_complex_structure_only_dominated_by_entry() {
    // This example comes from the paper from [A Simple, Fast Dominance Algorithm](http://www.hipersoft.rice.edu/grads/publications/dom14.pdf)
    // by Cooper, Harvey, and Kennedy and uses the node numbering from the paper. However, the resulting dominator graph
    // is different due to the numbering of the blocks, such that each block is numbered in reverse postorder.
    let mut program = new_program();
    program
        .callables
        .get_mut(CallableId(0))
        .expect("callable should be present")
        .body = Some(BlockId(6));
    program.blocks.insert(
        BlockId(6),
        Block(vec![Instruction::Branch(
            Value::Literal(Literal::Bool(true)),
            BlockId(5),
            BlockId(4),
        )]),
    );
    program
        .blocks
        .insert(BlockId(5), Block(vec![Instruction::Jump(BlockId(1))]));
    program.blocks.insert(
        BlockId(4),
        Block(vec![Instruction::Branch(
            Value::Literal(Literal::Bool(true)),
            BlockId(2),
            BlockId(3),
        )]),
    );
    program
        .blocks
        .insert(BlockId(1), Block(vec![Instruction::Jump(BlockId(2))]));
    program.blocks.insert(
        BlockId(2),
        Block(vec![Instruction::Branch(
            Value::Literal(Literal::Bool(true)),
            BlockId(3),
            BlockId(1),
        )]),
    );
    program
        .blocks
        .insert(BlockId(3), Block(vec![Instruction::Jump(BlockId(2))]));

    remap_block_ids(&mut program);
    let doms = build_dominator_graph(&program);

    expect![[r#"
        Block 0 dominated by block 0,
        Block 1 dominated by block 0,
        Block 2 dominated by block 0,
        Block 3 dominated by block 0,
        Block 4 dominated by block 0,
        Block 5 dominated by block 0,
    "#]]
    .assert_eq(&display_dominator_graph(&doms));
}

#[test]
fn test_dominator_graph_with_node_having_many_predicates() {
    let mut program = new_program();
    program.blocks.insert(
        BlockId(0),
        Block(vec![Instruction::Branch(
            Value::Literal(Literal::Bool(true)),
            BlockId(1),
            BlockId(2),
        )]),
    );
    program.blocks.insert(
        BlockId(1),
        Block(vec![Instruction::Branch(
            Value::Literal(Literal::Bool(true)),
            BlockId(3),
            BlockId(4),
        )]),
    );
    program.blocks.insert(
        BlockId(2),
        Block(vec![Instruction::Branch(
            Value::Literal(Literal::Bool(true)),
            BlockId(5),
            BlockId(6),
        )]),
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
    program
        .blocks
        .insert(BlockId(6), Block(vec![Instruction::Jump(BlockId(7))]));
    program
        .blocks
        .insert(BlockId(7), Block(vec![Instruction::Return]));

    remap_block_ids(&mut program);
    let doms = build_dominator_graph(&program);

    expect![[r#"
        Block 0 dominated by block 0,
        Block 1 dominated by block 0,
        Block 2 dominated by block 0,
        Block 3 dominated by block 1,
        Block 4 dominated by block 1,
        Block 5 dominated by block 2,
        Block 6 dominated by block 2,
        Block 7 dominated by block 0,
    "#]]
    .assert_eq(&display_dominator_graph(&doms));
}
