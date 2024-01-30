// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use num_bigint::BigUint;
use num_complex::Complex;
use quantum_sparse_sim::QuantumSim;
use rand::RngCore;

use crate::val::Value;

/// The trait that must be implemented by a quantum backend, whose functions will be invoked when
/// quantum intrinsics are called.
pub trait Backend {
    type ResultType;

    fn ccx(&mut self, ctl0: usize, ctl1: usize, q: usize);
    fn cx(&mut self, ctl: usize, q: usize);
    fn cy(&mut self, ctl: usize, q: usize);
    fn cz(&mut self, ctl: usize, q: usize);
    fn h(&mut self, q: usize);
    fn m(&mut self, q: usize) -> Self::ResultType;
    fn mresetz(&mut self, q: usize) -> Self::ResultType;
    fn reset(&mut self, q: usize);
    fn rx(&mut self, theta: f64, q: usize);
    fn rxx(&mut self, theta: f64, q0: usize, q1: usize);
    fn ry(&mut self, theta: f64, q: usize);
    fn ryy(&mut self, theta: f64, q0: usize, q1: usize);
    fn rz(&mut self, theta: f64, q: usize);
    fn rzz(&mut self, theta: f64, q0: usize, q1: usize);
    fn sadj(&mut self, q: usize);
    fn s(&mut self, q: usize);
    fn swap(&mut self, q0: usize, q1: usize);
    fn tadj(&mut self, q: usize);
    fn t(&mut self, q: usize);
    fn x(&mut self, q: usize);
    fn y(&mut self, q: usize);
    fn z(&mut self, q: usize);
    fn qubit_allocate(&mut self) -> usize;
    fn qubit_release(&mut self, q: usize);
    fn capture_quantum_state(&mut self) -> (Vec<(BigUint, Complex<f64>)>, usize);
    fn qubit_is_zero(&mut self, q: usize) -> bool;

    fn custom_intrinsic(&mut self, _name: &str, _arg: Value) -> Option<Result<Value, String>> {
        None
    }

    fn set_seed(&mut self, _seed: Option<u64>) {}
}

/// Default backend used when targeting sparse simulation.
pub struct SparseSim {
    sim: QuantumSim,
}

impl Default for SparseSim {
    fn default() -> Self {
        Self::new()
    }
}

impl SparseSim {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sim: QuantumSim::new(),
        }
    }
}

impl Backend for SparseSim {
    type ResultType = bool;

    fn ccx(&mut self, ctl0: usize, ctl1: usize, q: usize) {
        self.sim.mcx(&[ctl0, ctl1], q);
    }

    fn cx(&mut self, ctl: usize, q: usize) {
        self.sim.mcx(&[ctl], q);
    }

    fn cy(&mut self, ctl: usize, q: usize) {
        self.sim.mcy(&[ctl], q);
    }

    fn cz(&mut self, ctl: usize, q: usize) {
        self.sim.mcz(&[ctl], q);
    }

    fn h(&mut self, q: usize) {
        self.sim.h(q);
    }

    fn m(&mut self, q: usize) -> Self::ResultType {
        self.sim.measure(q)
    }

    fn mresetz(&mut self, q: usize) -> Self::ResultType {
        let res = self.sim.measure(q);
        if res {
            self.sim.x(q);
        }
        res
    }

    fn reset(&mut self, q: usize) {
        self.mresetz(q);
    }

    fn rx(&mut self, theta: f64, q: usize) {
        self.sim.rx(theta, q);
    }

    fn rxx(&mut self, theta: f64, q0: usize, q1: usize) {
        self.h(q0);
        self.h(q1);
        self.rzz(theta, q0, q1);
        self.h(q1);
        self.h(q0);
    }

    fn ry(&mut self, theta: f64, q: usize) {
        self.sim.ry(theta, q);
    }

    fn ryy(&mut self, theta: f64, q0: usize, q1: usize) {
        self.h(q0);
        self.s(q0);
        self.h(q0);
        self.h(q1);
        self.s(q1);
        self.h(q1);
        self.rzz(theta, q0, q1);
        self.h(q1);
        self.sadj(q1);
        self.h(q1);
        self.h(q0);
        self.sadj(q0);
        self.h(q0);
    }

    fn rz(&mut self, theta: f64, q: usize) {
        self.sim.rz(theta, q);
    }

    fn rzz(&mut self, theta: f64, q0: usize, q1: usize) {
        self.cx(q1, q0);
        self.rz(theta, q0);
        self.cx(q1, q0);
    }

    fn sadj(&mut self, q: usize) {
        self.sim.sadj(q);
    }

    fn s(&mut self, q: usize) {
        self.sim.s(q);
    }

    fn swap(&mut self, q0: usize, q1: usize) {
        self.sim.swap_qubit_ids(q0, q1);
    }

    fn tadj(&mut self, q: usize) {
        self.sim.tadj(q);
    }

    fn t(&mut self, q: usize) {
        self.sim.t(q);
    }

    fn x(&mut self, q: usize) {
        self.sim.x(q);
    }

    fn y(&mut self, q: usize) {
        self.sim.y(q);
    }

    fn z(&mut self, q: usize) {
        self.sim.z(q);
    }

    fn qubit_allocate(&mut self) -> usize {
        self.sim.allocate()
    }

    fn qubit_release(&mut self, q: usize) {
        self.sim.release(q);
    }

    fn capture_quantum_state(&mut self) -> (Vec<(BigUint, Complex<f64>)>, usize) {
        let (state, count) = self.sim.get_state();
        // Because the simulator returns the state indices with opposite endianness from the
        // expected one, we need to reverse the bit order of the indices.
        let mut new_state = state
            .into_iter()
            .map(|(idx, val)| {
                let mut new_idx = BigUint::default();
                for i in 0..(count as u64) {
                    if idx.bit((count as u64) - 1 - i) {
                        new_idx.set_bit(i, true);
                    }
                }
                (new_idx, val)
            })
            .collect::<Vec<_>>();
        new_state.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        (new_state, count)
    }

    fn qubit_is_zero(&mut self, q: usize) -> bool {
        self.sim.qubit_is_zero(q)
    }

    fn custom_intrinsic(&mut self, name: &str, _arg: Value) -> Option<Result<Value, String>> {
        match name {
            "BeginEstimateCaching" => Some(Ok(Value::Bool(true))),
            "EndEstimateCaching"
            | "AccountForEstimatesInternal"
            | "BeginRepeatEstimatesInternal"
            | "EndRepeatEstimatesInternal" => Some(Ok(Value::unit())),
            _ => None,
        }
    }

    fn set_seed(&mut self, seed: Option<u64>) {
        match seed {
            Some(seed) => self.sim.set_rng_seed(seed),
            None => self.sim.set_rng_seed(rand::thread_rng().next_u64()),
        }
    }
}
