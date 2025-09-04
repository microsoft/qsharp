// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![deny(clippy::all, clippy::pedantic)]

//! # Sparse State Quantum Simulator
//! This libary implements sparse state simulation, based on the design from
//! <a href="https://arxiv.org/abs/2105.01533">Leveraging state sparsity for more efficient quantum simulations</a>.

pub mod exp;
mod index_map;
mod nearly_zero;

// Additional test infrastructure is available in matrix_testing that allows comparing the transformations
// implemented here with direct matrix application to the state vector.
#[cfg(test)]
mod matrix_testing;

use crate::sparse::nearly_zero::NearlyZero;
use index_map::IndexMap;
use ndarray::{Array2, s};
use num_bigint::BigUint;
use num_complex::Complex64;
use num_traits::{One, ToPrimitive, Zero};
use rand::{Rng, SeedableRng, rngs::StdRng};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{cell::RefCell, f64::consts::FRAC_1_SQRT_2, fmt::Write};

type SparseState = Vec<(BigUint, Complex64)>;
type SparseStateMap = FxHashMap<BigUint, Complex64>;

const QUEUE_LIMIT: usize = 10_000;
const DEFAULT_INITIAL_SIZE: usize = 50;

/// The `QuantumSim` struct contains the necessary state for tracking the simulation. Each instance of a
/// `QuantumSim` represents an independant simulation.
pub struct QuantumSim {
    /// The structure that describes the current quantum state.
    pub(crate) state: SparseState,

    /// The mapping from qubit identifiers to internal state locations.
    pub(crate) id_map: IndexMap<usize, usize>,

    /// The random number generator used for probabilistic operations.
    rng: RefCell<StdRng>,

    /// The bitmap that tracks whether a given qubit has an pending H operation queued on it.
    h_flag: BigUint,

    /// The map for tracking queued Pauli-X rotations by a given angle for a given qubit.
    rx_queue: IndexMap<usize, f64>,

    /// The map for tracking queued Pauli-Y rotations by a given angle for a given qubit.
    ry_queue: IndexMap<usize, f64>,

    /// The list of queued gate operations.
    op_queue: Vec<(Vec<usize>, usize, OpCode)>,
}

/// Operations that support generic queuing.
#[derive(Debug, Copy, Clone)]
pub(crate) enum OpCode {
    X,
    Y,
    Z,
    S,
    Sadj,
    T,
    Tadj,
    Rz(f64),
}

/// Levels for flushing of queued gates.
#[derive(Debug, Copy, Clone)]
pub(crate) enum FlushLevel {
    H,
    HRx,
    HRxRy,
}

impl Default for QuantumSim {
    fn default() -> Self {
        Self::new(None)
    }
}

/// Provides the common set of functionality across all quantum simulation types.
impl QuantumSim {
    /// Creates a new sparse state quantum simulator object with empty initial state (no qubits allocated, no operations buffered).
    #[must_use]
    pub fn new(rng: Option<StdRng>) -> Self {
        let initial_state = vec![(BigUint::zero(), Complex64::one())];

        QuantumSim {
            state: initial_state,
            id_map: IndexMap::with_capacity(DEFAULT_INITIAL_SIZE),
            rng: RefCell::new(rng.unwrap_or_else(StdRng::from_entropy)),
            h_flag: BigUint::zero(),
            rx_queue: IndexMap::with_capacity(DEFAULT_INITIAL_SIZE),
            ry_queue: IndexMap::with_capacity(DEFAULT_INITIAL_SIZE),
            op_queue: Vec::with_capacity(DEFAULT_INITIAL_SIZE),
        }
    }

