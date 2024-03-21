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
    pub sim: QuantumSim,
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

/// Simple struct that chains two backends together so that the chained
/// backend is called before the main backend.
/// For any intrinsics that return a value,
/// the value returned by the chained backend is ignored.
/// The value returned by the main backend is returned.
pub struct Chain<T1, T2> {
    pub main: T1,
    pub chained: T2,
}

impl<T1, T2> Chain<T1, T2>
where
    T1: Backend,
    T2: Backend,
{
    pub fn new(primary: T1, chained: T2) -> Chain<T1, T2> {
        Chain {
            main: primary,
            chained,
        }
    }
}

impl<T1, T2> Backend for Chain<T1, T2>
where
    T1: Backend,
    T2: Backend,
{
    type ResultType = T1::ResultType;

    fn ccx(&mut self, ctl0: usize, ctl1: usize, q: usize) {
        self.chained.ccx(ctl0, ctl1, q);
        self.main.ccx(ctl0, ctl1, q);
    }

    fn cx(&mut self, ctl: usize, q: usize) {
        self.chained.cx(ctl, q);
        self.main.cx(ctl, q);
    }

    fn cy(&mut self, ctl: usize, q: usize) {
        self.chained.cy(ctl, q);
        self.main.cy(ctl, q);
    }

    fn cz(&mut self, ctl: usize, q: usize) {
        self.chained.cz(ctl, q);
        self.main.cz(ctl, q);
    }

    fn h(&mut self, q: usize) {
        self.chained.h(q);
        self.main.h(q);
    }

    fn m(&mut self, q: usize) -> Self::ResultType {
        let _ = self.chained.m(q);
        self.main.m(q)
    }

    fn mresetz(&mut self, q: usize) -> Self::ResultType {
        let _ = self.chained.mresetz(q);
        self.main.mresetz(q)
    }

    fn reset(&mut self, q: usize) {
        self.chained.reset(q);
        self.main.reset(q);
    }

    fn rx(&mut self, theta: f64, q: usize) {
        self.chained.rx(theta, q);
        self.main.rx(theta, q);
    }

    fn rxx(&mut self, theta: f64, q0: usize, q1: usize) {
        self.chained.rxx(theta, q0, q1);
        self.main.rxx(theta, q0, q1);
    }

    fn ry(&mut self, theta: f64, q: usize) {
        self.chained.ry(theta, q);
        self.main.ry(theta, q);
    }

    fn ryy(&mut self, theta: f64, q0: usize, q1: usize) {
        self.chained.ryy(theta, q0, q1);
        self.main.ryy(theta, q0, q1);
    }

    fn rz(&mut self, theta: f64, q: usize) {
        self.chained.rz(theta, q);
        self.main.rz(theta, q);
    }

    fn rzz(&mut self, theta: f64, q0: usize, q1: usize) {
        self.chained.rzz(theta, q0, q1);
        self.main.rzz(theta, q0, q1);
    }

    fn sadj(&mut self, q: usize) {
        self.chained.sadj(q);
        self.main.sadj(q);
    }

    fn s(&mut self, q: usize) {
        self.chained.s(q);
        self.main.s(q);
    }

    fn swap(&mut self, q0: usize, q1: usize) {
        self.chained.swap(q0, q1);
        self.main.swap(q0, q1);
    }

    fn tadj(&mut self, q: usize) {
        self.chained.tadj(q);
        self.main.tadj(q);
    }

    fn t(&mut self, q: usize) {
        self.chained.t(q);
        self.main.t(q);
    }

    fn x(&mut self, q: usize) {
        self.chained.x(q);
        self.main.x(q);
    }

    fn y(&mut self, q: usize) {
        self.chained.y(q);
        self.main.y(q);
    }

    fn z(&mut self, q: usize) {
        self.chained.z(q);
        self.main.z(q);
    }

    fn qubit_allocate(&mut self) -> usize {
        // Warning: we use the qubit id allocated by the
        // main backend, even for later calls into the chained
        // backend. This is not an issue today, but could
        // become an issue if the qubit ids differ between
        // the two backends.
        let _ = self.chained.qubit_allocate();
        self.main.qubit_allocate()
    }

    fn qubit_release(&mut self, q: usize) {
        self.chained.qubit_release(q);
        self.main.qubit_release(q);
    }

    fn capture_quantum_state(
        &mut self,
    ) -> (Vec<(num_bigint::BigUint, num_complex::Complex<f64>)>, usize) {
        let _ = self.chained.capture_quantum_state();
        self.main.capture_quantum_state()
    }

    fn qubit_is_zero(&mut self, q: usize) -> bool {
        let _ = self.chained.qubit_is_zero(q);
        self.main.qubit_is_zero(q)
    }

    fn custom_intrinsic(&mut self, name: &str, arg: Value) -> Option<Result<Value, String>> {
        let _ = self.chained.custom_intrinsic(name, arg.clone());
        self.main.custom_intrinsic(name, arg)
    }

    fn set_seed(&mut self, seed: Option<u64>) {
        self.chained.set_seed(seed);
        self.main.set_seed(seed);
    }
}
