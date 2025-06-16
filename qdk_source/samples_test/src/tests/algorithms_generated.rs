
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This build-generated module contains tests for the samples in the `/samples/algorithms` folder.
//! DO NOT MANUALLY EDIT THIS FILE. To regenerate this file, run `cargo check` or `cargo test` in the `samples_test` directory.

use super::algorithms::*;
use super::{compile_and_run, compile_and_run_debug};
use qsc::SourceMap;

#[allow(non_snake_case)]
fn BernsteinVazirani_src() -> SourceMap {
    SourceMap::new(
        vec![("BernsteinVazirani.qs".into(), include_str!("../../../../samples/algorithms/BernsteinVazirani.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_BernsteinVazirani() {
    let output = compile_and_run(BernsteinVazirani_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample BernsteinVazirani.qs
    BERNSTEINVAZIRANI_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_BernsteinVazirani() {
    let output = compile_and_run_debug(BernsteinVazirani_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample BernsteinVazirani.qs
    BERNSTEINVAZIRANI_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn BernsteinVaziraniNISQ_src() -> SourceMap {
    SourceMap::new(
        vec![("BernsteinVaziraniNISQ.qs".into(), include_str!("../../../../samples/algorithms/BernsteinVaziraniNISQ.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_BernsteinVaziraniNISQ() {
    let output = compile_and_run(BernsteinVaziraniNISQ_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample BernsteinVaziraniNISQ.qs
    BERNSTEINVAZIRANINISQ_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_BernsteinVaziraniNISQ() {
    let output = compile_and_run_debug(BernsteinVaziraniNISQ_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample BernsteinVaziraniNISQ.qs
    BERNSTEINVAZIRANINISQ_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn BitFlipCode_src() -> SourceMap {
    SourceMap::new(
        vec![("BitFlipCode.qs".into(), include_str!("../../../../samples/algorithms/BitFlipCode.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_BitFlipCode() {
    let output = compile_and_run(BitFlipCode_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample BitFlipCode.qs
    BITFLIPCODE_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_BitFlipCode() {
    let output = compile_and_run_debug(BitFlipCode_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample BitFlipCode.qs
    BITFLIPCODE_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn DeutschJozsa_src() -> SourceMap {
    SourceMap::new(
        vec![("DeutschJozsa.qs".into(), include_str!("../../../../samples/algorithms/DeutschJozsa.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_DeutschJozsa() {
    let output = compile_and_run(DeutschJozsa_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample DeutschJozsa.qs
    DEUTSCHJOZSA_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_DeutschJozsa() {
    let output = compile_and_run_debug(DeutschJozsa_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample DeutschJozsa.qs
    DEUTSCHJOZSA_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn DeutschJozsaNISQ_src() -> SourceMap {
    SourceMap::new(
        vec![("DeutschJozsaNISQ.qs".into(), include_str!("../../../../samples/algorithms/DeutschJozsaNISQ.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_DeutschJozsaNISQ() {
    let output = compile_and_run(DeutschJozsaNISQ_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample DeutschJozsaNISQ.qs
    DEUTSCHJOZSANISQ_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_DeutschJozsaNISQ() {
    let output = compile_and_run_debug(DeutschJozsaNISQ_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample DeutschJozsaNISQ.qs
    DEUTSCHJOZSANISQ_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn DotProductViaPhaseEstimation_src() -> SourceMap {
    SourceMap::new(
        vec![("DotProductViaPhaseEstimation.qs".into(), include_str!("../../../../samples/algorithms/DotProductViaPhaseEstimation.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_DotProductViaPhaseEstimation() {
    let output = compile_and_run(DotProductViaPhaseEstimation_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample DotProductViaPhaseEstimation.qs
    DOTPRODUCTVIAPHASEESTIMATION_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_DotProductViaPhaseEstimation() {
    let output = compile_and_run_debug(DotProductViaPhaseEstimation_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample DotProductViaPhaseEstimation.qs
    DOTPRODUCTVIAPHASEESTIMATION_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Grover_src() -> SourceMap {
    SourceMap::new(
        vec![("Grover.qs".into(), include_str!("../../../../samples/algorithms/Grover.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Grover() {
    let output = compile_and_run(Grover_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample Grover.qs
    GROVER_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Grover() {
    let output = compile_and_run_debug(Grover_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample Grover.qs
    GROVER_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn HiddenShift_src() -> SourceMap {
    SourceMap::new(
        vec![("HiddenShift.qs".into(), include_str!("../../../../samples/algorithms/HiddenShift.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_HiddenShift() {
    let output = compile_and_run(HiddenShift_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample HiddenShift.qs
    HIDDENSHIFT_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_HiddenShift() {
    let output = compile_and_run_debug(HiddenShift_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample HiddenShift.qs
    HIDDENSHIFT_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn HiddenShiftNISQ_src() -> SourceMap {
    SourceMap::new(
        vec![("HiddenShiftNISQ.qs".into(), include_str!("../../../../samples/algorithms/HiddenShiftNISQ.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_HiddenShiftNISQ() {
    let output = compile_and_run(HiddenShiftNISQ_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample HiddenShiftNISQ.qs
    HIDDENSHIFTNISQ_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_HiddenShiftNISQ() {
    let output = compile_and_run_debug(HiddenShiftNISQ_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample HiddenShiftNISQ.qs
    HIDDENSHIFTNISQ_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn PhaseEstimation_src() -> SourceMap {
    SourceMap::new(
        vec![("PhaseEstimation.qs".into(), include_str!("../../../../samples/algorithms/PhaseEstimation.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_PhaseEstimation() {
    let output = compile_and_run(PhaseEstimation_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample PhaseEstimation.qs
    PHASEESTIMATION_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_PhaseEstimation() {
    let output = compile_and_run_debug(PhaseEstimation_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample PhaseEstimation.qs
    PHASEESTIMATION_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn PhaseFlipCode_src() -> SourceMap {
    SourceMap::new(
        vec![("PhaseFlipCode.qs".into(), include_str!("../../../../samples/algorithms/PhaseFlipCode.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_PhaseFlipCode() {
    let output = compile_and_run(PhaseFlipCode_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample PhaseFlipCode.qs
    PHASEFLIPCODE_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_PhaseFlipCode() {
    let output = compile_and_run_debug(PhaseFlipCode_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample PhaseFlipCode.qs
    PHASEFLIPCODE_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn QRNG_src() -> SourceMap {
    SourceMap::new(
        vec![("QRNG.qs".into(), include_str!("../../../../samples/algorithms/QRNG.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_QRNG() {
    let output = compile_and_run(QRNG_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample QRNG.qs
    QRNG_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_QRNG() {
    let output = compile_and_run_debug(QRNG_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample QRNG.qs
    QRNG_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Shor_src() -> SourceMap {
    SourceMap::new(
        vec![("Shor.qs".into(), include_str!("../../../../samples/algorithms/Shor.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Shor() {
    let output = compile_and_run(Shor_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample Shor.qs
    SHOR_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Shor() {
    let output = compile_and_run_debug(Shor_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample Shor.qs
    SHOR_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn SimplePhaseEstimation_src() -> SourceMap {
    SourceMap::new(
        vec![("SimplePhaseEstimation.qs".into(), include_str!("../../../../samples/algorithms/SimplePhaseEstimation.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_SimplePhaseEstimation() {
    let output = compile_and_run(SimplePhaseEstimation_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample SimplePhaseEstimation.qs
    SIMPLEPHASEESTIMATION_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_SimplePhaseEstimation() {
    let output = compile_and_run_debug(SimplePhaseEstimation_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample SimplePhaseEstimation.qs
    SIMPLEPHASEESTIMATION_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn SimpleVQE_src() -> SourceMap {
    SourceMap::new(
        vec![("SimpleVQE.qs".into(), include_str!("../../../../samples/algorithms/SimpleVQE.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_SimpleVQE() {
    let output = compile_and_run(SimpleVQE_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample SimpleVQE.qs
    SIMPLEVQE_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_SimpleVQE() {
    let output = compile_and_run_debug(SimpleVQE_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample SimpleVQE.qs
    SIMPLEVQE_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn SuperdenseCoding_src() -> SourceMap {
    SourceMap::new(
        vec![("SuperdenseCoding.qs".into(), include_str!("../../../../samples/algorithms/SuperdenseCoding.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_SuperdenseCoding() {
    let output = compile_and_run(SuperdenseCoding_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample SuperdenseCoding.qs
    SUPERDENSECODING_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_SuperdenseCoding() {
    let output = compile_and_run_debug(SuperdenseCoding_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample SuperdenseCoding.qs
    SUPERDENSECODING_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Teleportation_src() -> SourceMap {
    SourceMap::new(
        vec![("Teleportation.qs".into(), include_str!("../../../../samples/algorithms/Teleportation.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_Teleportation() {
    let output = compile_and_run(Teleportation_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample Teleportation.qs
    TELEPORTATION_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Teleportation() {
    let output = compile_and_run_debug(Teleportation_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample Teleportation.qs
    TELEPORTATION_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn ThreeQubitRepetitionCode_src() -> SourceMap {
    SourceMap::new(
        vec![("ThreeQubitRepetitionCode.qs".into(), include_str!("../../../../samples/algorithms/ThreeQubitRepetitionCode.qs").into())],
        None,
    )
}

#[allow(non_snake_case)]
#[test]
fn run_ThreeQubitRepetitionCode() {
    let output = compile_and_run(ThreeQubitRepetitionCode_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample ThreeQubitRepetitionCode.qs
    THREEQUBITREPETITIONCODE_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_ThreeQubitRepetitionCode() {
    let output = compile_and_run_debug(ThreeQubitRepetitionCode_src());
    // This constant must be defined in `samples_test/src/tests/algorithms.rs` and
    // must contain the output of the sample ThreeQubitRepetitionCode.qs
    THREEQUBITREPETITIONCODE_EXPECT_DEBUG.assert_eq(&output);
}
