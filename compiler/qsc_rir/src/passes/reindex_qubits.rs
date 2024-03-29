// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use rustc_hash::FxHashMap;

use crate::rir::{
    Block, Callable, CallableId, CallableType, Instruction, Literal, Program, Ty, Value,
};

/// Reindexes qubits after they have been measured or reset. This ensures there is no qubit reuse in
/// the program. As part of the pass, reset callables are removed and mresetz calls are replaced with
/// mz calls.
/// Note that this pass has several assumptions:
/// 1. Only one callable has a body, which is the entry point callable.
/// 2. The entry point callable has a single block.
/// 3. No dynamic qubits are used.
/// The pass will panic if the input program violates any of these assumptions.
pub fn reindex_qubits(program: &mut Program) {
    validate_assumptions(program);

    let (mut used_mz, mz_id) = match find_measurement_callable(program, "__quantum__qis__mz__body")
    {
        Some(id) => (true, id),
        None => (false, add_mz(program)),
    };
    let mresetz_id = find_measurement_callable(program, "__quantum__qis__mresetz__body");

    let mut qubit_map = FxHashMap::default();
    let mut next_qubit_id = program.num_qubits;
    let mut highest_used_id = next_qubit_id - 1;
    let mut new_block = Vec::new();
    let (block_id, block) = program
        .blocks
        .drain()
        .next()
        .expect("program should have at least one block");
    for instr in &block.0 {
        // Assume qubits only appear in void call instructions.
        match instr {
            Instruction::Call(call_id, args, _)
                if program.get_callable(*call_id).call_type == CallableType::Reset =>
            {
                // Generate any new qubit ids and skip adding the instruction.
                for arg in args {
                    if let Value::Literal(Literal::Qubit(qubit_id)) = arg {
                        qubit_map.insert(*qubit_id, next_qubit_id);
                        next_qubit_id += 1;
                    }
                }
            }
            Instruction::Call(call_id, args, None) => {
                // Map the qubit args, if any, and copy over the instruction.
                let new_args = args
                    .iter()
                    .map(|arg| match arg {
                        Value::Literal(Literal::Qubit(qubit_id)) => {
                            if let Some(mapped_id) = qubit_map.get(qubit_id) {
                                highest_used_id = highest_used_id.max(*mapped_id);
                                Value::Literal(Literal::Qubit(*mapped_id))
                            } else {
                                *arg
                            }
                        }
                        _ => *arg,
                    })
                    .collect::<Vec<_>>();

                // If the call was to mresetz, replace with mz.
                let call_id = if Some(*call_id) == mresetz_id {
                    used_mz = true;
                    mz_id
                } else {
                    *call_id
                };

                new_block.push(Instruction::Call(call_id, new_args, None));

                if program.get_callable(call_id).call_type == CallableType::Measurement {
                    // Generate any new qubit ids after a measurement.
                    for arg in args {
                        if let Value::Literal(Literal::Qubit(qubit_id)) = arg {
                            qubit_map.insert(*qubit_id, next_qubit_id);
                            next_qubit_id += 1;
                        }
                    }
                }
            }
            _ => {
                // Copy over the instruction.
                new_block.push(instr.clone());
            }
        }
    }

    program.num_qubits = highest_used_id + 1;
    program.blocks.clear();
    program.blocks.insert(block_id, Block(new_block));

    // All reset function calls should be removed, so remove them from the callables.
    let mut callables_to_remove = Vec::new();
    for (callable_id, callable) in program.callables.iter() {
        if callable.call_type == CallableType::Reset || Some(callable_id) == mresetz_id {
            callables_to_remove.push(callable_id);
        }
    }
    for callable_id in callables_to_remove {
        program.callables.remove(callable_id);
    }

    // If mz was added but not used, remove it.
    if !used_mz {
        program.callables.remove(mz_id);
    }
}

fn validate_assumptions(program: &Program) {
    // Ensure only one callable with a body exists.
    for (callable_id, callable) in program.callables.iter() {
        assert!(
            callable.body.is_none() || callable_id == program.entry,
            "Only the entry point callable should have a body"
        );
    }

    // Ensure entry point callable has a single block.
    // Future enhancements may allow multiple blocks in the entry point callable.
    assert!(
        program.blocks.iter().count() == 1,
        "Entry point callable must have a single block"
    );

    // Ensure that no dynamic qubits are used.
    let Some((_, block)) = program.blocks.iter().next() else {
        panic!("No blocks found in the program");
    };
    for instr in &block.0 {
        assert!(
            !matches!(instr, Instruction::Store(_, var) if var.ty == Ty::Qubit),
            "Dynamic qubits are not supported"
        );
    }
}

fn find_measurement_callable(program: &Program, name: &str) -> Option<CallableId> {
    for (callable_id, callable) in program.callables.iter() {
        if callable.call_type == CallableType::Measurement && callable.name == name {
            return Some(callable_id);
        }
    }
    None
}

fn add_mz(program: &mut Program) -> CallableId {
    let mz_id = CallableId(
        program
            .callables
            .iter()
            .map(|(id, _)| id.0)
            .max()
            .expect("should be at least one callable")
            + 1,
    );
    program.callables.insert(
        mz_id,
        Callable {
            name: "__quantum__qis__mz__body".to_string(),
            input_type: vec![Ty::Qubit, Ty::Result],
            output_type: None,
            body: None,
            call_type: CallableType::Measurement,
        },
    );
    mz_id
}
