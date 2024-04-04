// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_eval::backend::Backend;
use qsc_rir::rir::{BlockId, CallableId};

#[derive(Default)]
pub struct Allocator {
    qubits_in_use: Vec<bool>,
    result_id: usize,
}

impl Allocator {
    pub fn qubit_allocate(&mut self) -> usize {
        if let Some(qubit_id) = self.qubits_in_use.iter().position(|in_use| !in_use) {
            qubit_id
        } else {
            self.qubits_in_use.push(true);
            self.qubits_in_use.len() - 1
        }
    }

    pub fn qubit_release(&mut self, q: usize) {
        self.qubits_in_use[q] = false;
    }

    pub fn next_result(&mut self) -> usize {
        let result_id = self.result_id;
        self.result_id += 1;
        result_id
    }
}

#[derive(Default)]
pub struct Assigner {
    next_callable: CallableId,
    next_block: BlockId,
}

impl Assigner {
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
