// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::noise::PauliNoise;
use crate::val::Value;
use num_bigint::BigUint;
use num_complex::Complex;
use quantum_sparse_sim::QuantumSim;
use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};

#[cfg(test)]
mod noise_tests;

/// The trait that must be implemented by a quantum backend, whose functions will be invoked when
/// quantum intrinsics are called.
pub trait Backend {
    type ResultType;

    fn ccx(&mut self, _ctl0: usize, _ctl1: usize, _q: usize) {
        unimplemented!("ccx gate");
    }
    fn cx(&mut self, _ctl: usize, _q: usize) {
        unimplemented!("cx gate");
    }
    fn cy(&mut self, _ctl: usize, _q: usize) {
        unimplemented!("cy gate");
    }
    fn cz(&mut self, _ctl: usize, _q: usize) {
        unimplemented!("cz gate");
    }
    fn h(&mut self, _q: usize) {
        unimplemented!("h gate");
    }
    fn m(&mut self, _q: usize) -> Self::ResultType {
        unimplemented!("m operation");
    }
    fn mresetz(&mut self, _q: usize) -> Self::ResultType {
        unimplemented!("mresetz operation");
    }
    fn reset(&mut self, _q: usize) {
        unimplemented!("reset gate");
    }
    fn rx(&mut self, _theta: f64, _q: usize) {
        unimplemented!("rx gate");
    }
    fn rxx(&mut self, _theta: f64, _q0: usize, _q1: usize) {
        unimplemented!("rxx gate");
    }
    fn ry(&mut self, _theta: f64, _q: usize) {
        unimplemented!("ry gate");
    }
    fn ryy(&mut self, _theta: f64, _q0: usize, _q1: usize) {
        unimplemented!("ryy gate");
    }
    fn rz(&mut self, _theta: f64, _q: usize) {
        unimplemented!("rz gate");
    }
    fn rzz(&mut self, _theta: f64, _q0: usize, _q1: usize) {
        unimplemented!("rzz gate");
    }
    fn sadj(&mut self, _q: usize) {
        unimplemented!("sadj gate");
    }
    fn s(&mut self, _q: usize) {
        unimplemented!("s gate");
    }
    fn swap(&mut self, _q0: usize, _q1: usize) {
        unimplemented!("swap gate");
    }
    fn tadj(&mut self, _q: usize) {
        unimplemented!("tadj gate");
    }
    fn t(&mut self, _q: usize) {
        unimplemented!("t gate");
    }
    fn x(&mut self, _q: usize) {
        unimplemented!("x gate");
    }
    fn y(&mut self, _q: usize) {
        unimplemented!("y gate");
    }
    fn z(&mut self, _q: usize) {
        unimplemented!("z gate");
    }
    fn qubit_allocate(&mut self) -> usize {
        unimplemented!("qubit_allocate operation");
    }
    /// `false` indicates that the qubit was in a non-zero state before the release,
    /// but should have been in the zero state.
    /// `true` otherwise. This includes the case when the qubit was in
    /// a non-zero state during a noisy simulation, which is allowed.
    fn qubit_release(&mut self, _q: usize) -> bool {
        unimplemented!("qubit_release operation");
    }
    fn qubit_swap_id(&mut self, _q0: usize, _q1: usize) {
        unimplemented!("qubit_swap_id operation");
    }
    fn capture_quantum_state(&mut self) -> (Vec<(BigUint, Complex<f64>)>, usize) {
        unimplemented!("capture_quantum_state operation");
    }
    fn qubit_is_zero(&mut self, _q: usize) -> bool {
        unimplemented!("qubit_is_zero operation");
    }
    /// Executes custom intrinsic specified by `_name`.
    /// Returns None if this intrinsic is unknown.
    /// Otherwise returns Some(Result), with the Result from intrinsic.
    fn custom_intrinsic(&mut self, _name: &str, _arg: Value) -> Option<Result<Value, String>> {
        None
    }
    fn set_seed(&mut self, _seed: Option<u64>) {}
}

