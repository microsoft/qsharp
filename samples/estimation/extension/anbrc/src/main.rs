// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::rc::Rc;

use code::RepetitionCode;
use counter::LogicalCounts;
use estimates::AliceAndBobEstimates;
use factories::ToffoliBuilder;
use qubit::CatQubit;
use resource_estimator::estimates::{ErrorBudget, PhysicalResourceEstimation};

/// Repetition code for biased error correction with a focus on phase flips
mod code;
/// Computes logical space-time volume overhead for resource estimation from Q#
/// files or formulas for ECC application
mod counter;
/// Convenience structure to display resource estimation results
mod estimates;
/// Toffoli magic state factories
mod factories;
/// Model for cat qubits
mod qubit;

fn main() -> Result<(), anyhow::Error> {
    // ECC pre-computed counts
    // -----------------------

    // This value can be changed to investigate other key sizes, e.g., those in
    // arXiv:2302.06639 (Table IV, p. 37)
    let bit_size = 256;
    // Value w_e as reported in arXiv:2302.06639 (Table IV, p. 37)
    let window_size = 18;

    let qubit = CatQubit::new();
    let qec = RepetitionCode::new();
    let builder = ToffoliBuilder::default();
    let overhead = Rc::new(LogicalCounts::from_elliptic_curve_crypto(
        bit_size,
        window_size,
    ));
    let budget = ErrorBudget::new(0.333 * 0.5, 0.333 * 0.5, 0.0);

    let estimation =
        PhysicalResourceEstimation::new(qec, Rc::new(qubit), builder, overhead, budget);
    let result: AliceAndBobEstimates = estimation.estimate()?.into();
    println!("{result}");

    let results = estimation.build_frontier()?;

    println!("----------------------------------------");
    for r in results {
        println!("{}", AliceAndBobEstimates::from(r));
    }
    println!("----------------------------------------");

    // Resource estimation from Q#
    // ---------------------------

    let filename = format!("{}/qsharp/Adder.qs", env!("CARGO_MANIFEST_DIR"));

    let qubit = CatQubit::new();
    let qec = RepetitionCode::new();
    let builder = ToffoliBuilder::default();
    let overhead = Rc::new(LogicalCounts::from_qsharp(filename).map_err(anyhow::Error::msg)?);
    let budget = ErrorBudget::new(0.001 * 0.5, 0.001 * 0.5, 0.0);

    let estimation =
        PhysicalResourceEstimation::new(qec, Rc::new(qubit), builder, overhead, budget);
    let result: AliceAndBobEstimates = estimation.estimate()?.into();
    println!("{result}");

    Ok(())
}
