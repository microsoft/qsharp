
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This build-generated module contains tests for the samples in the `/samples/OpenQASM` folder.
//! DO NOT MANUALLY EDIT THIS FILE. To regenerate this file, run `cargo check` or `cargo test` in the `samples_test` directory.

use super::OpenQASM::*;
use super::{compile_and_run_qasm, compile_and_run_debug_qasm};

#[allow(non_snake_case)]
fn BellPair_src() -> &'static str {
    include_str!("../../../../../samples/OpenQASM/BellPair.qasm")
}

#[allow(non_snake_case)]
#[test]
fn run_BellPair() {
    let output = compile_and_run_qasm(BellPair_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample BellPair.qasm
    BELLPAIR_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_BellPair() {
    let output = compile_and_run_debug_qasm(BellPair_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample BellPair.qasm
    BELLPAIR_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn BernsteinVazirani_src() -> &'static str {
    include_str!("../../../../../samples/OpenQASM/BernsteinVazirani.qasm")
}

#[allow(non_snake_case)]
#[test]
fn run_BernsteinVazirani() {
    let output = compile_and_run_qasm(BernsteinVazirani_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample BernsteinVazirani.qasm
    BERNSTEINVAZIRANI_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_BernsteinVazirani() {
    let output = compile_and_run_debug_qasm(BernsteinVazirani_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample BernsteinVazirani.qasm
    BERNSTEINVAZIRANI_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Grover_src() -> &'static str {
    include_str!("../../../../../samples/OpenQASM/Grover.qasm")
}

#[allow(non_snake_case)]
#[test]
fn run_Grover() {
    let output = compile_and_run_qasm(Grover_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample Grover.qasm
    GROVER_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Grover() {
    let output = compile_and_run_debug_qasm(Grover_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample Grover.qasm
    GROVER_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn OpenQasmHelloWorld_src() -> &'static str {
    include_str!("../../../../../samples/OpenQASM/OpenQasmHelloWorld.qasm")
}

#[allow(non_snake_case)]
#[test]
fn run_OpenQasmHelloWorld() {
    let output = compile_and_run_qasm(OpenQasmHelloWorld_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample OpenQasmHelloWorld.qasm
    OPENQASMHELLOWORLD_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_OpenQasmHelloWorld() {
    let output = compile_and_run_debug_qasm(OpenQasmHelloWorld_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample OpenQasmHelloWorld.qasm
    OPENQASMHELLOWORLD_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn RandomNumber_src() -> &'static str {
    include_str!("../../../../../samples/OpenQASM/RandomNumber.qasm")
}

#[allow(non_snake_case)]
#[test]
fn run_RandomNumber() {
    let output = compile_and_run_qasm(RandomNumber_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample RandomNumber.qasm
    RANDOMNUMBER_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_RandomNumber() {
    let output = compile_and_run_debug_qasm(RandomNumber_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample RandomNumber.qasm
    RANDOMNUMBER_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Simple1dIsingOrder1_src() -> &'static str {
    include_str!("../../../../../samples/OpenQASM/Simple1dIsingOrder1.qasm")
}

#[allow(non_snake_case)]
#[test]
fn run_Simple1dIsingOrder1() {
    let output = compile_and_run_qasm(Simple1dIsingOrder1_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample Simple1dIsingOrder1.qasm
    SIMPLE1DISINGORDER1_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Simple1dIsingOrder1() {
    let output = compile_and_run_debug_qasm(Simple1dIsingOrder1_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample Simple1dIsingOrder1.qasm
    SIMPLE1DISINGORDER1_EXPECT_DEBUG.assert_eq(&output);
}

#[allow(non_snake_case)]
fn Teleportation_src() -> &'static str {
    include_str!("../../../../../samples/OpenQASM/Teleportation.qasm")
}

#[allow(non_snake_case)]
#[test]
fn run_Teleportation() {
    let output = compile_and_run_qasm(Teleportation_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample Teleportation.qasm
    TELEPORTATION_EXPECT.assert_eq(&output);
}

#[allow(non_snake_case)]
#[test]
fn debug_Teleportation() {
    let output = compile_and_run_debug_qasm(Teleportation_src());
    // This constant must be defined in `samples_test/src/tests/OpenQASM.rs` and
    // must contain the output of the sample Teleportation.qasm
    TELEPORTATION_EXPECT_DEBUG.assert_eq(&output);
}