/// Default backend used when targeting sparse simulation.
pub struct SparseSim {
    /// Noiseless Sparse simulator to be used by this instance.
    pub sim: QuantumSim,
    /// Pauli noise that is applied after a gate or before a measurement is executed.
    /// Service functions aren't subject to noise.
    pub noise: PauliNoise,
    /// Random number generator to sample Pauli noise.
    /// Noise is not applied when rng is None.
    pub rng: Option<StdRng>,
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
            sim: QuantumSim::new(None),
            noise: PauliNoise::default(),
            rng: None,
        }
    }

    #[must_use]
    pub fn new_with_noise(noise: &PauliNoise) -> Self {
        let mut sim = SparseSim::new();
        sim.set_noise(noise);
        sim
    }

    fn set_noise(&mut self, noise: &PauliNoise) {
        self.noise = *noise;
        if noise.is_noiseless() {
            self.rng = None;
        } else {
            self.rng = Some(StdRng::from_entropy());
        }
    }

    #[must_use]
    fn is_noiseless(&self) -> bool {
        self.rng.is_none()
    }

    fn apply_noise(&mut self, q: usize) {
        if let Some(rng) = &mut self.rng {
            let p = rng.gen_range(0.0..1.0);
            if p >= self.noise.distribution[2] {
                // In the most common case we don't apply noise
            } else if p < self.noise.distribution[0] {
                self.sim.x(q);
            } else if p < self.noise.distribution[1] {
                self.sim.y(q);
            } else {
                self.sim.z(q);
            }
        }
        // No noise applied if rng is None.
    }
}

impl Backend for SparseSim {
    type ResultType = bool;

    fn ccx(&mut self, ctl0: usize, ctl1: usize, q: usize) {
        self.sim.mcx(&[ctl0, ctl1], q);
        self.apply_noise(ctl0);
        self.apply_noise(ctl1);
        self.apply_noise(q);
    }

    fn cx(&mut self, ctl: usize, q: usize) {
        self.sim.mcx(&[ctl], q);
        self.apply_noise(ctl);
        self.apply_noise(q);
    }

    fn cy(&mut self, ctl: usize, q: usize) {
        self.sim.mcy(&[ctl], q);
        self.apply_noise(ctl);
        self.apply_noise(q);
    }

    fn cz(&mut self, ctl: usize, q: usize) {
        self.sim.mcz(&[ctl], q);
        self.apply_noise(ctl);
        self.apply_noise(q);
    }

    fn h(&mut self, q: usize) {
        self.sim.h(q);
        self.apply_noise(q);
    }

    fn m(&mut self, q: usize) -> Self::ResultType {
        self.apply_noise(q);
        self.sim.measure(q)
    }

    fn mresetz(&mut self, q: usize) -> Self::ResultType {
        self.apply_noise(q); // Applying noise before measurement
        let res = self.sim.measure(q);
        if res {
            self.sim.x(q);
        }
        self.apply_noise(q); // Applying noise after reset
        res
    }

    fn reset(&mut self, q: usize) {
        self.mresetz(q);
        // Noise applied in mresetz.
    }

    fn rx(&mut self, theta: f64, q: usize) {
        self.sim.rx(theta, q);
        self.apply_noise(q);
    }

    fn rxx(&mut self, theta: f64, q0: usize, q1: usize) {
        self.sim.h(q0);
        self.sim.h(q1);
        self.sim.mcx(&[q1], q0);
        self.sim.rz(theta, q0);
        self.sim.mcx(&[q1], q0);
        self.sim.h(q1);
        self.sim.h(q0);
        self.apply_noise(q0);
        self.apply_noise(q1);
    }

    fn ry(&mut self, theta: f64, q: usize) {
        self.sim.ry(theta, q);
        self.apply_noise(q);
    }

    fn ryy(&mut self, theta: f64, q0: usize, q1: usize) {
        self.sim.h(q0);
        self.sim.s(q0);
        self.sim.h(q0);
        self.sim.h(q1);
        self.sim.s(q1);
        self.sim.h(q1);
        self.sim.mcx(&[q1], q0);
        self.sim.rz(theta, q0);
        self.sim.mcx(&[q1], q0);
        self.sim.h(q1);
        self.sim.sadj(q1);
        self.sim.h(q1);
        self.sim.h(q0);
        self.sim.sadj(q0);
        self.sim.h(q0);
        self.apply_noise(q0);
        self.apply_noise(q1);
    }

    fn rz(&mut self, theta: f64, q: usize) {
        self.sim.rz(theta, q);
        self.apply_noise(q);
    }

    fn rzz(&mut self, theta: f64, q0: usize, q1: usize) {
        self.sim.mcx(&[q1], q0);
        self.sim.rz(theta, q0);
        self.sim.mcx(&[q1], q0);
        self.apply_noise(q0);
        self.apply_noise(q1);
    }

    fn sadj(&mut self, q: usize) {
        self.sim.sadj(q);
        self.apply_noise(q);
    }

