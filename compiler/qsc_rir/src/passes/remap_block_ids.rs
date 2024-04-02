// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::VecDeque;

use rustc_hash::FxHashMap;

use crate::{
    rir::{BlockId, Instruction, Program},
    utils::get_block_successors,
};

#[cfg(test)]
mod tests;

/// Remaps block IDs in the given program to be contiguous, starting from 0,
/// and in a way that reflects control flow. Specifically, blocks in each new "layer" are
/// given new IDs such that the graph will be ordered by control flow.
/// This is useful for passes that need to iterate over all blocks in a program in a well-defined order.
/// For example, the following graph would be remapped as follows:
/// Entry: 2,
/// 2 -> 1,
/// 1 -> 0 or 3,
/// 0 -> 4,
/// 3 -> 4 or 6,
/// 4 -> 5,
/// 6 -> 5,
/// becomes:
/// Entry: 0,
/// 0 -> 1,
/// 1 -> 2 or 3,
/// 2 -> 4,
/// 3 -> 4 or 5,
/// 4 -> 6,
/// 5 -> 6
pub fn remap_block_ids(program: &mut Program) {
    // Only update the entry point.
    let entry_block_id = program
        .get_callable(program.entry)
        .body
        .expect("entry point should have a body block");

    let mut block_id_map = FxHashMap::default();
    let mut blocks_to_visit: VecDeque<BlockId> = vec![entry_block_id].into();
    let mut next_block_id = 0_usize;
    while let Some(block_id) = blocks_to_visit.pop_front() {
        if block_id_map.contains_key(&block_id) {
            continue;
        }

        block_id_map.insert(block_id, next_block_id);
        next_block_id += 1;

        blocks_to_visit.extend(get_block_successors(program.get_block(block_id)));
    }

    let blocks = program.blocks.drain().collect::<Vec<_>>();
    for (old_block_id, mut block) in blocks {
        let new_block_id = block_id_map[&old_block_id];
        update_instr(
            &block_id_map,
            block
                .0
                .last_mut()
                .expect("block should have at least one instruction"),
        );
        program.blocks.insert(new_block_id.into(), block);
    }
    program
        .callables
        .get_mut(program.entry)
        .expect("entry should exist")
        .body = Some(block_id_map[&entry_block_id].into());
}

fn update_instr(block_id_map: &FxHashMap<BlockId, usize>, instruction: &mut Instruction) {
    match instruction {
        Instruction::Jump(target) => {
            *target = block_id_map[target].into();
        }
        Instruction::Branch(_, target1, target2) => {
            *target1 = block_id_map[target1].into();
            *target2 = block_id_map[target2].into();
        }
        _ => {}
    }
}
