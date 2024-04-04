// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

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