    fn s(&mut self, q: usize) {
        self.sim.s(q);
        self.apply_noise(q);
    }

    fn swap(&mut self, q0: usize, q1: usize) {
        self.sim.swap_qubit_ids(q0, q1);
        self.apply_noise(q0);
        self.apply_noise(q1);
    }

    fn tadj(&mut self, q: usize) {
        self.sim.tadj(q);
        self.apply_noise(q);
    }

    fn t(&mut self, q: usize) {
        self.sim.t(q);
        self.apply_noise(q);
    }

    fn x(&mut self, q: usize) {
        self.sim.x(q);
        self.apply_noise(q);
    }

    fn y(&mut self, q: usize) {
        self.sim.y(q);
        self.apply_noise(q);
    }

    fn z(&mut self, q: usize) {
        self.sim.z(q);
        self.apply_noise(q);
    }

    fn qubit_allocate(&mut self) -> usize {
        // Fresh qubit start in ground state even with noise.
        self.sim.allocate()
    }

    fn qubit_release(&mut self, q: usize) -> bool {
        if self.is_noiseless() {
            let was_zero = self.sim.qubit_is_zero(q);
            self.sim.release(q);
            was_zero
        } else {
            self.sim.release(q);
            true
        }
    }

    fn qubit_swap_id(&mut self, q0: usize, q1: usize) {
        // This is a service function rather than a gate so it doesn't incur noise.
        self.sim.swap_qubit_ids(q0, q1);
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
        // This is a service function rather than a measurement so it doesn't incur noise.
        self.sim.qubit_is_zero(q)
    }

    fn custom_intrinsic(&mut self, name: &str, arg: Value) -> Option<Result<Value, String>> {
        // These intrinsics aren't subject to noise.
        match name {
            "GlobalPhase" => {
                // Apply a global phase to the simulation by doing an Rz to a fresh qubit.
                // The controls list may be empty, in which case the phase is applied unconditionally.
                let [ctls_val, theta] = &*arg.unwrap_tuple() else {
                    panic!("tuple arity for GlobalPhase intrinsic should be 2");
                };
                let ctls = ctls_val
                    .clone()
                    .unwrap_array()
                    .iter()
                    .map(|q| q.clone().unwrap_qubit().deref().0)
                    .collect::<Vec<_>>();
                let q = self.sim.allocate();
                // The new qubit is by-definition in the |0⟩ state, so by reversing the sign of the
                // angle we can apply the phase to the entire state without increasing its size in memory.
                self.sim
                    .mcrz(&ctls, -2.0 * theta.clone().unwrap_double(), q);
                self.sim.release(q);
                Some(Ok(Value::unit()))
            }
            "BeginEstimateCaching" => Some(Ok(Value::Bool(true))),
            "EndEstimateCaching"
            | "AccountForEstimatesInternal"
            | "BeginRepeatEstimatesInternal"
            | "EndRepeatEstimatesInternal" => Some(Ok(Value::unit())),
            "ConfigurePauliNoise" => {
                let [xv, yv, zv] = &*arg.unwrap_tuple() else {
                    panic!("tuple arity for ConfigurePauliNoise intrinsic should be 3");
                };
                let px = xv.get_double();
                let py = yv.get_double();
                let pz = zv.get_double();
                match PauliNoise::from_probabilities(px, py, pz) {
                    Ok(noise) => {
                        self.set_noise(&noise);
                        Some(Ok(Value::unit()))
                    }
                    Err(message) => Some(Err(message)),
                }
            }
            "ApplyIdleNoise" => {
                let q = arg.unwrap_qubit().deref().0;
                self.apply_noise(q);
                Some(Ok(Value::unit()))
            }
            _ => None,
        }
    }

    fn set_seed(&mut self, seed: Option<u64>) {
        if let Some(seed) = seed {
            if !self.is_noiseless() {
                self.rng = Some(StdRng::seed_from_u64(seed));
            }
            self.sim.set_rng_seed(seed);
        } else {
            if !self.is_noiseless() {
                self.rng = Some(StdRng::from_entropy());
            }
            self.sim.set_rng_seed(rand::thread_rng().next_u64());
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

    fn qubit_release(&mut self, q: usize) -> bool {
        let _ = self.chained.qubit_release(q);
        self.main.qubit_release(q)
    }

    fn qubit_swap_id(&mut self, q0: usize, q1: usize) {
        self.chained.qubit_swap_id(q0, q1);
        self.main.qubit_swap_id(q0, q1);
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
