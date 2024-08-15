// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use num_bigint::BigUint;
use num_complex::Complex;
use qsc_eval::{
    backend::Backend,
    val::{Qubit, Result, Value},
};
use qsc_rir::rir::{BlockId, CallableId, VariableId};

/// Manages IDs for resources needed while performing partial evaluation.
#[derive(Default)]
pub struct ResourceManager {
    qubits_in_use: Vec<bool>,
    next_callable: CallableId,
    next_block: BlockId,
    next_result_register: usize,
    next_var: usize,
}

impl ResourceManager {
    /// Count of qubits used.
    pub fn qubit_count(&self) -> usize {
        self.qubits_in_use.len()
    }

    /// Count of results registers used.
    pub fn result_register_count(&self) -> usize {
        self.next_result_register
    }

    /// Allocates a qubit by favoring available qubit IDs before using new ones.
    pub fn allocate_qubit(&mut self) -> Qubit {
        if let Some(qubit_id) = self.qubits_in_use.iter().position(|in_use| !in_use) {
            self.qubits_in_use[qubit_id] = true;
            Qubit(qubit_id)
        } else {
            self.qubits_in_use.push(true);
            let qubit_id = self.qubits_in_use.len() - 1;
            Qubit(qubit_id)
        }
    }

    /// Releases a qubit ID for future use.
    pub fn release_qubit(&mut self, q: Qubit) {
        self.qubits_in_use[q.0] = false;
    }

    /// Gets the next block ID.
    pub fn next_block(&mut self) -> BlockId {
        let id = self.next_block;
        self.next_block = id.successor();
        id
    }

    /// Gets the next callable ID.
    pub fn next_callable(&mut self) -> CallableId {
        let id = self.next_callable;
        self.next_callable = id.successor();
        id
    }

    /// Gets the next result register ID.
    pub fn next_result_register(&mut self) -> Result {
        let result_id = self.next_result_register;
        self.next_result_register += 1;
        Result::Id(result_id)
    }

    /// Gets the next variable ID.
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
    type MeasurementType = usize;

    fn qubit_is_zero(&mut self, _q: usize) -> bool {
        // Because `qubit_is_zero` is called on every qubit release, this must return
        // true to avoid a panic.
        true
    }

    // Needed for calls to `DumpMachine` and `DumpRegister`.
    fn capture_quantum_state(&mut self) -> (Vec<(BigUint, Complex<f64>)>, usize) {
        (Vec::new(), 0)
    }

    // Only intrinsic functions are supported here since they're the only ones that will be classically evaluated.
    fn custom_intrinsic(
        &mut self,
        name: &str,
        _arg: Value,
    ) -> Option<std::result::Result<Value, String>> {
        match name {
            "BeginEstimateCaching" => Some(Ok(Value::Bool(true))),
            "EndEstimateCaching" | "GlobalPhase" => Some(Ok(Value::unit())),
            _ => None,
        }
    }
}
