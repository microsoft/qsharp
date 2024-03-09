// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub trait Overhead {
    fn logical_qubits(&self) -> u64;
    fn logical_qubits_without_padding(&self) -> u64 {
        self.logical_qubits()
    }
    fn logical_depth(&self, num_magic_states_per_rotation: u64) -> u64;
    fn num_magic_states(&self, num_magic_states_per_rotation: u64) -> u64;
    fn num_magic_states_per_rotation(&self, eps_synthesis: f64) -> Option<u64>;
}
