// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{fmt::Display, ops::Deref};

use num_traits::{FromPrimitive, ToPrimitive};
use resource_estimator::estimates::{FactoryPart, Overhead, PhysicalResourceEstimationResult};

use crate::{code::RepetitionCode, counter::LogicalCounts, factories::ToffoliFactory};

pub struct AliceAndBobEstimates(
    PhysicalResourceEstimationResult<RepetitionCode, ToffoliFactory, LogicalCounts>,
);

impl AliceAndBobEstimates {
    pub fn toffoli_factory_part(&self) -> Option<&FactoryPart<ToffoliFactory>> {
        self.factory_parts()[0].as_ref()
    }

    pub fn physical_qubits(&self) -> u64 {
        let additional_routing_qubits = 2
            * ((3 * self.layout_overhead().logical_qubits()
                + self.toffoli_factory_part().map_or(0, FactoryPart::copies) * 6)
                - 1);
        self.0.physical_qubits() + additional_routing_qubits
    }

    pub fn factory_fraction(&self) -> f64 {
        (self
            .physical_qubits_for_factories()
            .to_f64()
            .expect("can convert")
            / self.physical_qubits().to_f64().expect("can convert"))
            * 100.0
    }

    pub fn total_error(&self) -> f64 {
        let logical = (self.num_cycles() * self.layout_overhead().logical_qubits())
            .to_f64()
            .expect("can convert volume as f64")
            * self.logical_patch().logical_error_rate();
        let magic_states = self.toffoli_factory_part().map_or(0.0, |p| {
            self.num_magic_states(0)
                .to_f64()
                .expect("can convert number of magic states as f64")
                * p.factory().error_probability()
        });

        logical + magic_states
    }
}

impl Deref for AliceAndBobEstimates {
    type Target = PhysicalResourceEstimationResult<RepetitionCode, ToffoliFactory, LogicalCounts>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PhysicalResourceEstimationResult<RepetitionCode, ToffoliFactory, LogicalCounts>>
    for AliceAndBobEstimates
{
    fn from(
        value: PhysicalResourceEstimationResult<RepetitionCode, ToffoliFactory, LogicalCounts>,
    ) -> Self {
        Self(value)
    }
}

impl Display for AliceAndBobEstimates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f,)?;
        writeln!(f, "─────────────────────────────")?;
        writeln!(f, "#physical qubits:    {}", self.physical_qubits())?;
        writeln!(
            f,
            "runtime:             {:.2} hrs",
            f64::from_u64(self.runtime()).expect("runtime is not too large") / 1e9 / 3600.0
        )?;
        writeln!(f, "total error:         {:.5}", self.total_error())?;
        writeln!(f, "─────────────────────────────")?;
        writeln!(
            f,
            "code distance:       {}",
            self.logical_patch().code_parameter()
        )?;
        writeln!(
            f,
            "#factories:          {}",
            self.toffoli_factory_part().map_or(0, FactoryPart::copies)
        )?;
        writeln!(f, "factory fraction:    {:.2}%", self.factory_fraction())?;
        writeln!(f, "─────────────────────────────")
    }
}
