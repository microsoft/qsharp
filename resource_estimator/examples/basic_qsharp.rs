#![allow(clippy::cast_precision_loss)]

// This example illustrates how to use the Resource Estimator API to perform a
// standard resource estimation on top of the qubit and QEC models provided by
// the systems architecture that we ship. We are running resource estimation on
// the Dynamics sample that is in the samples directory of the repository.

use std::{fs::read_to_string, io, rc::Rc};

use qsc::{
    compile::{core, std},
    interpret::{GenericReceiver, Interpreter},
    LanguageFeatures, PackageStore, PackageType, SourceMap, TargetCapabilityFlags,
};
use resource_estimator::{
    counts::LogicalCounter,
    estimates::{ErrorBudget, PhysicalResourceEstimation},
    system::{floquet_code, LogicalResourceCounts, PhysicalQubit, TFactoryBuilder},
};

fn main() {
    // The sample is equivalent to the simpler sample `basic_logical_counts.rs`,
    // which we advise to check first. We are using here the same error
    // correction code, the same qubit model, the same factory builder, and the
    // same error budget. The difference is that we are computing the logical
    // post-layout overhead from executing a Q# file.

    // 1) A quantum error correction code
    let code = floquet_code();

    // 2) A qubit model
    let qubit = Rc::new(PhysicalQubit::qubit_maj_ns_e6());

    // 3) A factory builder to provide magic states
    let builder = TFactoryBuilder::default();

    // 4) The logical resource overhead; in this example we are evaluating a Q#
    //    file to retrieve the logical resource counts.
    let logical_counts = Rc::new(compute_logical_counts_from_file());

    // 5) An error budget
    let budget = ErrorBudget::from_uniform(0.001);

    // Perform resource estimation and print main result metrics
    let estimation = PhysicalResourceEstimation::new(code, qubit, builder, logical_counts, budget);
    let result = estimation.estimate().expect("estimation does not fail");

    println!("Number of physical qubits: {}", result.physical_qubits());
    println!(
        "Runtime:                   {:.2e} secs",
        result.runtime() as f64 / 1e9
    );
}

fn compute_logical_counts_from_file() -> LogicalResourceCounts {
    // Construct filename from samples directory, read its contents, and create
    // a source object for the Q# interpreter
    let filename = format!(
        "{}/../samples/estimation/Dynamics.qs",
        env!("CARGO_MANIFEST_DIR")
    );
    let contents = read_to_string(&filename).expect("can read from file");
    let sources = SourceMap::new([("Dynamics.qs".into(), contents.into())], None);

    // Create a package store and dependency to the Q# standard library
    let mut store = PackageStore::new(core());
    let std = store.insert(std(&store, TargetCapabilityFlags::all()));

    // Construct an interpreter with all target capabilities for resource
    // estimation, using the sources and the standard library dependency
    let mut interpreter = Interpreter::new(
        sources,
        PackageType::Lib,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
        store,
        &[(std, None)],
    )
    .expect("can create interpreter");

    // Create a logical counter that is used as the simulation backend for the
    // interpreter
    let mut logical_counter = LogicalCounter::default();

    // Interpret the source code and return the computed logical resource counts
    interpreter
        .run_with_sim(
            &mut logical_counter,
            &mut GenericReceiver::new(&mut io::sink()),
            Some("QuantumDynamics.Main()"),
        )
        .expect("can execute file");
    logical_counter.logical_resources()
}
