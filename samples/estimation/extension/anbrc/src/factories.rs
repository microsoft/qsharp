// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use num_traits::FromPrimitive;
use resource_estimator::estimates::{self, FactoryBuilder};
use std::{borrow::Cow, rc::Rc};

use crate::{code::CodeParameter, CatQubit, RepetitionCode};

#[derive(Clone, PartialEq)]
pub struct ToffoliFactory {
    code_distance: usize,
    alpha_sq: f64,
    error_probability: f64,
    steps: usize,
    acceptance_probability: f64,
}

impl ToffoliFactory {
    pub fn error_probability(&self) -> f64 {
        self.error_probability
    }

    pub fn normalized_volume(&self) -> u64 {
        use estimates::Factory;

        assert_eq!(self.num_output_states(), 1);

        self.physical_qubits() * self.duration()
    }
}

impl estimates::Factory for ToffoliFactory {
    type Parameter = CodeParameter;

    fn physical_qubits(&self) -> u64 {
        // A Toffoli factory requires 4 logical qubits, arXiv:2302.06639 (p. 27)
        let num_logical_qubits: u64 = 4;
        let horizontal_routing_qubits = num_logical_qubits.div_ceil(4) + 1;

        (num_logical_qubits + horizontal_routing_qubits) * (2 * self.code_distance as u64 - 1)
    }

    fn duration(&self) -> u64 {
        let t = 100.0; // 1/κ₂

        // the more accurate time 89.2 was taken from the Github code
        let gate_time = (89.2 * t / self.alpha_sq) / self.acceptance_probability;

        f64::from_usize(self.steps)
            .map(|steps| (gate_time * steps).round())
            .and_then(u64::from_f64)
            .expect("cannot compute runtime of factory")
    }

    fn num_output_states(&self) -> u64 {
        1
    }
    fn max_code_parameter(&self) -> Option<Cow<Self::Parameter>> {
        Some(Cow::Owned(CodeParameter::new(
            self.code_distance as u64,
            self.alpha_sq.sqrt(),
        )))
    }
}

impl Eq for ToffoliFactory {}

impl Ord for ToffoliFactory {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.normalized_volume().cmp(&other.normalized_volume())
    }
}

impl PartialOrd for ToffoliFactory {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ToffoliBuilder {
    factories: Vec<ToffoliFactory>,
    lowest_error_probability: f64,
}

impl Default for ToffoliBuilder {
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        // arXiv:2302.06639 (p. 35, Table 3)
        let factories = vec![
            ToffoliFactory {
                code_distance: 3,
                alpha_sq: 3.75,
                error_probability: 1.05e-3,
                steps: 23,
                acceptance_probability: 0.84,
            },
            ToffoliFactory {
                code_distance: 3,
                alpha_sq: 3.93,
                error_probability: 1.02e-4,
                steps: 29,
                acceptance_probability: 0.745,
            },
            ToffoliFactory {
                code_distance: 3,
                alpha_sq: 5.32,
                error_probability: 8.14e-5,
                steps: 35,
                acceptance_probability: 0.66,
            },
            ToffoliFactory {
                code_distance: 5,
                alpha_sq: 7.15,
                error_probability: 4.62e-6,
                steps: 46,
                acceptance_probability: 0.456,
            },
            ToffoliFactory {
                code_distance: 5,
                alpha_sq: 8.18,
                error_probability: 7.00e-7,
                steps: 53,
                acceptance_probability: 0.362,
            },
            ToffoliFactory {
                code_distance: 5,
                alpha_sq: 8.38,
                error_probability: 5.36e-7,
                steps: 60,
                acceptance_probability: 0.288,
            },
            ToffoliFactory {
                code_distance: 7,
                alpha_sq: 9.71,
                error_probability: 6.14e-8,
                steps: 73,
                acceptance_probability: 0.148,
            },
            ToffoliFactory {
                code_distance: 7,
                alpha_sq: 10.76,
                error_probability: 8.40e-9,
                steps: 81,
                acceptance_probability: 0.105,
            },
            ToffoliFactory {
                code_distance: 7,
                alpha_sq: 11.06,
                error_probability: 5.16e-9,
                steps: 89,
                acceptance_probability: 0.0727,
            },
            ToffoliFactory {
                code_distance: 9,
                alpha_sq: 11.64,
                error_probability: 2.28e-9,
                steps: 104,
                acceptance_probability: 0.0262,
            },
            ToffoliFactory {
                code_distance: 9,
                alpha_sq: 12.83,
                error_probability: 2.30e-10,
                steps: 113,
                acceptance_probability: 0.0154,
            },
            ToffoliFactory {
                code_distance: 9,
                alpha_sq: 13.44,
                error_probability: 7.36e-11,
                steps: 122,
                acceptance_probability: 0.00975,
            },
            ToffoliFactory {
                code_distance: 19,
                alpha_sq: 17.35,
                error_probability: 7.90e-12,
                steps: 9576,
                acceptance_probability: 1.0,
            },
            ToffoliFactory {
                code_distance: 21,
                alpha_sq: 18.94,
                error_probability: 5.40e-13,
                steps: 14112,
                acceptance_probability: 1.0,
            },
            ToffoliFactory {
                code_distance: 23,
                alpha_sq: 20.53,
                error_probability: 3.74e-14,
                steps: 21344,
                acceptance_probability: 1.0,
            },
        ];

        let lowest_error_probability = factories
            .iter()
            .map(|f| f.error_probability)
            .min_by(f64::total_cmp)
            .unwrap_or_default();

        Self {
            factories,
            lowest_error_probability,
        }
    }
}

impl FactoryBuilder<RepetitionCode> for ToffoliBuilder {
    type Factory = ToffoliFactory;

    fn find_factories(
        &self,
        _ftp: &RepetitionCode,
        _qubit: &Rc<CatQubit>,
        _magic_state_type: usize,
        output_error_rate: f64,
        _max_code_parameter: &CodeParameter,
    ) -> Vec<Cow<Self::Factory>> {
        assert!(
            output_error_rate > self.lowest_error_probability,
            "Requested error probability is too low"
        );

        let mut factories: Vec<_> = self
            .factories
            .iter()
            .filter_map(|factory| {
                (factory.error_probability <= output_error_rate).then_some(Cow::Borrowed(factory))
            })
            .collect();
        factories.sort_unstable();
        factories
    }

    fn num_magic_state_types(&self) -> usize {
        1
    }
}
