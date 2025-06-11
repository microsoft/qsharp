
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This build-generated module contains tests for the samples in the `/samples/getting_started` folder.
//! DO NOT MANUALLY EDIT THIS FILE. To regenerate this file, run `cargo check` or `cargo test` in the `samples_test` directory.

use super::getting_started::*;
use super::{compile_and_run, compile_and_run_debug};
use qsc::SourceMap;

#[allow(non_snake_case)]
fn BellPair_src() -> SourceMap {
    SourceMap::new(
        vec![("BellPair.qs".into(), include_str!("../../../../samples/getting_started/BellPair.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_BellPair() {
    let output = compile_and_run(BellPair_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample BellPair.qs
    BELLPAIR_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_BellPair() {
    let output = compile_and_run_debug(BellPair_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample BellPair.qs
    BELLPAIR_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn BellStates_src() -> SourceMap {
    SourceMap::new(
        vec![("BellStates.qs".into(), include_str!("../../../../samples/getting_started/BellStates.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_BellStates() {
    let output = compile_and_run(BellStates_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample BellStates.qs
    BELLSTATES_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_BellStates() {
    let output = compile_and_run_debug(BellStates_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample BellStates.qs
    BELLSTATES_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn CatStates_src() -> SourceMap {
    SourceMap::new(
        vec![("CatStates.qs".into(), include_str!("../../../../samples/getting_started/CatStates.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_CatStates() {
    let output = compile_and_run(CatStates_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample CatStates.qs
    CATSTATES_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_CatStates() {
    let output = compile_and_run_debug(CatStates_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample CatStates.qs
    CATSTATES_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Entanglement_src() -> SourceMap {
    SourceMap::new(
        vec![("Entanglement.qs".into(), include_str!("../../../../samples/getting_started/Entanglement.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Entanglement() {
    let output = compile_and_run(Entanglement_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample Entanglement.qs
    ENTANGLEMENT_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Entanglement() {
    let output = compile_and_run_debug(Entanglement_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample Entanglement.qs
    ENTANGLEMENT_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn JointMeasurement_src() -> SourceMap {
    SourceMap::new(
        vec![("JointMeasurement.qs".into(), include_str!("../../../../samples/getting_started/JointMeasurement.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_JointMeasurement() {
    let output = compile_and_run(JointMeasurement_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample JointMeasurement.qs
    JOINTMEASUREMENT_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_JointMeasurement() {
    let output = compile_and_run_debug(JointMeasurement_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample JointMeasurement.qs
    JOINTMEASUREMENT_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Measurement_src() -> SourceMap {
    SourceMap::new(
        vec![("Measurement.qs".into(), include_str!("../../../../samples/getting_started/Measurement.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Measurement() {
    let output = compile_and_run(Measurement_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample Measurement.qs
    MEASUREMENT_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Measurement() {
    let output = compile_and_run_debug(Measurement_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample Measurement.qs
    MEASUREMENT_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn QuantumHelloWorld_src() -> SourceMap {
    SourceMap::new(
        vec![("QuantumHelloWorld.qs".into(), include_str!("../../../../samples/getting_started/QuantumHelloWorld.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_QuantumHelloWorld() {
    let output = compile_and_run(QuantumHelloWorld_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample QuantumHelloWorld.qs
    QUANTUMHELLOWORLD_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_QuantumHelloWorld() {
    let output = compile_and_run_debug(QuantumHelloWorld_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample QuantumHelloWorld.qs
    QUANTUMHELLOWORLD_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn RandomBits_src() -> SourceMap {
    SourceMap::new(
        vec![("RandomBits.qs".into(), include_str!("../../../../samples/getting_started/RandomBits.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_RandomBits() {
    let output = compile_and_run(RandomBits_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample RandomBits.qs
    RANDOMBITS_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_RandomBits() {
    let output = compile_and_run_debug(RandomBits_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample RandomBits.qs
    RANDOMBITS_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn SimpleTeleportation_src() -> SourceMap {
    SourceMap::new(
        vec![("SimpleTeleportation.qs".into(), include_str!("../../../../samples/getting_started/SimpleTeleportation.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_SimpleTeleportation() {
    let output = compile_and_run(SimpleTeleportation_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample SimpleTeleportation.qs
    SIMPLETELEPORTATION_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_SimpleTeleportation() {
    let output = compile_and_run_debug(SimpleTeleportation_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample SimpleTeleportation.qs
    SIMPLETELEPORTATION_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Superposition_src() -> SourceMap {
    SourceMap::new(
        vec![("Superposition.qs".into(), include_str!("../../../../samples/getting_started/Superposition.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Superposition() {
    let output = compile_and_run(Superposition_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample Superposition.qs
    SUPERPOSITION_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Superposition() {
    let output = compile_and_run_debug(Superposition_src());
    // This constant must be defined in `samples_test/src/tests/getting_started.rs` and
    // must contain the output of the sample Superposition.qs
    SUPERPOSITION_EXPECT_DEBUG.assert_eq(&output);
}