    /// Sets the seed for the random number generator used for probabilistic operations.
    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng.replace(StdRng::seed_from_u64(seed));
    }

    pub fn take_rng(&mut self) -> StdRng {
        self.rng.replace(StdRng::from_entropy())
    }

    /// Returns a sorted copy of the current sparse state as a vector of pairs of indices and complex numbers, along with
    /// the total number of currently allocated qubits to help in interpreting the sparse state.
    #[allow(clippy::missing_panics_doc)] // reason="Panics can only occur if the keys are not present in the map, which should not happen."
    #[must_use]
    pub fn get_state(&mut self) -> (Vec<(BigUint, Complex64)>, usize) {
        // Swap all the entries in the state to be ordered by qubit identifier. This makes
        // interpreting the state easier for external consumers that don't have access to the id map.
        let sorted_keys: Vec<usize> = self.id_map.iter().map(|(k, _)| k).collect();
        self.flush_queue(&sorted_keys, FlushLevel::HRxRy);

        sorted_keys.iter().enumerate().for_each(|(index, &key)| {
            if index != self.id_map[key] {
                self.swap_qubit_state(self.id_map[key], index);
                if let Some((swapped_key, _)) =
                    self.id_map.iter().find(|&(_, &value)| value == index)
                {
                    *(self
                        .id_map
                        .get_mut(swapped_key)
                        .expect("key should be present in map")) = self.id_map[key];
                }
                *(self
                    .id_map
                    .get_mut(key)
                    .expect("key should be present in map")) = index;
            }
        });

        let mut state = self.state.clone();
        state.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        (state, sorted_keys.len())
    }

    /// Allocates a fresh qubit, returning its identifier. Note that this will use the lowest available
    /// identifier, and may result in qubits being allocated "in the middle" of an existing register
    /// if those identifiers are available.
    #[must_use]
    pub fn allocate(&mut self) -> usize {
        // Add the new entry into the FxHashMap at the first available sequential ID and first available
        // sequential location.
        let sorted_keys: Vec<usize> = self.id_map.iter().map(|(k, _)| k).collect();
        let mut sorted_vals: Vec<&usize> = self.id_map.values().collect();
        sorted_vals.sort_unstable();
        let new_key = sorted_keys
            .iter()
            .enumerate()
            .take_while(|(index, key)| index == *key)
            .last()
            .map_or(0_usize, |(_, &key)| key + 1);
        let new_val = sorted_vals
            .iter()
            .enumerate()
            .take_while(|(index, val)| index == **val)
            .last()
            .map_or(0_usize, |(_, &&val)| val + 1);
        self.id_map.insert(new_key, new_val);

        // Return the new ID that was used.
        new_key
    }

    /// Releases the given qubit, collapsing its state in the process. After release that identifier is
    /// no longer valid for use in other functions and will cause an error if used.
    /// # Panics
    ///
    /// The function will panic if the given id does not correpsond to an allocated qubit.
    pub fn release(&mut self, id: usize) {
        if self.id_map.iter().count() == 1 {
            // This is a release of the last qubit.
            // When no qubits are allocated, we can reset the sparse state to a clean ground, so
            // any accumulated phase dissappears.
            // There is no need to apply any pending operations, as we are about throw away the state
            // anyway.
            self.op_queue = Vec::with_capacity(DEFAULT_INITIAL_SIZE);
            self.h_flag = BigUint::zero();
            self.rx_queue = IndexMap::with_capacity(DEFAULT_INITIAL_SIZE);
            self.ry_queue = IndexMap::with_capacity(DEFAULT_INITIAL_SIZE);
            self.state = vec![(BigUint::zero(), Complex64::one())];
        } else {
            // Measure and collapse the state for this qubit. This will also apply any queued operations.
            let res = self.measure(id);
            let loc = self.id_map[id];

            // If the result of measurement was true then we must set the bit for this qubit in every key
            // to zero to "reset" the qubit.
            if res {
                self.state.iter_mut().for_each(|(k, _)| {
                    if k.bit(loc as u64) {
                        k.set_bit(loc as u64, false);
                    }
                });
            }
        }

        // Remove the qubit from the ID map now that any operations on it are complete.
        self.id_map.remove(id);
    }

    /// Prints the current state vector to standard output with integer labels for the states, skipping any
    /// states with zero amplitude.
    #[allow(clippy::missing_panics_doc)] // reason="Panics can only occur if the keys are not present in the map, which should not happen."
    #[must_use]
    pub fn dump(&mut self) -> String {
        // Swap all the entries in the state to be ordered by qubit identifier. This makes
        // interpreting the state easier for external consumers that don't have access to the id map.
        let mut sorted_keys: Vec<usize> = self.id_map.iter().map(|(k, _)| k).collect();
        self.flush_queue(&sorted_keys, FlushLevel::HRxRy);

        sorted_keys.sort_unstable();
        sorted_keys.iter().enumerate().for_each(|(index, &key)| {
            if index != self.id_map[key] {
                self.swap_qubit_state(self.id_map[key], index);
                if let Some((swapped_key, _)) =
                    self.id_map.iter().find(|&(_, &value)| value == index)
                {
                    *(self
                        .id_map
                        .get_mut(swapped_key)
                        .expect("key should be present in map")) = self.id_map[key];
                }
                *(self
                    .id_map
                    .get_mut(key)
                    .expect("key should be present in map")) = index;
            }
        });

        self.dump_impl(false)
    }

    /// Utility function that performs the actual output of state (and optionally map) to screen. Can
    /// be called internally from other functions to aid in debugging and does not perform any modification
    /// of the internal structures.
    fn dump_impl(&mut self, print_id_map: bool) -> String {
        #[cfg(windows)]
        const LINE_ENDING: &[u8] = b"\r\n";
        #[cfg(not(windows))]
        const LINE_ENDING: &[u8] = b"\n";

        let mut output = String::new();
        let nl = String::from_utf8(LINE_ENDING.to_vec()).expect("Failed to create newline string");
        if print_id_map {
            output
                .write_str(&format!("MAP: {:?}", self.id_map))
                .expect("Failed to write output");
            output.write_str(&nl).expect("Failed to write output");
        }
        output
            .write_str("STATE: [ ")
            .expect("Failed to write output");

        self.state.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        for (key, value) in &self.state {
            output
                .write_str(&format!("|{key}\u{27e9}: {value}, "))
                .expect("Failed to write output");
        }
        output.write_str("]").expect("Failed to write output");
        output.write_str(&nl).expect("Failed to write output");
        output
    }

    /// Checks the probability of parity measurement in the computational basis for the given set of
    /// qubits.
    /// # Panics
    ///
    /// This function will panic if the given ids do not all correspond to allocated qubits.
    /// This function will panic if there are duplicate ids in the given list.
    #[must_use]
    pub fn joint_probability(&mut self, ids: &[usize]) -> f64 {
        // Flush the queue only if there are pending H, Rx, or Ry operations on this qubit.
        // Other queued operations will be applied by `check_joint_probability` below.
        self.maybe_flush_queue(ids, FlushLevel::HRxRy);

        Self::check_for_duplicates(ids);
        let locs: Vec<usize> = ids
            .iter()
            .map(|id| {
                *self
                    .id_map
                    .get(*id)
                    .unwrap_or_else(|| panic!("Unable to find qubit with id {id}"))
            })
            .collect();

        self.check_joint_probability(&locs)
    }

    /// Checks the internal state of the given qubit and returns true only if the given qubit is in exactly the |0⟩ state.
    pub fn qubit_is_zero(&mut self, id: usize) -> bool {
        self.joint_probability(&[id]).is_nearly_zero()
    }

    /// Measures the qubit with the given id, collapsing the state based on the measured result.
    /// # Panics
    ///
    /// This funciton will panic if the given identifier does not correspond to an allocated qubit.
    #[must_use]
    pub fn measure(&mut self, id: usize) -> bool {
        // We only need to flush the queue here if there are pending H, Rx, or Ry operations.
        // Any operations in `self.op_queue` will get applied when `check_joint_probability`
        // iterates through the state vector.
        self.maybe_flush_queue(&[id], FlushLevel::HRxRy);

        let loc = *self
            .id_map
            .get(id)
            .unwrap_or_else(|| panic!("Unable to find qubit with id {id}"));
        let random_sample = self.rng.borrow_mut().r#gen::<f64>();
        let prob = self.check_joint_probability(&[loc]);
        let res = random_sample < prob;
        self.collapse(loc, res, prob);
        res
    }

    /// Performs a joint measurement to get the parity of the given qubits, collapsing the state
    /// based on the measured result.
    /// # Panics
    ///
    /// This function will panic if any of the given identifiers do not correspond to an allocated qubit.
    /// This function will panic if any of the given identifiers are duplicates.
    #[must_use]
    pub fn joint_measure(&mut self, ids: &[usize]) -> bool {
        // Flush the queue only if there are pending H, Rx, or Ry operations on this qubit.
        // Other queued operations will be applied by `check_joint_probability` below.
        self.maybe_flush_queue(ids, FlushLevel::HRxRy);

        Self::check_for_duplicates(ids);
        let locs: Vec<usize> = ids
            .iter()
            .map(|id| {
                *self
                    .id_map
                    .get(*id)
                    .unwrap_or_else(|| panic!("Unable to find qubit with id {id}"))
            })
            .collect();

        let random_sample = self.rng.borrow_mut().r#gen::<f64>();
        let prob = self.check_joint_probability(&locs);
        let res = random_sample < prob;
        self.joint_collapse(&locs, res, prob);
        res
    }

    /// Utility to get the sum of all probabilies where an odd number of the bits at the given locations
    /// are set. This corresponds to the probability of jointly measuring those qubits in the computational
    /// basis.
    fn check_joint_probability(&mut self, locs: &[usize]) -> f64 {
        let mask = locs.iter().fold(BigUint::zero(), |accum, loc| {
            accum | (BigUint::one() << loc)
        });

        let ops = self.take_ops();

        self.state.iter_mut().fold(0.0_f64, |accum, (index, val)| {
            apply_ops(&ops, index, val);
            if (&*index & &mask).count_ones() & 1 > 0 {
                accum + val.norm_sqr()
            } else {
                accum
            }
        })
    }

    /// Takes ownership of the queued operations and returns them, clearing the `op_queue`.
    /// This also resolves and checks the qubits for each operation, mapping them to their locations.
    fn take_ops(&mut self) -> Vec<(Vec<u64>, u64, OpCode)> {
        let ops = self
            .op_queue
            .iter()
            .map(|(ctls, target, op)| {
                let (target, ctls) = self.resolve_and_check_qubits(*target, ctls);
                (ctls, target, *op)
            })
            .collect::<Vec<_>>();
        self.op_queue.clear();
        ops
    }

    /// Utility to collapse the probability at the given location based on the boolean value. This means
    /// that if the given value is 'true' then all keys in the sparse state where the given location
    /// has a zero bit will be reduced to zero and removed. Then the sparse state is normalized.
    fn collapse(&mut self, loc: usize, val: bool, scaling_denominator: f64) {
        self.joint_collapse(&[loc], val, scaling_denominator);
    }

    /// Utility to collapse the joint probability of a particular set of locations in the sparse state.
    /// The entries that do not correspond to the given boolean value are removed, and then the whole
    /// state is normalized.
    fn joint_collapse(&mut self, locs: &[usize], val: bool, scaling_denominator: f64) {
        let mask = locs.iter().fold(BigUint::zero(), |accum, loc| {
            accum | (BigUint::one() << loc)
        });

        // Normalize the new state using the accumulated scaling.
        let scaling = 1.0
            / (if val {
                scaling_denominator
            } else {
                1.0 - scaling_denominator
            })
            .sqrt();
        self.state = self
            .state
            .drain(..)
            .filter_map(|(k, v)| {
                if (&k & &mask).count_ones() & 1 == u64::from(val) {
                    Some((k, v * scaling))
                } else {
                    None
                }
            })
            .collect();
    }

    /// Swaps the mapped ids for the given qubits.
    /// # Panics
    /// This function will panic if either of the given identifiers do not correspond to an allocated qubit.
    pub fn swap_qubit_ids(&mut self, qubit1: usize, qubit2: usize) {
        self.flush_ops();

        // Must also swap any queued operations.
        let (h_val1, h_val2) = (
            self.h_flag.bit(qubit1 as u64),
            self.h_flag.bit(qubit2 as u64),
        );
        self.h_flag.set_bit(qubit1 as u64, h_val2);
        self.h_flag.set_bit(qubit2 as u64, h_val1);

        let x_angle1 = self.rx_queue.get(qubit1).copied();
        let x_angle2 = self.rx_queue.get(qubit2).copied();
        if let Some(angle) = x_angle1 {
            self.rx_queue.insert(qubit2, angle);
        } else {
            self.rx_queue.remove(qubit2);
        }
        if let Some(angle) = x_angle2 {
            self.rx_queue.insert(qubit1, angle);
        } else {
            self.rx_queue.remove(qubit1);
        }

        let y_angle1 = self.ry_queue.get(qubit1).copied();
        let y_angle2 = self.ry_queue.get(qubit2).copied();
        if let Some(ry_val) = y_angle1 {
            self.ry_queue.insert(qubit2, ry_val);
        } else {
            self.ry_queue.remove(qubit2);
        }
        if let Some(ry_val) = y_angle2 {
            self.ry_queue.insert(qubit1, ry_val);
        } else {
            self.ry_queue.remove(qubit1);
        }

        let qubit1_mapped = *self
            .id_map
            .get(qubit1)
            .unwrap_or_else(|| panic!("Unable to find qubit with id {qubit1}"));
        let qubit2_mapped = *self
            .id_map
            .get(qubit2)
            .unwrap_or_else(|| panic!("Unable to find qubit with id {qubit2}"));
        *self.id_map.get_mut(qubit1).unwrap() = qubit2_mapped;
        *self.id_map.get_mut(qubit2).unwrap() = qubit1_mapped;
    }

    /// Swaps the states of two qubits throughout the sparse state map.
    pub(crate) fn swap_qubit_state(&mut self, qubit1: usize, qubit2: usize) {
        if qubit1 == qubit2 {
            return;
        }

        self.flush_queue(&[qubit1, qubit2], FlushLevel::HRxRy);

        let (q1, q2) = (qubit1 as u64, qubit2 as u64);

        // Swap entries in the sparse state to correspond to swapping of two qubits' locations.
        self.state.iter_mut().for_each(|(k, _)| {
            if k.bit(q1) != k.bit(q2) {
                let mut new_k = k.clone();
                new_k.set_bit(q1, !k.bit(q1));
                new_k.set_bit(q2, !k.bit(q2));
                *k = new_k;
            }
        });
    }

    pub(crate) fn check_for_duplicates(ids: &[usize]) {
        let mut unique = FxHashSet::default();
        for id in ids {
            assert!(
                unique.insert(id),
                "Duplicate qubit id '{id}' found in application."
            );
        }
    }

    /// Verifies that the given target and list of controls does not contain any duplicate entries, and returns
    /// those values mapped to internal identifiers and converted to `u64`.
    fn resolve_and_check_qubits(&self, target: usize, ctls: &[usize]) -> (u64, Vec<u64>) {
        let mut ids = ctls.to_owned();
        ids.push(target);
        Self::check_for_duplicates(&ids);

        let target = *self
            .id_map
            .get(target)
            .unwrap_or_else(|| panic!("Unable to find qubit with id {target}"))
            as u64;

        let ctls: Vec<u64> = ctls
            .iter()
            .map(|c| {
                *self
                    .id_map
                    .get(*c)
                    .unwrap_or_else(|| panic!("Unable to find qubit with id {c}"))
                    as u64
            })
            .collect();

        (target, ctls)
    }

    fn enqueue_op(&mut self, target: usize, ctls: Vec<usize>, op: OpCode) {
        if self.op_queue.len() == QUEUE_LIMIT {
            self.flush_ops();
        }
        self.op_queue.push((ctls, target, op));
    }

    fn has_queued_hrxy(&self, target: usize) -> bool {
        self.h_flag.bit(target as u64)
            || self.rx_queue.contains_key(target)
            || self.ry_queue.contains_key(target)
    }

    fn maybe_flush_queue(&mut self, qubits: &[usize], level: FlushLevel) {
        if qubits.iter().any(|q| self.has_queued_hrxy(*q)) {
            self.flush_queue(qubits, level);
        }
    }

    pub(crate) fn flush_queue(&mut self, qubits: &[usize], level: FlushLevel) {
        for target in qubits {
            if self.h_flag.bit(*target as u64) {
                self.apply_mch(&[], *target);
                self.h_flag.set_bit(*target as u64, false);
            }
            match level {
                FlushLevel::H => (),
                FlushLevel::HRx => self.flush_rx(*target),
                FlushLevel::HRxRy => {
                    self.flush_rx(*target);
                    self.flush_ry(*target);
                }
            }
        }
        // Always call flush ops afterward to make sure no pending operations remain. If any of the above
        // already applied operations, this will be a no-op since the queue will be empty.
        self.flush_ops();
    }

    fn flush_ops(&mut self) {
        if !self.op_queue.is_empty() {
            let ops = self.take_ops();
            self.state.iter_mut().for_each(|(index, value)| {
                apply_ops(&ops, index, value);
            });
        }
    }

    fn flush_rx(&mut self, target: usize) {
        if let Some(theta) = self.rx_queue.get(target) {
            self.mcrotation(&[], *theta, target, false);
            self.rx_queue.remove(target);
        }
    }

    fn flush_ry(&mut self, target: usize) {
        if let Some(theta) = self.ry_queue.get(target) {
            self.mcrotation(&[], *theta, target, true);
            self.ry_queue.remove(target);
        }
    }

    /// Performs the Pauli-X transformation on a single state.
    fn x_transform((index, _val): (&mut BigUint, &mut Complex64), target: u64) {
        index.set_bit(target, !index.bit(target));
    }

    /// Single qubit X gate.
    pub fn x(&mut self, target: usize) {
        if let Some(entry) = self.ry_queue.get_mut(target) {
            // XY = -YX, so switch the sign on any queued Ry rotations.
            *entry *= -1.0;
        }
        if self.h_flag.bit(target as u64) {
            // XH = HZ, so execute a Z transformation if there is an H queued.
            self.enqueue_op(target, Vec::new(), OpCode::Z);
        } else {
            self.enqueue_op(target, Vec::new(), OpCode::X);
        }
    }

    /// Multi-controlled X gate.
    pub fn mcx(&mut self, ctls: &[usize], target: usize) {
        if ctls.is_empty() {
            self.x(target);
            return;
        }

        if self.ry_queue.contains_key(target) {
            self.flush_queue(&[target], FlushLevel::HRxRy);
        }

        if ctls.len() > 1 {
            self.maybe_flush_queue(ctls, FlushLevel::HRxRy);
        } else if self.ry_queue.contains_key(ctls[0])
            || self.rx_queue.contains_key(ctls[0])
            || (self.h_flag.bit(ctls[0] as u64) && !self.h_flag.bit(target as u64))
        {
            self.flush_queue(ctls, FlushLevel::HRxRy);
        }

        if self.h_flag.bit(target as u64) {
            if ctls.len() == 1 && self.h_flag.bit(ctls[0] as u64) {
                // An H on both target and single control means we can perform a CNOT with the control
                // and target switched.
                self.enqueue_op(ctls[0], vec![target], OpCode::X);
            } else {
                // XH = HZ, so perform a mulit-controlled Z here.
                self.enqueue_op(target, ctls.into(), OpCode::Z);
            }
        } else {
            self.enqueue_op(target, ctls.into(), OpCode::X);
        }
    }

    /// Performs the Pauli-Y transformation on a single state.
    fn y_transform((index, val): (&mut BigUint, &mut Complex64), target: u64) {
        index.set_bit(target, !index.bit(target));
        *val *= if index.bit(target) {
            Complex64::i()
        } else {
            -Complex64::i()
        };
    }

    /// Single qubit Y gate.
    pub fn y(&mut self, target: usize) {
        if let Some(entry) = self.rx_queue.get_mut(target) {
            // XY = -YX, so flip the sign on any queued Rx rotation.
            *entry *= -1.0;
        }

        self.enqueue_op(target, Vec::new(), OpCode::Y);
    }

    /// Multi-controlled Y gate.
    #[allow(clippy::missing_panics_doc)] // reason="Panics can only occur if ctrls are empty, which is handled at the top of the function."
    pub fn mcy(&mut self, ctls: &[usize], target: usize) {
        if ctls.is_empty() {
            self.y(target);
            return;
        }

        self.maybe_flush_queue(ctls, FlushLevel::HRxRy);

        if self.rx_queue.contains_key(target) {
            self.flush_queue(&[target], FlushLevel::HRx);
        }

        if self.h_flag.bit(target as u64) {
            // HY = -YH, so add a phase to one of the controls.
            let (target, ctls) = ctls
                .split_first()
                .expect("Controls list cannot be empty here.");
            self.enqueue_op(*target, ctls.into(), OpCode::Z);
        }

        self.enqueue_op(target, ctls.into(), OpCode::Y);
    }

    /// Performs a phase transformation (a rotation in the computational basis) on a single state.
    fn phase_transform(
        phase: Complex64,
        (index, val): (&mut BigUint, &mut Complex64),
        target: u64,
    ) {
        *val *= if index.bit(target) {
            phase
        } else {
            Complex64::one()
        };
    }

    /// Multi-controlled phase rotation ("G" gate).
    pub fn mcphase(&mut self, ctls: &[usize], phase: Complex64, target: usize) {
        self.flush_queue(ctls, FlushLevel::HRxRy);
        self.flush_queue(&[target], FlushLevel::HRxRy);

        let (target, ctls) = self.resolve_and_check_qubits(target, ctls);

        self.state.iter_mut().for_each(|(index, value)| {
            if ctls.iter().all(|c| index.bit(*c)) {
                Self::phase_transform(phase, (index, value), target);
            }
        });
    }

    /// Performs the Pauli-Z transformation on a single state.
    fn z_transform((index, val): (&mut BigUint, &mut Complex64), target: u64) {
        Self::phase_transform(-Complex64::one(), (index, val), target);
    }

    /// Single qubit Z gate.
    pub fn z(&mut self, target: usize) {
        if let Some(entry) = self.ry_queue.get_mut(target) {
            // ZY = -YZ, so flip the sign on any queued Ry rotations.
            *entry *= -1.0;
        }

        if let Some(entry) = self.rx_queue.get_mut(target) {
            // ZX = -XZ, so flip the sign on any queued Rx rotations.
            *entry *= -1.0;
        }

        if self.h_flag.bit(target as u64) {
            // HZ = XH, so execute an X if an H is queued.
            self.enqueue_op(target, Vec::new(), OpCode::X);
        } else {
            self.enqueue_op(target, Vec::new(), OpCode::Z);
        }
    }

    /// Multi-controlled Z gate.
    pub fn mcz(&mut self, ctls: &[usize], target: usize) {
        if ctls.is_empty() {
            self.z(target);
            return;
        }

        // Count up the instances of queued H and Rx/Ry on controls and target, treating rotations as 2.
        let count = ctls.iter().fold(0, |accum, c| {
            accum
                + i32::from(self.h_flag.bit(*c as u64))
                + if self.rx_queue.contains_key(*c) || self.ry_queue.contains_key(*c) {
                    2
                } else {
                    0
                }
        }) + i32::from(self.h_flag.bit(target as u64))
            + if self.rx_queue.contains_key(target) || self.ry_queue.contains_key(target) {
                2
            } else {
                0
            };

        if count == 1 {
            // Only when count is exactly one can we optimize, meaning there is exactly one H on either
            // the target or one control. Create a new controls list and target where the target is whichever
            // qubit has the H queued.
            let (ctls, target): (Vec<usize>, usize) =
                if let Some(h_ctl) = ctls.iter().find(|c| self.h_flag.bit(**c as u64)) {
                    // The H is queued on one control, so create a new controls list that swaps that control for the original target.
                    (
                        ctls.iter()
                            .map(|c| if c == h_ctl { target } else { *c })
                            .collect(),
                        *h_ctl,
                    )
                } else {
                    // The H is queued on the target, so use the original values.
                    (ctls.to_owned(), target)
                };
            // With a single H queued, treat the multi-controlled Z as a multi-controlled X.
            self.enqueue_op(target, ctls, OpCode::X);
        } else {
            self.flush_queue(ctls, FlushLevel::HRxRy);
            self.flush_queue(&[target], FlushLevel::HRxRy);
            self.enqueue_op(target, ctls.into(), OpCode::Z);
        }
    }

    /// Performs the S transformation on a single state.
    fn s_transform((index, val): (&mut BigUint, &mut Complex64), target: u64) {
        Self::phase_transform(Complex64::i(), (index, val), target);
    }

    /// Single qubit S gate.
    pub fn s(&mut self, target: usize) {
        self.maybe_flush_queue(&[target], FlushLevel::HRxRy);
        self.enqueue_op(target, Vec::new(), OpCode::S);
    }

    /// Multi-controlled S gate.
    pub fn mcs(&mut self, ctls: &[usize], target: usize) {
        self.maybe_flush_queue(ctls, FlushLevel::HRxRy);
        self.maybe_flush_queue(&[target], FlushLevel::HRxRy);
        self.enqueue_op(target, ctls.into(), OpCode::S);
    }

    /// Performs the adjoint S transformation on a single state.
    fn sadj_transform((index, val): (&mut BigUint, &mut Complex64), target: u64) {
        Self::phase_transform(-Complex64::i(), (index, val), target);
    }

    /// Single qubit Adjoint S Gate.
    pub fn sadj(&mut self, target: usize) {
        self.maybe_flush_queue(&[target], FlushLevel::HRxRy);
        self.enqueue_op(target, Vec::new(), OpCode::Sadj);
    }

    /// Multi-controlled Adjoint S gate.
    pub fn mcsadj(&mut self, ctls: &[usize], target: usize) {
        self.maybe_flush_queue(ctls, FlushLevel::HRxRy);
        self.maybe_flush_queue(&[target], FlushLevel::HRxRy);
        self.enqueue_op(target, ctls.into(), OpCode::Sadj);
    }

    /// Performs the T transformation on a single state.
    fn t_transform((index, val): (&mut BigUint, &mut Complex64), target: u64) {
        Self::phase_transform(
            Complex64::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
            (index, val),
            target,
        );
    }

    /// Single qubit T gate.
    pub fn t(&mut self, target: usize) {
        self.maybe_flush_queue(&[target], FlushLevel::HRxRy);
        self.enqueue_op(target, Vec::new(), OpCode::T);
    }

    /// Multi-controlled T gate.
    pub fn mct(&mut self, ctls: &[usize], target: usize) {
        self.maybe_flush_queue(ctls, FlushLevel::HRxRy);
        self.maybe_flush_queue(&[target], FlushLevel::HRxRy);
        self.enqueue_op(target, ctls.into(), OpCode::T);
    }

    /// Performs the adjoint T transformation to a single state.
    fn tadj_transform((index, val): (&mut BigUint, &mut Complex64), target: u64) {
        Self::phase_transform(
            Complex64::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
            (index, val),
            target,
        );
    }

    /// Single qubit Adjoint T gate.
    pub fn tadj(&mut self, target: usize) {
        self.maybe_flush_queue(&[target], FlushLevel::HRxRy);
        self.enqueue_op(target, Vec::new(), OpCode::Tadj);
    }

    /// Multi-controlled Adjoint T gate.
    pub fn mctadj(&mut self, ctls: &[usize], target: usize) {
        self.maybe_flush_queue(ctls, FlushLevel::HRxRy);
        self.maybe_flush_queue(&[target], FlushLevel::HRxRy);
        self.enqueue_op(target, ctls.into(), OpCode::Tadj);
    }

    /// Performs the Rz transformation with the given angle to a single state.
    fn rz_transform((index, val): (&mut BigUint, &mut Complex64), theta: f64, target: u64) {
        *val *= Complex64::exp(Complex64::new(
            0.0,
            theta / 2.0 * if index.bit(target) { 1.0 } else { -1.0 },
        ));
    }

    /// Single qubit Rz gate.
    pub fn rz(&mut self, theta: f64, target: usize) {
        self.maybe_flush_queue(&[target], FlushLevel::HRxRy);
        self.enqueue_op(target, Vec::new(), OpCode::Rz(theta));
    }

    /// Multi-controlled Rz gate.
    pub fn mcrz(&mut self, ctls: &[usize], theta: f64, target: usize) {
        self.maybe_flush_queue(ctls, FlushLevel::HRxRy);
        self.maybe_flush_queue(&[target], FlushLevel::HRxRy);
        self.enqueue_op(target, ctls.into(), OpCode::Rz(theta));
    }

    /// Single qubit H gate.
    pub fn h(&mut self, target: usize) {
        if let Some(entry) = self.ry_queue.get_mut(target) {
            // YH = -HY, so flip the sign on any queued Ry rotations.
            *entry *= -1.0;
        }

        if self.rx_queue.contains_key(target) {
            // Can't commute well with queued Rx, so flush those ops.
            self.flush_queue(&[target], FlushLevel::HRx);
        }

        self.h_flag
            .set_bit(target as u64, !self.h_flag.bit(target as u64));
    }

    /// Multi-controlled H gate.
    pub fn mch(&mut self, ctls: &[usize], target: usize) {
        self.flush_queue(ctls, FlushLevel::HRxRy);
        if self.ry_queue.contains_key(target) || self.rx_queue.contains_key(target) {
            self.flush_queue(&[target], FlushLevel::HRxRy);
        }

        self.apply_mch(ctls, target);
    }

    /// Apply the full state transformation corresponding to the multi-controlled H gate. Note that
    /// this can increase the size of the state vector by introducing new non-zero states
    /// or decrease the size by bringing some states to zero.
    fn apply_mch(&mut self, ctls: &[usize], target: usize) {
        let (target, ctls) = self.resolve_and_check_qubits(target, ctls);

        // This operation requires reading other entries in the state vector while modifying one, so convert it into a state map
        // to support lookups. Apply any pending operations in the process.
        let ops = self.take_ops();
        let mapped_state: SparseStateMap = self
            .state
            .drain(..)
            .map(|(mut index, mut val)| {
                apply_ops(&ops, &mut index, &mut val);
                (index, val)
            })
            .collect();

        let mut flipped = BigUint::zero();
        flipped.set_bit(target, true);

        self.state.extend(mapped_state.iter().fold(
            SparseState::default(),
            |mut accum, (index, value)| {
                if ctls.iter().all(|c| index.bit(*c)) {
                    let flipped_index = index ^ &flipped;
                    if !mapped_state.contains_key(&flipped_index) {
                        // The state vector does not have an entry for the state where the target is flipped
                        // and all other qubits are the same, meaning there is no superposition for this state.
                        // Create the additional state caluclating the resulting superposition.
                        let mut zero_bit_index = index.clone();
                        zero_bit_index.set_bit(target, false);
                        accum.push((zero_bit_index, value * std::f64::consts::FRAC_1_SQRT_2));

                        let mut one_bit_index = index.clone();
                        one_bit_index.set_bit(target, true);
                        accum.push((
                            one_bit_index,
                            value
                                * std::f64::consts::FRAC_1_SQRT_2
                                * (if index.bit(target) { -1.0 } else { 1.0 }),
                        ));
                    } else if !index.bit(target) {
                        // The state vector already has a superposition for this state, so calculate the resulting
                        // updates using the value from the flipped state. Note we only want to perform this for one
                        // of the states to avoid duplication, so we pick the Zero state by checking the target bit
                        // in the index is not set.
                        let flipped_value = &mapped_state[&flipped_index];

                        let new_val = (value + flipped_value) as Complex64;
                        if !new_val.is_nearly_zero() {
                            accum.push((index.clone(), new_val * std::f64::consts::FRAC_1_SQRT_2));
                        }

                        let new_val = (value - flipped_value) as Complex64;
                        if !new_val.is_nearly_zero() {
                            accum.push((
                                index | &flipped,
                                new_val * std::f64::consts::FRAC_1_SQRT_2,
                            ));
                        }
                    }
                } else {
                    accum.push((index.clone(), *value));
                }
                accum
            },
        ));
    }

    /// Performs a rotation in the non-computational basis, which cannot be done in-place. This
    /// corresponds to an Rx or Ry depending on the requested sign flip, and notably can increase or
    /// decrease the size of the state vector.
    fn mcrotation(&mut self, ctls: &[usize], theta: f64, target: usize, sign_flip: bool) {
        // Calculate the matrix entries for the rotation by the given angle, respecting the sign flip.
        let m00 = Complex64::new(f64::cos(theta / 2.0), 0.0);
        let m01 = Complex64::new(0.0, f64::sin(theta / -2.0))
            * if sign_flip {
                -Complex64::i()
            } else {
                Complex64::one()
            };

        if m00.is_nearly_zero() {
            // This is just a Pauli rotation.
            if sign_flip {
                self.mcy(ctls, target);
            } else {
                self.mcx(ctls, target);
            }
            // Rx/Ry are different from X/Y by a global phase of -i, so apply that here when indicated by m01,
            // for mathematical correctness.
            let (_, ctls) = self.resolve_and_check_qubits(target, ctls);
            let factor = m01
                * if sign_flip {
                    Complex64::i()
                } else {
                    Complex64::one()
                };
            if factor != Complex64::one() {
                let ops = self.take_ops();
                self.state.iter_mut().for_each(|(index, value)| {
                    apply_ops(&ops, index, value);
                    if ctls.iter().all(|c| index.bit(*c)) {
                        *value *= factor;
                    }
                });
            }
        } else if m01.is_nearly_zero() {
            // This is just identity, so we can effectively no-op, and just add a phase of -1 as indicated by m00.
            // Here, m00 + 1 == 0 is used to check if m00 == -1.
            if (m00 + Complex64::one()).is_nearly_zero() {
                let ops = self.take_ops();
                let (_, ctls) = self.resolve_and_check_qubits(target, ctls);
                self.state.iter_mut().for_each(|(index, value)| {
                    apply_ops(&ops, index, value);
                    if ctls.iter().all(|c| index.bit(*c)) {
                        *value *= -Complex64::one();
                    }
                });
            }
        } else {
            let (target, ctls) = self.resolve_and_check_qubits(target, ctls);
            let m10 = m01 * if sign_flip { -1.0 } else { 1.0 };
            let mut flipped = BigUint::zero();
            flipped.set_bit(target, true);

            // This operation requires reading other entries in the state vector while modifying one, so convert it into a state map
            // to support lookups. Apply any pending operations in the process.
            let ops = self.take_ops();
            let mapped_state: SparseStateMap = self
                .state
                .drain(..)
                .map(|(mut index, mut val)| {
                    apply_ops(&ops, &mut index, &mut val);
                    (index, val)
                })
                .collect();

            self.state.extend(mapped_state.iter().fold(
                SparseState::default(),
                |mut accum, (index, value)| {
                    if ctls.iter().all(|c| index.bit(*c)) {
                        let flipped_index = index ^ &flipped;
                        if !mapped_state.contains_key(&flipped_index) {
                            // The state vector doesn't have an entry for the flipped target bit, so there
                            // isn't a superposition. Calculate the superposition using the matrix entries.
                            if index.bit(target) {
                                accum.push((flipped_index, value * m01));
                                accum.push((index.clone(), value * m00));
                            } else {
                                accum.push((index.clone(), value * m00));
                                accum.push((flipped_index, value * m10));
                            }
                        } else if !index.bit(target) {
                            // There is already a superposition of the target for this state, so calculate the new
                            // entries using the values from the flipped state. Note we only want to do this for one of
                            // the states, so we pick the Zero state by checking the target bit in the index is not set.
                            let flipped_val = mapped_state[&flipped_index];

                            let new_val = (value * m00 + flipped_val * m01) as Complex64;
                            if !new_val.is_nearly_zero() {
                                accum.push((index.clone(), new_val));
                            }

                            let new_val = (value * m10 + flipped_val * m00) as Complex64;
                            if !new_val.is_nearly_zero() {
                                accum.push((flipped_index, new_val));
                            }
                        }
                    } else {
                        accum.push((index.clone(), *value));
                    }
                    accum
                },
            ));
        }
    }

    /// Single qubit Rx gate.
    pub fn rx(&mut self, theta: f64, target: usize) {
        if self.h_flag.bit(target as u64) || self.ry_queue.contains_key(target) {
            self.flush_queue(&[target], FlushLevel::HRxRy);
        }
        if let Some(entry) = self.rx_queue.get_mut(target) {
            *entry += theta;
            if entry.is_nearly_zero() {
                self.rx_queue.remove(target);
            }
        } else {
            self.rx_queue.insert(target, theta);
        }
    }

    /// Multi-controlled Rx gate.
    pub fn mcrx(&mut self, ctls: &[usize], theta: f64, target: usize) {
        self.flush_queue(ctls, FlushLevel::HRxRy);

        if self.ry_queue.contains_key(target) {
            self.flush_queue(&[target], FlushLevel::HRxRy);
        } else if self.h_flag.bit(target as u64) {
            self.flush_queue(&[target], FlushLevel::H);
        }

        self.mcrotation(ctls, theta, target, false);
    }

    /// Single qubit Ry gate.
    pub fn ry(&mut self, theta: f64, target: usize) {
        if let Some(entry) = self.ry_queue.get_mut(target) {
            *entry += theta;
            if entry.is_nearly_zero() {
                self.ry_queue.remove(target);
            }
        } else {
            self.ry_queue.insert(target, theta);
        }
    }

    /// Multi-controlled Ry gate.
    pub fn mcry(&mut self, ctls: &[usize], theta: f64, target: usize) {
        self.flush_queue(ctls, FlushLevel::HRxRy);

        if self.rx_queue.contains_key(target) {
            self.flush_queue(&[target], FlushLevel::HRx);
        } else if self.h_flag.bit(target as u64) {
            self.flush_queue(&[target], FlushLevel::H);
        }

        self.mcrotation(ctls, theta, target, true);
    }

    /// Applies the given unitary to the given targets, extending the unitary to accomodate controls if any.
    /// # Panics
    ///
    /// This function will panic if given ids in either targets or optional controls that do not correspond to allocated
    /// qubits, or if there is a duplicate id in targets or controls.
    /// This funciton will panic if the given unitary matrix does not match the number of targets provided.
    /// This function will panic if the given unitary is not square.
    /// This function will panic if the total number of targets and controls too large for a `u32`.
    pub fn apply(
        &mut self,
        unitary: &Array2<Complex64>,
        targets: &[usize],
        controls: Option<&[usize]>,
    ) {
        let mut targets = targets.to_vec();
        let mut unitary = unitary.clone();

        assert!(
            unitary.ncols() == unitary.nrows(),
            "Application given non-square matrix."
        );

        assert!(
            targets.len() == unitary.ncols() / 2,
            "Application given incorrect number of targets; expected {}, given {}.",
            unitary.ncols() / 2,
            targets.len()
        );

        if let Some(ctrls) = controls {
            // Add controls in order as targets.
            ctrls
                .iter()
                .enumerate()
                .for_each(|(index, &element)| targets.insert(index, element));

            // Extend the provided unitary by inserting it into an identity matrix.
            unitary = controlled(&unitary, ctrls.len().try_into().unwrap());
        }
        Self::check_for_duplicates(&targets);

        self.flush_queue(&targets, FlushLevel::HRxRy);

        targets
            .iter()
            .rev()
            .enumerate()
            .for_each(|(target_loc, target)| {
                let loc = *self
                    .id_map
                    .get(*target)
                    .unwrap_or_else(|| panic!("Unable to find qubit with id {target}"));
                let swap_id = self
                    .id_map
                    .iter()
                    .find(|&(_, &value)| value == target_loc)
                    .unwrap()
                    .0;
                self.swap_qubit_state(loc, target_loc);
                *(self.id_map.get_mut(swap_id).unwrap()) = loc;
                *(self.id_map.get_mut(*target).unwrap()) = target_loc;
            });

        let op_size = unitary.nrows();
        // Applying the unitary to the state vector requires looking up other entries in the state,
        // so convert into a hash map while iterating through the state vector. We drain that map
        // at the end to convert it back into a vector.
        self.state = self
            .state
            .drain(..)
            .fold(SparseStateMap::default(), |mut accum, (index, val)| {
                let i = &index / op_size;
                let l = (&index % op_size)
                    .to_usize()
                    .expect("Cannot operate on more than 64 qubits at a time.");
                for j in (0..op_size).filter(|j| !unitary.row(*j)[l].is_nearly_zero()) {
                    let loc = (&i * op_size) + j;
                    if let Some(entry) = accum.get_mut(&loc) {
                        *entry += unitary.row(j)[l] * val;
                    } else {
                        accum.insert((&i * op_size) + j, unitary.row(j)[l] * val);
                    }
                    if accum
                        .get(&loc)
                        .map_or_else(|| false, |entry| (*entry).is_nearly_zero())
                    {
                        accum.remove(&loc);
                    }
                }
                accum
            })
            .drain()
            .collect();
        assert!(
            !self.state.is_empty(),
            "State vector should never be empty."
        );
    }
}

