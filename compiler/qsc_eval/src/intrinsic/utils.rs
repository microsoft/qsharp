// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::hash_map::Entry;

use num_bigint::BigUint;
use num_complex::{Complex, Complex64};
use num_traits::{One, Zero};
use rustc_hash::FxHashMap;

pub fn split_state(
    qubits: &[usize],
    state: Vec<(BigUint, Complex64)>,
    qubit_count: usize,
) -> Result<Vec<(BigUint, Complex64)>, ()> {
    let state = state.into_iter().collect::<FxHashMap<_, _>>();
    let mut dump_state = FxHashMap::default();
    let mut other_state = FxHashMap::default();

    // Compute the mask for the qubits to dump and the mask for the other qubits.
    let (dump_mask, other_mask) = compute_mask(qubit_count, qubits);

    // Try to split out the state for the given qubits from the whole state, detecting any entanglement
    // and returning an error if the qubits are not separable.
    let dump_norm = collect_split_state(
        &state,
        &dump_mask,
        &other_mask,
        &mut dump_state,
        &mut other_state,
    )?;

    // If the product of the collected states is not equal to the total number of input states, then that
    // implies some states are zero amplitude that would have to be non-zero for the state to be separable.
    if state.len() != dump_state.len() * other_state.len() {
        return Err(());
    }

    let dump_norm = 1.0 / dump_norm.sqrt();
    Ok(dump_state
        .into_iter()
        .filter_map(|(label, val)| {
            normalize_and_reorder(val, dump_norm, qubits, &label, qubit_count)
        })
        .collect())
}

fn compute_mask(qubit_count: usize, qubits: &[usize]) -> (BigUint, BigUint) {
    let mut dump_mask = BigUint::zero();
    let mut other_mask = BigUint::zero();
    for q in 0..qubit_count {
        // Note that the qubit order is reversed to match the order of the qubits in the state.
        if qubits.contains(&q) {
            dump_mask.set_bit((qubit_count - q - 1) as u64, true);
        } else {
            other_mask.set_bit((qubit_count - q - 1) as u64, true);
        }
    }
    (dump_mask, other_mask)
}

fn collect_split_state(
    state: &FxHashMap<BigUint, Complex64>,
    dump_mask: &BigUint,
    other_mask: &BigUint,
    dump_state: &mut FxHashMap<BigUint, Complex64>,
    other_state: &mut FxHashMap<BigUint, Complex64>,
) -> Result<f64, ()> {
    let mut state_iter = state.iter();
    let (base_label, base_val) = state_iter.next().expect("state should never be empty");
    let dump_base_label = base_label & dump_mask;
    let other_base_label = base_label & other_mask;
    let mut dump_norm = 1.0_f64;

    // Start with an amplitude of 1 in the first expected split states. This becomes the basis
    // of the split later, but will get normalized as part of collecting the remaining states.
    dump_state.insert(dump_base_label.clone(), Complex64::one());
    other_state.insert(other_base_label.clone(), Complex64::one());

    for (curr_label, curr_val) in state_iter {
        let dump_label = curr_label & dump_mask;
        let other_label = curr_label & other_mask;

        // If either the state identified by the dump mask or the state identified by the other mask
        // is None, that means it has zero amplitude and we can conclude the state is not separable.
        let Some(dump_val) = state.get(&(&dump_label | &other_base_label)) else {
            return Err(());
        };
        let Some(other_val) = state.get(&(&dump_base_label | &other_label)) else {
            return Err(());
        };

        if !(dump_val * other_val - base_val * curr_val)
            .norm()
            .is_nearly_zero()
        {
            // Coefficients are not equal, so the state is not separable.
            return Err(());
        }

        if let Entry::Vacant(entry) = dump_state.entry(dump_label) {
            // When capturing the amplitude for the dump state, we must divide out the amplitude for the other
            // state, and vice-versa below.
            let amplitude = curr_val / other_val;
            let norm = amplitude.norm();
            if !norm.is_nearly_zero() {
                entry.insert(amplitude);
                dump_norm += norm;
            }
        }
        if let Entry::Vacant(entry) = other_state.entry(other_label) {
            let amplitude = curr_val / dump_val;
            let norm = amplitude.norm();
            if !norm.is_nearly_zero() {
                entry.insert(amplitude);
            }
        }
    }
    Ok(dump_norm)
}

fn normalize_and_reorder(
    val: Complex64,
    dump_norm: f64,
    qubits: &[usize],
    label: &BigUint,
    qubit_count: usize,
) -> Option<(BigUint, Complex64)> {
    // Normalize the dump state by the collected factor.
    let new_val = val * dump_norm;
    // Drop any zero amplitude states.
    if new_val.is_nearly_zero() {
        None
    } else {
        // Reorder the bits in the label to match the provided qubit order.
        let mut new_label = BigUint::zero();
        for (i, q) in qubits.iter().enumerate() {
            // Note that the qubit order is reversed to match the order of the qubits in the state.
            if label.bit((qubit_count - *q - 1) as u64) {
                new_label.set_bit((qubits.len() - i - 1) as u64, true);
            }
        }

        Some((new_label, new_val))
    }
}

trait NearlyZero {
    fn is_nearly_zero(&self) -> bool;
}

impl NearlyZero for f64 {
    fn is_nearly_zero(&self) -> bool {
        self.abs() <= 1e-10
    }
}

impl<T> NearlyZero for Complex<T>
where
    T: NearlyZero,
{
    fn is_nearly_zero(&self) -> bool {
        self.re.is_nearly_zero() && self.im.is_nearly_zero()
    }
}
