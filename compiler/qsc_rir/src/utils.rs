// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::rir::{Block, BlockId, Instruction, Program};
use qsc_data_structures::index_map::IndexMap;
use rustc_hash::FxHashSet;

/// Given a block, return the block IDs of its successors.
#[must_use]
pub fn get_block_successors(block: &Block) -> Vec<BlockId> {
    let mut successors = Vec::new();
    // Assume that the block is well-formed and that terminators only appear as the last instruction.
    match block
        .0
        .last()
        .expect("block should have at least one instruction")
    {
        Instruction::Branch(_, target1, target2) => {
            successors.push(*target1);
            successors.push(*target2);
        }
        Instruction::Jump(target) => successors.push(*target),
        _ => {}
    }
    successors
}

/// Given a block ID and a containing program, return the block IDs of all blocks reachable from the given block including itself.
/// The returned block IDs are sorted in ascending order.
#[must_use]
pub fn get_all_block_successors(block: BlockId, program: &Program) -> Vec<BlockId> {
    let mut blocks_to_visit = vec![block];
    let mut blocks_visited = FxHashSet::default();
    while let Some(block_id) = blocks_to_visit.pop() {
        if blocks_visited.contains(&block_id) {
            continue;
        }
        blocks_visited.insert(block_id);
        let block = program.get_block(block_id);
        let block_successors = get_block_successors(block);
        blocks_to_visit.extend(block_successors.clone());
    }
    let mut successors = blocks_visited.into_iter().collect::<Vec<_>>();
    successors.sort_unstable();
    successors
}

/// Given a program, return a map from block IDs to the block IDs of their predecessors.
/// The vectors used as values in the map are sorted in ascending order, ensuring that block ids
/// for predecessors are listed lowest to highest.
#[must_use]
pub fn build_predecessors_map(program: &Program) -> IndexMap<BlockId, Vec<BlockId>> {
    let mut preds: IndexMap<BlockId, Vec<BlockId>> = IndexMap::default();

    for (block_id, block) in program.blocks.iter() {
        for successor in get_block_successors(block) {
            if let Some(preds_list) = preds.get_mut(successor) {
                preds_list.push(block_id);
            } else {
                preds.insert(successor, vec![block_id]);
            }
        }
    }

    for preds_list in preds.values_mut() {
        preds_list.sort_unstable();
    }

    preds
}
