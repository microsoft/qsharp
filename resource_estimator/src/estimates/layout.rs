// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::ErrorBudget;

/// Trait to model post-layout logical overhead
pub trait Overhead {
    /// The number of logical qubits to execute the algorithm after mapping
    ///
    /// This number does not include qubit used to produce magic states.
    fn logical_qubits(&self) -> u64;

    /// The number of logical unit cycles to execute the algorithm
    ///
    /// This number is a lower bound for the execution time of the algorithm,
    /// and might be extended by assuming no-ops.
    fn logical_depth(&self, budget: &ErrorBudget) -> u64;

    /// The number of magic states
    ///
    /// The index is used to indicate the type of magic states and must be
    /// supported by available factory builders in the physical estimation.
    fn num_magic_states(&self, budget: &ErrorBudget, index: usize) -> u64;

    /// Computes the logical volume
    ///
    /// By default, the logical volume is the number of logical qubits
    /// multiplied with the logical depth.
    fn logical_volume(
        &self,
        budget: &ErrorBudget,
        #[allow(unused_variables)] adjusted_logical_depth: u64,
    ) -> u64 {
        self.logical_qubits() * self.logical_depth(budget)
    }
}