/// Given a list of operations, applies them sequentially to the given state vector index and value in-place.
fn apply_ops(
    ops: &[(Vec<u64>, u64, OpCode)],
    index: &mut BigUint,
    amplitude: &mut num_complex::Complex<f64>,
) {
    for (ctls, target, op) in ops {
        if ctls.iter().all(|c| index.bit(*c)) {
            match op {
                OpCode::X => QuantumSim::x_transform((index, amplitude), *target),
                OpCode::Y => QuantumSim::y_transform((index, amplitude), *target),
                OpCode::Z => QuantumSim::z_transform((index, amplitude), *target),
                OpCode::S => QuantumSim::s_transform((index, amplitude), *target),
                OpCode::Sadj => QuantumSim::sadj_transform((index, amplitude), *target),
                OpCode::T => QuantumSim::t_transform((index, amplitude), *target),
                OpCode::Tadj => QuantumSim::tadj_transform((index, amplitude), *target),
                OpCode::Rz(theta) => {
                    QuantumSim::rz_transform((index, amplitude), *theta, *target);
                }
            }
        }
    }
}

/// Extends the given unitary matrix into a matrix corresponding to the same unitary with a given number of controls
/// by inserting it into an identity matrix.
#[must_use]
pub fn controlled(u: &Array2<Complex64>, num_ctrls: u32) -> Array2<Complex64> {
    let mut controlled_u = Array2::eye(u.nrows() * 2_usize.pow(num_ctrls));
    controlled_u
        .slice_mut(s![
            (controlled_u.nrows() - u.nrows())..,
            (controlled_u.ncols() - u.ncols())..
        ])
        .assign(u);
    controlled_u
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn almost_equal(a: f64, b: f64) -> bool {
        a.max(b) - b.min(a) <= 1e-10
    }

    // Test that basic allocation and release of qubits doesn't fail.
    #[test]
    fn test_alloc_release() {
        let sim = &mut QuantumSim::default();
        for i in 0..16 {
            assert_eq!(sim.allocate(), i);
        }
        sim.release(4);
        sim.release(7);
        sim.release(12);
        assert_eq!(sim.allocate(), 4);
        for i in 0..7 {
            sim.release(i);
        }
        for i in 8..12 {
            sim.release(i);
        }
        for i in 13..16 {
            sim.release(i);
        }
    }

    /// Verifies that application of gates to a qubit results in the correct probabilities.
    #[test]
    fn test_probability() {
        let mut sim = QuantumSim::default();
        let q = sim.allocate();
        let extra = sim.allocate();
        assert!(almost_equal(0.0, sim.joint_probability(&[q])));
        sim.x(q);
        assert!(almost_equal(1.0, sim.joint_probability(&[q])));
        sim.x(q);
        assert!(almost_equal(0.0, sim.joint_probability(&[q])));
        sim.h(q);
        assert!(almost_equal(0.5, sim.joint_probability(&[q])));
        sim.h(q);
        assert!(almost_equal(0.0, sim.joint_probability(&[q])));
        sim.x(q);
        sim.h(q);
        sim.s(q);
        assert!(almost_equal(0.5, sim.joint_probability(&[q])));
        sim.sadj(q);
        sim.h(q);
        sim.x(q);
        assert!(almost_equal(0.0, sim.joint_probability(&[q])));
        sim.release(extra);
        sim.release(q);
    }

    /// Verify that a qubit in superposition has probability corresponding the measured value and
    /// can be operationally reset back into the ground state.
    #[test]
    fn test_measure() {
        let mut sim = QuantumSim::default();
        let q = sim.allocate();
        let extra = sim.allocate();
        assert!(!sim.measure(q));
        sim.x(q);
        assert!(sim.measure(q));
        let mut res = false;
        while !res {
            sim.h(q);
            res = sim.measure(q);
            assert!(almost_equal(
                sim.joint_probability(&[q]),
                if res { 1.0 } else { 0.0 }
            ));
            if res {
                sim.x(q);
            }
        }
        assert!(almost_equal(sim.joint_probability(&[q]), 0.0));
        sim.release(extra);
        sim.release(q);
    }

    // Verify that out of order release of non-zero qubits behaves as expected, namely qubits that
    // are not released are still in the expected states, newly allocated qubits use the available spot
    // and start in a zero state.
    #[test]
    fn test_out_of_order_release() {
        let sim = &mut QuantumSim::default();
        for i in 0..5 {
            assert_eq!(sim.allocate(), i);
            sim.x(i);
        }

        // Release out of order.
        sim.release(3);

        // Remaining qubits should all still be in one.
        assert_eq!(sim.state.len(), 1);
        assert!(!sim.joint_probability(&[0]).is_nearly_zero());
        assert!(!sim.joint_probability(&[1]).is_nearly_zero());
        assert!(!sim.joint_probability(&[2]).is_nearly_zero());
        assert!(!sim.joint_probability(&[4]).is_nearly_zero());

        // Cheat and peak at the released location to make sure it has been zeroed out.
        assert!(sim.check_joint_probability(&[3]).is_nearly_zero());

        // Next allocation should be the empty spot, and it should be in zero state.
        assert_eq!(sim.allocate(), 3);
        assert!(sim.joint_probability(&[3]).is_nearly_zero());

        for i in 0..5 {
            sim.release(i);
        }
        assert_eq!(sim.state.len(), 1);
    }

    /// Verify joint probability works as expected, namely that it corresponds to the parity of the
    /// qubits.
    #[test]
    fn test_joint_probability() {
        let mut sim = QuantumSim::default();
        let q0 = sim.allocate();
        let q1 = sim.allocate();
        assert!(almost_equal(0.0, sim.joint_probability(&[q0, q1])));
        sim.x(q0);
        assert!(almost_equal(1.0, sim.joint_probability(&[q0, q1])));
        sim.x(q1);
        assert!(almost_equal(0.0, sim.joint_probability(&[q0, q1])));
        assert!(almost_equal(1.0, sim.joint_probability(&[q0])));
        assert!(almost_equal(1.0, sim.joint_probability(&[q1])));
        sim.h(q0);
        assert!(almost_equal(0.5, sim.joint_probability(&[q0, q1])));
        sim.release(q1);
        sim.release(q0);
    }

    /// Verify joint measurement works as expected, namely that it corresponds to the parity of the
    /// qubits.
    #[test]
    fn test_joint_measurement() {
        let mut sim = QuantumSim::default();
        let q0 = sim.allocate();
        let q1 = sim.allocate();
        assert!(!sim.joint_measure(&[q0, q1]));
        sim.x(q0);
        assert!(sim.joint_measure(&[q0, q1]));
        sim.x(q1);
        assert!(!sim.joint_measure(&[q0, q1]));
        assert!(sim.joint_measure(&[q0]));
        assert!(sim.joint_measure(&[q1]));
        sim.h(q0);
        let res = sim.joint_measure(&[q0, q1]);
        assert!(almost_equal(
            if res { 1.0 } else { 0.0 },
            sim.joint_probability(&[q0, q1])
        ));
        sim.release(q1);
        sim.release(q0);
    }

    /// Test multiple controls.
    #[test]
    fn test_multiple_controls() {
        let mut sim = QuantumSim::default();
        let q0 = sim.allocate();
        let q1 = sim.allocate();
        let q2 = sim.allocate();
        assert!(almost_equal(0.0, sim.joint_probability(&[q0])));
        sim.h(q0);
        assert!(almost_equal(0.5, sim.joint_probability(&[q0])));
        sim.h(q0);
        assert!(almost_equal(0.0, sim.joint_probability(&[q0])));
        sim.mch(&[q1], q0);
        assert!(almost_equal(0.0, sim.joint_probability(&[q0])));
        sim.x(q1);
        sim.mch(&[q1], q0);
        assert!(almost_equal(0.5, sim.joint_probability(&[q0])));
        sim.mch(&[q2, q1], q0);
        assert!(almost_equal(0.5, sim.joint_probability(&[q0])));
        sim.x(q2);
        sim.mch(&[q2, q1], q0);
        assert!(almost_equal(0.0, sim.joint_probability(&[q0])));
        sim.x(q0);
        sim.x(q1);
        sim.release(q2);
        sim.release(q1);
        sim.release(q0);
    }

    /// Verify that targets cannot be duplicated.
    #[test]
    #[should_panic(expected = "Duplicate qubit id '0' found in application.")]
    fn test_duplicate_target() {
        let mut sim = QuantumSim::new(None);
        let q = sim.allocate();
        sim.mcx(&[q], q);
        let _ = sim.dump();
    }

    /// Verify that controls cannot be duplicated.
    #[test]
    #[should_panic(expected = "Duplicate qubit id '1' found in application.")]
    fn test_duplicate_control() {
        let mut sim = QuantumSim::new(None);
        let q = sim.allocate();
        let c = sim.allocate();
        sim.mcx(&[c, c], q);
        let _ = sim.dump();
    }

    /// Verify that targets aren't in controls.
    #[test]
    #[should_panic(expected = "Duplicate qubit id '0' found in application.")]
    fn test_target_in_control() {
        let mut sim = QuantumSim::new(None);
        let q = sim.allocate();
        let c = sim.allocate();
        sim.mcx(&[c, q], q);
        let _ = sim.dump();
    }

    /// Large, entangled state handling.
    #[test]
    fn test_large_state() {
        let mut sim = QuantumSim::new(None);
        let ctl = sim.allocate();
        sim.h(ctl);
        for _ in 0..4999 {
            let q = sim.allocate();
            sim.mcx(&[ctl], q);
        }
        let _ = sim.measure(ctl);
        for i in 0..5000 {
            sim.release(i);
        }
    }

    /// Verify seeded RNG is predictable.
    #[test]
    fn test_seeded_rng() {
        let mut sim = QuantumSim::new(None);
        sim.set_rng_seed(42);
        let q = sim.allocate();
        let mut val1 = 0_u64;
        for i in 0..64 {
            sim.h(q);
            if sim.measure(q) {
                val1 += 1 << i;
            }
        }
        let mut sim = QuantumSim::new(None);
        sim.set_rng_seed(42);
        let q = sim.allocate();
        let mut val2 = 0_u64;
        for i in 0..64 {
            sim.h(q);
            if sim.measure(q) {
                val2 += 1 << i;
            }
        }
        assert_eq!(val1, val2);
    }

    /// Verify that dump after swap on released qubits doesn't crash.
    #[test]
    fn test_swap_dump() {
        let mut sim = QuantumSim::new(None);
        let q = sim.allocate();
        let inner_q = sim.allocate();
        sim.swap_qubit_ids(q, inner_q);
        sim.release(inner_q);
        println!("{}", sim.dump());
    }

    /// Verify that swap preserves queued rotations.
    #[test]
    fn test_swap_rotations() {
        let mut sim = QuantumSim::new(None);
        let (q1, q2) = (sim.allocate(), sim.allocate());
        sim.rx(PI / 7.0, q1);
        sim.ry(PI / 7.0, q2);
        sim.swap_qubit_ids(q1, q2);
        sim.rx(-PI / 7.0, q2);
        sim.ry(-PI / 7.0, q1);
        assert!(sim.joint_probability(&[q1]).is_nearly_zero());
        assert!(sim.joint_probability(&[q2]).is_nearly_zero());
    }

    /// Verify that two queued Rx rotations that sum to zero are treated as
    /// a no-op.
    #[test]
    fn test_rx_queue_nearly_zero() {
        let mut sim = QuantumSim::new(None);
        let q = sim.allocate();
        sim.rx(PI / 4.0, q);
        assert_eq!(sim.state.len(), 1);
        sim.rx(-PI / 4.0, q);
        assert_eq!(sim.state.len(), 1);
        assert!(sim.joint_probability(&[q]).is_nearly_zero());
    }

    /// Verify that two queued Ry rotations that sum to zero are treated as
    /// a no-op.
    #[test]
    fn test_ry_queue_nearly_zero() {
        let mut sim = QuantumSim::new(None);
        let q = sim.allocate();
        sim.ry(PI / 4.0, q);
        assert_eq!(sim.state.len(), 1);
        sim.ry(-PI / 4.0, q);
        assert_eq!(sim.state.len(), 1);
        assert!(sim.joint_probability(&[q]).is_nearly_zero());
    }

    /// Verifies that an Rx rotation by PI, which becomes an X gate, is correctly flushed.
    #[test]
    fn test_rx_pi_flushed() {
        let mut sim = QuantumSim::new(None);
        let q = sim.allocate();
        sim.rx(PI, q);
        assert!(almost_equal(
            sim.joint_probability(&[q]),
            sim.joint_probability(&[q])
        ));
        assert!(!sim.joint_probability(&[q]).is_nearly_zero());
    }

    /// Verifies that an Ry rotation by PI, which becomes an Y gate, is correctly flushed.
    #[test]
    fn test_ry_pi_flushed() {
        let mut sim = QuantumSim::new(None);
        let q = sim.allocate();
        sim.ry(PI, q);
        assert!(almost_equal(
            sim.joint_probability(&[q]),
            sim.joint_probability(&[q])
        ));
        assert!(!sim.joint_probability(&[q]).is_nearly_zero());
    }

    /// Verifies that when a controlled Ry(PI) is recognized as equivalent to a
    /// controlled -iY (and handed as such), the state vector is not corrupted
    #[test]
    fn test_mcry_pi() {
        let mut sim = QuantumSim::new(None);
        let q1 = sim.allocate();
        let q2 = sim.allocate();
        sim.h(q1);
        sim.x(q1);
        sim.mcry(&[q1], PI, q2);
        sim.x(q1);
        // Expected result is an equal superposition of |01⟩ and |10⟩
        assert!(almost_equal(sim.joint_probability(&[q1, q2]), 1.0));
    }

    /// Verifies that when a controlled Ry(2*PI) is recognized as equivalent to a
    /// controlled -I (and handed as such), the state vector is not corrupted
    #[test]
    fn test_mcry_2pi() {
        let mut sim = QuantumSim::new(None);
        let q1 = sim.allocate();
        let q2 = sim.allocate();
        sim.h(q1);
        sim.mcry(&[q1], 2.0 * PI, q2);
        sim.h(q1);
        // Expected result is |10⟩ because CRy(2pi) = Z ⊗ I, so conjugating
        // with Hadamards on the left makes it equivalent to a bit flip X ⊗ I
        assert!(almost_equal(sim.joint_probability(&[q1, q2]), 1.0));
    }

    /// Utility for testing operation equivalence.
    fn assert_operation_equal_referenced<F1, F2>(mut op: F1, mut reference: F2, count: usize)
    where
        F1: FnMut(&mut QuantumSim, &[usize]),
        F2: FnMut(&mut QuantumSim, &[usize]),
    {
        enum QueuedOp {
            NoOp,
            H,
            Rx,
            Ry,
        }

        for inner_op in [QueuedOp::NoOp, QueuedOp::H, QueuedOp::Rx, QueuedOp::Ry] {
            let mut sim = QuantumSim::default();

            // Allocte the control we use to verify behavior.
            let ctl = sim.allocate();
            sim.h(ctl);

            // Allocate the requested number of targets, entangling the control with them.
            let mut qs = vec![];
            for _ in 0..count {
                let q = sim.allocate();
                sim.mcx(&[ctl], q);
                qs.push(q);
            }

            // To test queuing, try the op after running each of the different intermediate operations that
            // can be queued.
            match inner_op {
                QueuedOp::NoOp => (),
                QueuedOp::H => {
                    for &q in &qs {
                        sim.h(q);
                    }
                }
                QueuedOp::Rx => {
                    for &q in &qs {
                        sim.rx(PI / 7.0, q);
                    }
                }
                QueuedOp::Ry => {
                    for &q in &qs {
                        sim.ry(PI / 7.0, q);
                    }
                }
            }

            op(&mut sim, &qs);

            // Trigger a flush between the op and expected adjoint reference to ensure the reference is
            // run without any queued, commuted operations.
            let _ = sim.joint_probability(&qs);

            reference(&mut sim, &qs);

            // Perform the adjoint of any additional ops. We check the joint probability of the target
            // qubits before and after to force a flush of the operation queue. This helps us verify queuing, as the
            // original operation will have used the queue and commuting while the adjoint perform here will not.
            let _ = sim.joint_probability(&qs);
            match inner_op {
                QueuedOp::NoOp => (),
                QueuedOp::H => {
                    for &q in &qs {
                        sim.h(q);
                    }
                }
                QueuedOp::Rx => {
                    for &q in &qs {
                        sim.rx(PI / -7.0, q);
                    }
                }
                QueuedOp::Ry => {
                    for &q in &qs {
                        sim.ry(PI / -7.0, q);
                    }
                }
            }
            let _ = sim.joint_probability(&qs);

            // Undo the entanglement.
            for q in &qs {
                sim.mcx(&[ctl], *q);
            }
            sim.h(ctl);

            // We know the operations are equal if the qubits are left in the zero state.
            assert!(sim.joint_probability(&[ctl]).is_nearly_zero());
            for q in qs {
                assert!(sim.joint_probability(&[q]).is_nearly_zero());
            }

            // Sparse state vector should have one entry for |0⟩.
            // Dump the state first to force a flush of any queued operations.
            println!("{}", sim.dump());
            assert_eq!(sim.state.len(), 1);
        }
    }

    #[test]
    fn test_h() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.h(qs[0]);
            },
            |sim, qs| {
                sim.h(qs[0]);
            },
            1,
        );
    }

    #[test]
    fn test_x() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.x(qs[0]);
            },
            |sim, qs| {
                sim.x(qs[0]);
            },
            1,
        );
    }

    #[test]
    fn test_y() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.y(qs[0]);
            },
            |sim, qs| {
                sim.y(qs[0]);
            },
            1,
        );
    }

    #[test]
    fn test_z() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.z(qs[0]);
            },
            |sim, qs| {
                sim.z(qs[0]);
            },
            1,
        );
    }

    #[test]
    fn test_s() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.s(qs[0]);
            },
            |sim, qs| {
                sim.sadj(qs[0]);
            },
            1,
        );
    }

    #[test]
    fn test_sadj() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.sadj(qs[0]);
            },
            |sim, qs| {
                sim.s(qs[0]);
            },
            1,
        );
    }

    #[test]
    fn test_cx() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.mcx(&[qs[0]], qs[1]);
            },
            |sim, qs| {
                sim.mcx(&[qs[0]], qs[1]);
            },
            2,
        );
    }

    #[test]
    fn test_cz() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.mcz(&[qs[0]], qs[1]);
            },
            |sim, qs| {
                sim.mcz(&[qs[0]], qs[1]);
            },
            2,
        );
    }

    #[test]
    fn test_swap() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.swap_qubit_ids(qs[0], qs[1]);
            },
            |sim, qs| {
                sim.swap_qubit_ids(qs[0], qs[1]);
            },
            2,
        );
    }

    #[test]
    fn test_rz() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.rz(PI / 7.0, qs[0]);
            },
            |sim, qs| {
                sim.rz(-PI / 7.0, qs[0]);
            },
            1,
        );
    }

    #[test]
    fn test_rx() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.rx(PI / 7.0, qs[0]);
            },
            |sim, qs| {
                sim.rx(-PI / 7.0, qs[0]);
            },
            1,
        );
    }

    #[test]
    fn test_ry() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.ry(PI / 7.0, qs[0]);
            },
            |sim, qs| {
                sim.ry(-PI / 7.0, qs[0]);
            },
            1,
        );
    }

    #[test]
    fn test_mcri() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.mcphase(
                    &qs[2..3],
                    Complex64::exp(Complex64::new(0.0, -(PI / 7.0) / 2.0)),
                    qs[1],
                );
            },
            |sim, qs| {
                sim.mcphase(
                    &qs[2..3],
                    Complex64::exp(Complex64::new(0.0, (PI / 7.0) / 2.0)),
                    qs[1],
                );
            },
            3,
        );
    }

    #[test]
    fn test_op_queue_flushes_at_limit() {
        let mut sim = QuantumSim::default();
        let q = sim.allocate();
        for _ in 0..10_002 {
            sim.x(q);
        }
        assert_eq!(sim.op_queue.len(), 2);
        assert_eq!(sim.state.len(), 1);
    }

    #[test]
    fn test_cx_after_h_ry_executes_queued_operations_in_order() {
        assert_operation_equal_referenced(
            |sim, qs| {
                sim.h(qs[0]);
                sim.ry(PI, qs[0]);
                sim.h(qs[1]);
                sim.mcx(&[qs[1]], qs[0]);
            },
            |sim, qs| {
                sim.mcx(&[qs[1]], qs[0]);
                sim.h(qs[1]);
                sim.ry(-PI, qs[0]);
                sim.h(qs[0]);
            },
            2,
        );
    }

    #[test]
    fn test_global_phase_dropped_when_all_qubits_released() {
        let mut sim = QuantumSim::default();
        let q = sim.allocate();
        sim.x(q);
        sim.z(q);
        sim.release(q);
        let _ = sim.allocate();
        let (state, count) = sim.get_state();
        assert_eq!(count, 1);
        assert_eq!(state.len(), 1);
        let (index, value) = state.first().unwrap();
        assert_eq!(index, &BigUint::zero());
        assert_eq!(value, &Complex64::one());
    }
}
