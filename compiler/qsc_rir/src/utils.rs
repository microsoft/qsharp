// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::rir::{Block, BlockId, Instruction, Program};
use rustc_hash::FxHashSet;

/// Given a block, return the block IDs of its successors.
#[must_use]
pub fn get_block_successors(block: &Block) -> Vec<BlockId> {
    let mut successors = Vec::new();
    for instr in &block.0 {
        match instr {
            Instruction::Jump(target) => {
                successors.push(*target);
            }
            Instruction::Branch(_, target1, target2) => {
                successors.push(*target1);
                successors.push(*target2);
            }
            _ => {}
        }
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
