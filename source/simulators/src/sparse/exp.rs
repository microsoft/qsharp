// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// This file contains the native support for the multi-qubit Exp rotation gate.
// See https://learn.microsoft.com/en-us/qsharp/api/qsharp/microsoft.quantum.intrinsic.exp for details on the gate.
// This is intentionally kept separate from the main simulator implementation as it is likely to be removed
// in favor of having high level languages decompose into CNOT and single qubit rotations (see
// https://github.com/microsoft/qsharp-runtime/issues/999 and https://github.com/microsoft/QuantumLibraries/issues/579).

use num_bigint::BigUint;
use num_complex::Complex64;
use num_traits::{One, Zero};
use std::ops::ControlFlow;

use super::{FlushLevel, QuantumSim, SparseStateMap, nearly_zero::NearlyZero};

pub enum Pauli {
    I,
    X,
    Z,
    Y,
}

impl QuantumSim {
    /// Exp multi-qubit rotation gate.
    pub fn exp(&mut self, paulis: &[Pauli], theta: f64, targets: &[usize]) {
        self.mcexp(&[], paulis, theta, targets);
    }

    /// Multi-controlled Exp multi-qubit rotation gate.
    /// # Panics
    /// Panics if any of the qubit ids in `ctls` or `targets` are not allocated.
    #[allow(clippy::too_many_lines)]
    pub fn mcexp(&mut self, ctls: &[usize], paulis: &[Pauli], theta: f64, targets: &[usize]) {
        self.flush_queue(ctls, FlushLevel::HRxRy);
        self.flush_queue(targets, FlushLevel::HRxRy);

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

        let targets: Vec<u64> = targets
            .iter()
            .map(|c| {
                *self
                    .id_map
                    .get(*c)
                    .unwrap_or_else(|| panic!("Unable to find qubit with id {c}"))
                    as u64
            })
            .collect();

        let mut sorted_qubits = ctls.clone();
        sorted_qubits.append(&mut targets.clone());
        sorted_qubits.sort_unstable();
        if let ControlFlow::Break(Some(duplicate)) =
            sorted_qubits.iter().try_fold(None, |last, current| {
                last.map_or_else(
                    || ControlFlow::Continue(Some(current)),
                    |last| {
                        if last == current {
                            ControlFlow::Break(Some(current))
                        } else {
                            ControlFlow::Continue(Some(current))
                        }
                    },
                )
            })
        {
            panic!("Duplicate qubit id '{duplicate}' found in application.");
        }

        let id_coeff = Complex64::new(theta.cos(), 0.0);
        let pauli_coeff = Complex64::new(0.0, theta.sin());

        let mut xy_mask = BigUint::zero();
        let mut yz_mask = BigUint::zero();
        let mut y_count = 0_u64;
        for i in 0..paulis.len() {
            match paulis[i] {
                Pauli::I => (),
                Pauli::X => xy_mask.set_bit(targets[i], true),
                Pauli::Y => {
                    yz_mask.set_bit(targets[i], true);
                    xy_mask.set_bit(targets[i], true);
                    y_count += 1;
                }
                Pauli::Z => yz_mask.set_bit(targets[i], true),
            }
        }

        if xy_mask.is_zero() {
            // The operation is purely Pauli-Z, so we can rotate in the computational basis.
            let pauli_coeff = pauli_coeff + id_coeff;
            let id_coeff = 2.0 * id_coeff - pauli_coeff;
            if pauli_coeff.is_nearly_zero() {
                // pauli_coeff is zero, so use only the states multiplied by id_coeff.
                self.state = self
                    .state
                    .drain(..)
                    .filter_map(|(index, value)| {
                        if ctls.iter().all(|c| index.bit(*c))
                            && (&index & &yz_mask).count_ones() & 1 != 0
                        {
                            Some((index, value * id_coeff))
                        } else {
                            None
                        }
                    })
                    .collect();
            } else if id_coeff.is_nearly_zero() {
                // id_coeff is zero, so use only the states multiplied by pauli_coeff.
                self.state = self
                    .state
                    .drain(..)
                    .filter_map(|(index, value)| {
                        if ctls.iter().all(|c| index.bit(*c))
                            && (&index & &yz_mask).count_ones() & 1 != 0
                        {
                            Some((index, value * pauli_coeff))
                        } else {
                            None
                        }
                    })
                    .collect();
            } else {
                // Both coefficients are non-zero, so modify each of the state records.
                self.state.iter_mut().for_each(|(index, val)| {
                    if ctls.iter().all(|c| index.bit(*c)) {
                        *val *= if (index.clone() & &yz_mask).count_ones() & 1 == 0 {
                            pauli_coeff
                        } else {
                            id_coeff
                        };
                    }
                });
            }
        } else {
            // The operation includes some non-Pauli-Z rotations.
            let pauli_coeff = pauli_coeff
                * match y_count % 4 {
                    1 => Complex64::i(),
                    2 => -Complex64::one(),
                    3 => -Complex64::i(),
                    _ => Complex64::one(),
                };
            let pauli_coeff_alt = if y_count % 2 == 0 {
                pauli_coeff
            } else {
                -pauli_coeff
            };

            // This operation requires reading other entries in the state vector while modifying one, so convert it into a state map
            // to support lookups.
            let mapped_state: SparseStateMap = self.state.drain(..).collect();
            for (index, value) in &mapped_state {
                if ctls.iter().all(|c| index.bit(*c)) {
                    let alt_index = index ^ &xy_mask;
                    if !mapped_state.contains_key(&alt_index) {
                        self.state.push((index.clone(), value * id_coeff));
                        self.state.push((
                            alt_index,
                            value
                                * if (index & &yz_mask).count_ones() & 1 == 0 {
                                    pauli_coeff
                                } else {
                                    -pauli_coeff
                                },
                        ));
                    } else if index < &alt_index {
                        let parity = (index & &yz_mask).count_ones() & 1 != 0;
                        let alt_value = mapped_state[&alt_index] as Complex64;

                        let new_value = value * id_coeff
                            + alt_value
                                * if parity {
                                    -pauli_coeff_alt
                                } else {
                                    pauli_coeff_alt
                                };
                        if !new_value.is_nearly_zero() {
                            self.state.push((index.clone(), new_value));
                        }

                        let new_value = alt_value * id_coeff
                            + value * if parity { -pauli_coeff } else { pauli_coeff };
                        if !new_value.is_nearly_zero() {
                            self.state.push((alt_index, new_value));
                        }
                    }
                } else {
                    self.state.push((index.clone(), *value));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_exp_from_cnot() {
        let sim = &mut QuantumSim::default();
        let (control, target, paired) = (sim.allocate(), sim.allocate(), sim.allocate());

        // Entangle the check qubit `paired` with both `control` and `target`
        sim.h(paired);
        sim.mcx(&[paired], control);
        sim.mcx(&[paired], target);

        // Perform the decomposition of CNOT in terms of Rx, Rz, and Exp.
        // This decomposition is sensitive to angle convention in rotations, including
        // multipliers and sign.
        let theta = PI / -4.0;
        sim.rx(2.0 * theta, target);
        sim.rz(2.0 * theta, control);
        sim.exp(&[Pauli::Z, Pauli::X], theta, &[control, target]);

        // Perform the adjoint of CNOT, which is just CNOT again.
        sim.mcx(&[control], target);

        // Undo the entanglement.
        sim.mcx(&[paired], target);
        sim.mcx(&[paired], control);
        sim.h(paired);

        // If the rotations were performed correctly, the check qubit `paired` should
        // always be back in the ground state, and the whole state vector should be
        // back to a single zero state.
        assert!(sim.joint_probability(&[paired]).is_nearly_zero());
        assert!(sim.joint_probability(&[target]).is_nearly_zero());
        assert!(sim.joint_probability(&[control]).is_nearly_zero());
        assert_eq!(sim.state.len(), 1);
    }
}
