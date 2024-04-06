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

#[derive(Default)]
pub struct QubitsAndResultsAllocator {
    qubit_id: usize,
    result_id: usize,
}

impl Backend for QubitsAndResultsAllocator {
    type ResultType = usize;

    fn m(&mut self, _q: usize) -> Self::ResultType {
        self.next_measurement()
    }

    fn mresetz(&mut self, _q: usize) -> Self::ResultType {
        self.next_measurement()
    }

    fn qubit_allocate(&mut self) -> usize {
        self.next_qubit()
    }

    fn qubit_release(&mut self, _q: usize) {
        // Do nothing.
    }

    fn qubit_is_zero(&mut self, _q: usize) -> bool {
        true
    }
}

impl QubitsAndResultsAllocator {
    fn next_measurement(&mut self) -> usize {
        let result_id = self.result_id;
        self.result_id += 1;
        result_id
    }

    fn next_qubit(&mut self) -> usize {
        let qubit_id = self.qubit_id;
        self.qubit_id += 1;
        qubit_id
    }
}
