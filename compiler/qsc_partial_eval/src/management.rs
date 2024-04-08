// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_eval::{
    backend::Backend,
    val::{Qubit, Result},
};
use qsc_rir::rir::{BlockId, CallableId, VariableId};

#[derive(Default)]
pub struct Allocator {
    qubits_in_use: Vec<bool>,
    next_callable: CallableId,
    next_block: BlockId,
    next_result: usize,
    next_var: usize,
}

impl Allocator {
    pub fn qubit_allocate(&mut self) -> Qubit {
        if let Some(qubit_id) = self.qubits_in_use.iter().position(|in_use| !in_use) {
            Qubit(qubit_id)
        } else {
            self.qubits_in_use.push(true);
            let qubit_id = self.qubits_in_use.len() - 1;
            Qubit(qubit_id)
        }
    }

    pub fn qubit_release(&mut self, q: Qubit) {
        self.qubits_in_use[q.0] = false;
    }

    pub fn next_block(&mut self) -> BlockId {
        let id = self.next_block;
        self.next_block = id.successor();
        id
    }

    pub fn next_callable(&mut self) -> CallableId {
        let id = self.next_callable;
        self.next_callable = id.successor();
        id
    }

    pub fn next_result(&mut self) -> Result {
        let result_id = self.next_result;
        self.next_result += 1;
        Result::Id(result_id)
    }

    pub fn next_var(&mut self) -> VariableId {
        let var_id = self.next_var;
        self.next_var += 1;
        var_id.into()
    }
}

/// Custom backend meant to panic when most of its methods are called.
/// Since the partial evaluator is meant to generate instructions for most quantum operations, but we are also using
/// the evaluator for computations that are purely classical, the role of this backend is to catch instances of
/// quantum operations being simulated when they should not.
#[derive(Default)]
pub struct QuantumIntrinsicsChecker {}

impl Backend for QuantumIntrinsicsChecker {
    type ResultType = usize;

    fn qubit_is_zero(&mut self, _q: usize) -> bool {
        // Because `qubit_is_zero` is called on every qubit release, this must return
        // true to avoid a panic.
        true
    }
}
