// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::index_map::IndexMap;

use crate::{
    rir::{BlockId, Program},
    utils::build_predecessors_map,
};

#[cfg(test)]
mod tests;

/// Given a program, return a map from block IDs to the block ID of its immediate dominator. From this,
/// the dominator tree can be constructed by treating the map as a directed graph where the keys are the
/// children and the values are the parents.
/// This algorithm is from [A Simple, Fast Dominance Algorithm](http://www.hipersoft.rice.edu/grads/publications/dom14.pdf)
/// by Cooper, Harvey, and Kennedy, with two notable differences:
/// - Blocks are assumed to be sequentially numbered starting from 0 in reverse postorder rather than depth first order.
/// - Given that reversal, intersection between nodes uses the lesser of the two nodes rather than the greater.
#[must_use]
pub fn build_dominator_graph(program: &Program) -> IndexMap<BlockId, BlockId> {
    let mut doms = IndexMap::default();
    let entry_block_id = program
        .get_callable(program.entry)
        .body
        .expect("entry point should have a body");

    let preds = build_predecessors_map(program);

    // The entry block dominates itself.
    doms.insert(entry_block_id, entry_block_id);

    let mut changed = true;
    while changed {
        changed = false;
        for (block_id, _) in program.blocks.iter().skip(1) {
            let (first_pred, rest_preds) = preds
                .get(block_id)
                .expect("block should be present")
                .split_first()
                .expect("every block should have at least one predecessor");
            let mut new_dom = *first_pred;
            for pred in rest_preds {
                if doms.contains_key(*pred) {
                    new_dom = intersect(&doms, new_dom, *pred);
                }
            }
            if doms.get(block_id) != Some(&new_dom) {
                doms.insert(block_id, new_dom);
                changed = true;
            }
        }
    }

    doms
}

fn intersect(
    doms: &IndexMap<BlockId, BlockId>,
    mut block1: BlockId,
    mut block2: BlockId,
) -> BlockId {
    while block1 != block2 {
        while block1 > block2 {
            block1 = *doms.get(block1).expect("block should be present");
        }
        while block2 > block1 {
            block2 = *doms.get(block2).expect("block should be present");
        }
    }
    block1
}
