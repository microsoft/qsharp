// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use num_traits::{FromPrimitive, ToPrimitive};
use std::{cmp::Ordering, fmt::Display};

use resource_estimator::estimates::ErrorCorrection;

use crate::qubit::CatQubit;

pub struct RepetitionCode {
    p_threshold: f64,
}

impl RepetitionCode {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // arXiv:2302.06639 (p. 28, eq. E1)
    #[must_use]
    fn logical_phaseflip_probability(
        &self,
        physical_qubit: &CatQubit,
        parameter: &CodeParameter,
    ) -> Option<f64> {
        // arXiv:2302.06639 (p. 29, Fig. 26)
        let prefactor = 5.6e-2;
        let exponent = (i32::from_u64(parameter.distance)? + 1) / 2;

        Some(
            prefactor
                * ((parameter.alpha_sq.powf(0.86) * physical_qubit.k1_k2) / self.p_threshold)
                    .powi(exponent),
        )
    }

    #[allow(clippy::similar_names)]
    #[must_use]
    fn logical_bitflip_probability(parameter: &CodeParameter) -> Option<f64> {
        // number of CX gates in a repetition code cycle
        let ncx = 2 * (parameter.distance - 1);

        // bitflip error probability of a CX gate (numerically estimates using
        // full process tomography), arXiv:2302.06639 (p. 26, eq. D8)
        let pcx = 0.5 * (-2.0 * parameter.alpha_sq).exp();

        Some(f64::from_u64(ncx)? * pcx)
    }
}

impl Default for RepetitionCode {
    fn default() -> Self {
        // arXiv:2302.06639 (p. 4, p. 28, Fig. 26)
        let p_threshold = 0.013;

        Self { p_threshold }
    }
}

#[derive(Clone)]
pub struct CodeParameter {
    distance: u64,
    // amplitude ɑ arXiv:2302.06639 (p. 3)
    // average number of photons ɑ²
    alpha_sq: f64,
}

impl CodeParameter {
    #[must_use]
    pub fn new(distance: u64, alpha_sq: f64) -> Self {
        Self { distance, alpha_sq }
    }
}

impl Display for CodeParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (ɑ = {})", self.distance, self.alpha_sq)
    }
}

struct CodeParameterRange {
    distance: u64,
    alpha_sq: u64,
    max_distance: u64,
    max_alpha_sq: u64,
}

impl CodeParameterRange {
    pub fn new(lower_bound: Option<&CodeParameter>, max_distance: u64, max_alpha_sq: f64) -> Self {
        let lower_bound = lower_bound.cloned().unwrap_or(CodeParameter::new(1, 1.0));

        Self {
            distance: lower_bound.distance,
            alpha_sq: lower_bound
                .alpha_sq
                .to_u64()
                .expect("alpha can be represented as u64"),
            max_distance,
            max_alpha_sq: max_alpha_sq
                .to_u64()
                .expect("max_alpha can be represented as u64"),
        }
    }
}

impl Iterator for CodeParameterRange {
    type Item = CodeParameter;

    fn next(&mut self) -> Option<Self::Item> {
        if self.distance > self.max_distance {
            None
        } else {
            let result = CodeParameter::new(
                self.distance,
                self.alpha_sq.to_f64().expect("alpha fits in f64"),
            );

            if self.alpha_sq == self.max_alpha_sq {
                self.distance += 2;
                self.alpha_sq = 1;
            } else {
                self.alpha_sq += 1;
            }

            Some(result)
        }
    }
}

impl ErrorCorrection for RepetitionCode {
    type Qubit = CatQubit;
    type Parameter = CodeParameter;

    fn code_parameter_range(
        &self,
        lower_bound: Option<&Self::Parameter>,
    ) -> impl Iterator<Item = Self::Parameter> {
        CodeParameterRange::new(lower_bound, 49, 30.0)
    }

    fn physical_qubits(&self, parameter: &Self::Parameter) -> Result<u64, String> {
        // arXiv:2302.06639 (p. 27)
        Ok(2 * parameter.distance - 1)
    }

    fn logical_qubits(&self, _parameter: &Self::Parameter) -> Result<u64, String> {
        Ok(1)
    }

    fn logical_cycle_time(
        &self,
        _qubit: &Self::Qubit,
        parameter: &Self::Parameter,
    ) -> Result<u64, String> {
        // arXiv:2302.06639 (p. 28, repetition code cycle time in d code cycles)
        Ok(500 * parameter.distance)
    }

    fn logical_error_rate(
        &self,
        qubit: &Self::Qubit,
        parameter: &Self::Parameter,
    ) -> Result<f64, String> {
        if let (Some(code_distance_f64), Some(lzp), Some(lxp)) = (
            f64::from_u64(parameter.distance),
            self.logical_phaseflip_probability(qubit, parameter),
            Self::logical_bitflip_probability(parameter),
        ) {
            // arXiv:2302.06639 (p. 4, eq. 3)
            Ok(code_distance_f64 * (lzp + lxp))
        } else {
            Err("cannot compute logical failure probability".into())
        }
    }

    fn compute_code_parameter(
        &self,
        qubit: &Self::Qubit,
        required_logical_error_rate: f64,
    ) -> Result<Self::Parameter, String> {
        self.compute_code_parameter_for_smallest_size(qubit, required_logical_error_rate)
    }

    fn code_parameter_cmp(
        &self,
        qubit: &Self::Qubit,
        p1: &Self::Parameter,
        p2: &Self::Parameter,
    ) -> std::cmp::Ordering {
        if let (
            Ok(num_qubits1),
            Ok(logical_cycle_time1),
            Ok(num_qubits2),
            Ok(logical_cycle_time2),
        ) = (
            self.physical_qubits(p1),
            self.logical_cycle_time(qubit, p1),
            self.physical_qubits(p2),
            self.logical_cycle_time(qubit, p2),
        ) {
            num_qubits1
                .cmp(&num_qubits2)
                .then(logical_cycle_time1.cmp(&logical_cycle_time2))
        } else {
            Ordering::Equal
        }
    }
}
