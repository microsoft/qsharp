// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use expect_test::expect;
use qsc::{
    interpret::{GenericReceiver, Interpreter},
    LanguageFeatures, PackageType, SourceMap, TargetCapabilityFlags,
};

fn compile_and_run(sources: SourceMap) -> String {
    compile_and_run_internal(sources, false)
}

fn compile_and_run_debug(sources: SourceMap) -> String {
    compile_and_run_internal(sources, true)
}

fn compile_and_run_internal(sources: SourceMap, debug: bool) -> String {
    let mut interpreter = match (if debug {
        Interpreter::new_with_debug
    } else {
        Interpreter::new
    })(
        true,
        sources,
        PackageType::Exe,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    ) {
        Ok(interpreter) => interpreter,
        Err(errors) => {
            for error in &errors {
                eprintln!("error: {error}");
            }
            panic!("compilation failed (first error: {:?})", errors[0]);
        }
    };
    interpreter.set_classical_seed(Some(1));
    interpreter.set_quantum_seed(Some(1));

    let mut output = Vec::new();
    let mut out = GenericReceiver::new(&mut output);
    let val = match interpreter.eval_entry(&mut out) {
        Ok(val) => val,
        Err(errors) => {
            for error in &errors {
                eprintln!("error: {error}");
            }
            panic!("execution failed (first error: {:?})", errors[0]);
        }
    };
    String::from_utf8(output).expect("output should be valid UTF-8") + &val.to_string()
}

fn bell_state_src() -> SourceMap {
    SourceMap::new(
        [(
            "BellState.qs".into(),
            include_str!("../../../samples/algorithms/BellState.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_bell_state() {
    let output = compile_and_run(bell_state_src());
    expect![[r#"
        Bell state |Φ+〉:
        STATE:
        |00⟩: 0.7071+0.0000𝑖
        |11⟩: 0.7071+0.0000𝑖
        Bell state |Φ-〉:
        STATE:
        |00⟩: 0.7071+0.0000𝑖
        |11⟩: −0.7071+0.0000𝑖
        Bell state |Ψ+〉:
        STATE:
        |01⟩: 0.7071+0.0000𝑖
        |10⟩: 0.7071+0.0000𝑖
        Bell state |Ψ-〉:
        STATE:
        |01⟩: 0.7071+0.0000𝑖
        |10⟩: −0.7071+0.0000𝑖
        [(Zero, Zero), (One, One), (One, Zero), (One, Zero)]"#]]
    .assert_eq(&output);
}

#[test]
fn debug_bell_state() {
    let output = compile_and_run_debug(bell_state_src());
    expect![[r#"
        Bell state |Φ+〉:
        STATE:
        |00⟩: 0.7071+0.0000𝑖
        |11⟩: 0.7071+0.0000𝑖
        Bell state |Φ-〉:
        STATE:
        |00⟩: 0.7071+0.0000𝑖
        |11⟩: −0.7071+0.0000𝑖
        Bell state |Ψ+〉:
        STATE:
        |01⟩: 0.7071+0.0000𝑖
        |10⟩: 0.7071+0.0000𝑖
        Bell state |Ψ-〉:
        STATE:
        |01⟩: 0.7071+0.0000𝑖
        |10⟩: −0.7071+0.0000𝑖
        [(Zero, Zero), (One, One), (One, Zero), (One, Zero)]"#]]
    .assert_eq(&output);
}

fn bv_src() -> SourceMap {
    SourceMap::new(
        [(
            "BernsteinVazirani.qs".into(),
            include_str!("../../../samples/algorithms/BernsteinVazirani.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_bv() {
    let output = compile_and_run(bv_src());
    expect![[r#"
        Successfully decoded bit string as int: 127
        Successfully decoded bit string as int: 238
        Successfully decoded bit string as int: 512
        [127, 238, 512]"#]]
    .assert_eq(&output);
}

#[test]
fn debug_bv() {
    let output = compile_and_run_debug(bv_src());
    expect![[r#"
        Successfully decoded bit string as int: 127
        Successfully decoded bit string as int: 238
        Successfully decoded bit string as int: 512
        [127, 238, 512]"#]]
    .assert_eq(&output);
}

fn bv_nisq_src() -> SourceMap {
    SourceMap::new(
        [(
            "BernsteinVaziraniNISQ.qs".into(),
            include_str!("../../../samples/algorithms/BernsteinVaziraniNISQ.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_bv_nisq() {
    let output = compile_and_run(bv_nisq_src());
    expect!["[One, Zero, One, Zero, One]"].assert_eq(&output);
}

#[test]
fn debug_bv_nisq() {
    let output = compile_and_run_debug(bv_nisq_src());
    expect!["[One, Zero, One, Zero, One]"].assert_eq(&output);
}

fn bit_flip_code_src() -> SourceMap {
    SourceMap::new(
        [(
            "BitFlipCode.qs".into(),
            include_str!("../../../samples/algorithms/BitFlipCode.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_bit_flip_code() {
    let output = compile_and_run(bit_flip_code_src());
    expect![[r#"
        STATE:
        |001⟩: 0.4472+0.0000𝑖
        |110⟩: 0.8944+0.0000𝑖
        STATE:
        |000⟩: 0.4472+0.0000𝑖
        |111⟩: 0.8944+0.0000𝑖
        One"#]]
    .assert_eq(&output);
}

#[test]
fn debug_bit_flip_code() {
    let output = compile_and_run_debug(bit_flip_code_src());
    expect![[r#"
        STATE:
        |001⟩: 0.4472+0.0000𝑖
        |110⟩: 0.8944+0.0000𝑖
        STATE:
        |000⟩: 0.4472+0.0000𝑖
        |111⟩: 0.8944+0.0000𝑖
        One"#]]
    .assert_eq(&output);
}

fn cat_state_src() -> SourceMap {
    SourceMap::new(
        [(
            "CatState.qs".into(),
            include_str!("../../../samples/algorithms/CatState.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_cat_state() {
    let output = compile_and_run(cat_state_src());
    expect![[r#"
        STATE:
        |00000⟩: 0.7071+0.0000𝑖
        |11111⟩: 0.7071+0.0000𝑖
        [Zero, Zero, Zero, Zero, Zero]"#]]
    .assert_eq(&output);
}

#[test]
fn debug_cat_state() {
    let output = compile_and_run_debug(cat_state_src());
    expect![[r#"
        STATE:
        |00000⟩: 0.7071+0.0000𝑖
        |11111⟩: 0.7071+0.0000𝑖
        [Zero, Zero, Zero, Zero, Zero]"#]]
    .assert_eq(&output);
}

fn dj_src() -> SourceMap {
    SourceMap::new(
        [(
            "DeutschJozsa.qs".into(),
            include_str!("../../../samples/algorithms/DeutschJozsa.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_dj() {
    let output = compile_and_run(dj_src());
    expect![[r#"
        SimpleConstantBoolF is constant
        SimpleBalancedBoolF is balanced
        ConstantBoolF is constant
        BalancedBoolF is balanced
        [(SimpleConstantBoolF, true), (SimpleBalancedBoolF, false), (ConstantBoolF, true), (BalancedBoolF, false)]"#]].assert_eq(&output);
}

#[test]
fn debug_dj() {
    let output = compile_and_run_debug(dj_src());
    expect![[r#"
        SimpleConstantBoolF is constant
        SimpleBalancedBoolF is balanced
        ConstantBoolF is constant
        BalancedBoolF is balanced
        [(SimpleConstantBoolF, true), (SimpleBalancedBoolF, false), (ConstantBoolF, true), (BalancedBoolF, false)]"#]].assert_eq(&output);
}

fn dj_nisq_src() -> SourceMap {
    SourceMap::new(
        [(
            "DeutschJozsaNISQ.qs".into(),
            include_str!("../../../samples/algorithms/DeutschJozsaNISQ.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_dj_nisq() {
    let output = compile_and_run(dj_nisq_src());
    expect!["([One, Zero, Zero, Zero, Zero], [Zero, Zero, Zero, Zero, Zero])"].assert_eq(&output);
}

#[test]
fn debug_dj_nisq() {
    let output = compile_and_run_debug(dj_nisq_src());
    expect!["([One, Zero, Zero, Zero, Zero], [Zero, Zero, Zero, Zero, Zero])"].assert_eq(&output);
}

fn entanglement_src() -> SourceMap {
    SourceMap::new(
        [(
            "Entanglement.qs".into(),
            include_str!("../../../samples/algorithms/Entanglement.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_entanglement() {
    let output = compile_and_run(entanglement_src());
    expect![[r#"
        STATE:
        |00⟩: 0.7071+0.0000𝑖
        |11⟩: 0.7071+0.0000𝑖
        (Zero, Zero)"#]]
    .assert_eq(&output);
}

#[test]
fn debug_entanglement() {
    let output = compile_and_run_debug(entanglement_src());
    expect![[r#"
        STATE:
        |00⟩: 0.7071+0.0000𝑖
        |11⟩: 0.7071+0.0000𝑖
        (Zero, Zero)"#]]
    .assert_eq(&output);
}

fn ghz_src() -> SourceMap {
    SourceMap::new(
        [(
            "GHZ.qs".into(),
            include_str!("../../../samples/algorithms/GHZ.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_ghz() {
    let output = compile_and_run(ghz_src());
    expect![[r#"
        STATE:
        |000⟩: 0.7071+0.0000𝑖
        |111⟩: 0.7071+0.0000𝑖
        [Zero, Zero, Zero]"#]]
    .assert_eq(&output);
}

#[test]
fn debug_ghz() {
    let output = compile_and_run_debug(ghz_src());
    expect![[r#"
        STATE:
        |000⟩: 0.7071+0.0000𝑖
        |111⟩: 0.7071+0.0000𝑖
        [Zero, Zero, Zero]"#]]
    .assert_eq(&output);
}

fn grover_src() -> SourceMap {
    SourceMap::new(
        [(
            "Grover.qs".into(),
            include_str!("../../../samples/algorithms/Grover.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_grover() {
    let output = compile_and_run(grover_src());
    expect![[r#"
        Number of iterations: 4
        Reflecting about marked state...
        Reflecting about marked state...
        Reflecting about marked state...
        Reflecting about marked state...
        [Zero, One, Zero, One, Zero]"#]]
    .assert_eq(&output);
}

#[test]
fn debug_grover() {
    let output = compile_and_run_debug(grover_src());
    expect![[r#"
        Number of iterations: 4
        Reflecting about marked state...
        Reflecting about marked state...
        Reflecting about marked state...
        Reflecting about marked state...
        [Zero, One, Zero, One, Zero]"#]]
    .assert_eq(&output);
}

fn hidden_shift_src() -> SourceMap {
    SourceMap::new(
        [(
            "HiddenShift.qs".into(),
            include_str!("../../../samples/algorithms/HiddenShift.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_hidden_shift() {
    let output = compile_and_run(hidden_shift_src());
    expect![[r#"
        Found 170 successfully!
        Found 512 successfully!
        Found 999 successfully!
        [170, 512, 999]"#]]
    .assert_eq(&output);
}

#[test]
fn debug_hidden_shift() {
    let output = compile_and_run_debug(hidden_shift_src());
    expect![[r#"
        Found 170 successfully!
        Found 512 successfully!
        Found 999 successfully!
        [170, 512, 999]"#]]
    .assert_eq(&output);
}

fn hidden_shift_nisq_src() -> SourceMap {
    SourceMap::new(
        [(
            "HiddenShiftNISQ.qs".into(),
            include_str!("../../../samples/algorithms/HiddenShiftNISQ.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_hidden_shift_nisq() {
    let output = compile_and_run(hidden_shift_nisq_src());
    expect!["[One, Zero, Zero, Zero, Zero, One]"].assert_eq(&output);
}

#[test]
fn debug_hidden_shift_nisq() {
    let output = compile_and_run_debug(hidden_shift_nisq_src());
    expect!["[One, Zero, Zero, Zero, Zero, One]"].assert_eq(&output);
}

fn joint_measurement_src() -> SourceMap {
    SourceMap::new(
        [(
            "JointMeasurement.qs".into(),
            include_str!("../../../samples/algorithms/JointMeasurement.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_joint_measurement() {
    let output = compile_and_run(joint_measurement_src());
    expect![[r#"
        STATE:
        |00⟩: 0.7071+0.0000𝑖
        |11⟩: 0.7071+0.0000𝑖
        STATE:
        |00⟩: 0.7071+0.0000𝑖
        |11⟩: 0.7071+0.0000𝑖
        STATE:
        |11⟩: 1.0000+0.0000𝑖
        STATE:
        |11⟩: 1.0000+0.0000𝑖
        (Zero, [One, One])"#]]
    .assert_eq(&output);
}

#[test]
fn debug_joint_measurement() {
    let output = compile_and_run_debug(joint_measurement_src());
    expect![[r#"
        STATE:
        |00⟩: 0.7071+0.0000𝑖
        |11⟩: 0.7071+0.0000𝑖
        STATE:
        |00⟩: 0.7071+0.0000𝑖
        |11⟩: 0.7071+0.0000𝑖
        STATE:
        |11⟩: 1.0000+0.0000𝑖
        STATE:
        |11⟩: 1.0000+0.0000𝑖
        (Zero, [One, One])"#]]
    .assert_eq(&output);
}

fn measurement_src() -> SourceMap {
    SourceMap::new(
        [(
            "Measurement.qs".into(),
            include_str!("../../../samples/algorithms/Measurement.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_measurement() {
    let output = compile_and_run(measurement_src());
    expect!["(Zero, [Zero, Zero])"].assert_eq(&output);
}

#[test]
fn debug_measurement() {
    let output = compile_and_run_debug(measurement_src());
    expect!["(Zero, [Zero, Zero])"].assert_eq(&output);
}

fn phase_flip_code_src() -> SourceMap {
    SourceMap::new(
        [(
            "PhaseFlipCode.qs".into(),
            include_str!("../../../samples/algorithms/PhaseFlipCode.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_phase_flip_code() {
    let output = compile_and_run(phase_flip_code_src());
    expect![[r#"
        STATE:
        |000⟩: 0.4743+0.0000𝑖
        |001⟩: 0.1581+0.0000𝑖
        |010⟩: −0.1581+0.0000𝑖
        |011⟩: −0.4743+0.0000𝑖
        |100⟩: −0.1581+0.0000𝑖
        |101⟩: −0.4743+0.0000𝑖
        |110⟩: 0.4743+0.0000𝑖
        |111⟩: 0.1581+0.0000𝑖
        STATE:
        |000⟩: 0.4743+0.0000𝑖
        |001⟩: −0.1581+0.0000𝑖
        |010⟩: −0.1581+0.0000𝑖
        |011⟩: 0.4743+0.0000𝑖
        |100⟩: −0.1581+0.0000𝑖
        |101⟩: 0.4743+0.0000𝑖
        |110⟩: 0.4743+0.0000𝑖
        |111⟩: −0.1581+0.0000𝑖
        One"#]]
    .assert_eq(&output);
}

#[test]
fn debug_phase_flip_code() {
    let output = compile_and_run_debug(phase_flip_code_src());
    expect![[r#"
        STATE:
        |000⟩: 0.4743+0.0000𝑖
        |001⟩: 0.1581+0.0000𝑖
        |010⟩: −0.1581+0.0000𝑖
        |011⟩: −0.4743+0.0000𝑖
        |100⟩: −0.1581+0.0000𝑖
        |101⟩: −0.4743+0.0000𝑖
        |110⟩: 0.4743+0.0000𝑖
        |111⟩: 0.1581+0.0000𝑖
        STATE:
        |000⟩: 0.4743+0.0000𝑖
        |001⟩: −0.1581+0.0000𝑖
        |010⟩: −0.1581+0.0000𝑖
        |011⟩: 0.4743+0.0000𝑖
        |100⟩: −0.1581+0.0000𝑖
        |101⟩: 0.4743+0.0000𝑖
        |110⟩: 0.4743+0.0000𝑖
        |111⟩: −0.1581+0.0000𝑖
        One"#]]
    .assert_eq(&output);
}

fn qrng_src() -> SourceMap {
    SourceMap::new(
        [(
            "QRNG.qs".into(),
            include_str!("../../../samples/algorithms/QRNG.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_qrng() {
    let output = compile_and_run(qrng_src());
    expect![[r#"
        Sampling a random number between 0 and 100:
        46"#]]
    .assert_eq(&output);
}

#[test]
fn debug_qrng() {
    let output = compile_and_run_debug(qrng_src());
    expect![[r#"
        Sampling a random number between 0 and 100:
        46"#]]
    .assert_eq(&output);
}

fn qrnq_nisq_src() -> SourceMap {
    SourceMap::new(
        [(
            "QRNGNISQ.qs".into(),
            include_str!("../../../samples/algorithms/QRNGNISQ.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_qrng_nisq() {
    let output = compile_and_run(qrnq_nisq_src());
    expect!["[Zero, Zero, One, One, One]"].assert_eq(&output);
}

#[test]
fn debug_qrng_nisq() {
    let output = compile_and_run_debug(qrnq_nisq_src());
    expect!["[Zero, Zero, One, One, One]"].assert_eq(&output);
}

fn quantum_hello_world_src() -> SourceMap {
    SourceMap::new(
        [(
            "QuantumHelloWorld.qs".into(),
            include_str!("../../../samples/algorithms/QuantumHelloWorld.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_quantum_hello_world() {
    let output = compile_and_run(quantum_hello_world_src());
    expect![[r#"
        Hello world!
        Zero"#]]
    .assert_eq(&output);
}

#[test]
fn debug_quantum_hello_world() {
    let output = compile_and_run_debug(quantum_hello_world_src());
    expect![[r#"
        Hello world!
        Zero"#]]
    .assert_eq(&output);
}

fn random_bit_src() -> SourceMap {
    SourceMap::new(
        [(
            "RandomBit.qs".into(),
            include_str!("../../../samples/algorithms/RandomBit.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_random_bit() {
    let output = compile_and_run(random_bit_src());
    expect!["Zero"].assert_eq(&output);
}

#[test]
fn debug_random_bit() {
    let output = compile_and_run_debug(random_bit_src());
    expect!["Zero"].assert_eq(&output);
}

#[cfg(not(debug_assertions))]
fn shor_src() -> SourceMap {
    SourceMap::new(
        [(
            "Shor.qs".into(),
            include_str!("../../../samples/algorithms/Shor.qs").into(),
        )],
        None,
    )
}

#[cfg(not(debug_assertions))]
#[test]
fn run_shor() {
    let output = compile_and_run(shor_src());
    expect![[r#"
        *** Factorizing 143, attempt 1.
        Estimating period of 139.
        Estimating frequency with bitsPrecision=17.
        Estimated frequency=30583
        Found period=30
        Found factor=13
        Found factorization 143 = 13 * 11
        (13, 11)"#]]
    .assert_eq(&output);
}

#[cfg(not(debug_assertions))]
#[test]
fn debug_shor() {
    let output = compile_and_run_debug(shor_src());
    expect![[r#"
        *** Factorizing 143, attempt 1.
        Estimating period of 139.
        Estimating frequency with bitsPrecision=17.
        Estimated frequency=30583
        Found period=30
        Found factor=13
        Found factorization 143 = 13 * 11
        (13, 11)"#]]
    .assert_eq(&output);
}

fn superdense_coding_src() -> SourceMap {
    SourceMap::new(
        [(
            "SuperdenseCoding.qs".into(),
            include_str!("../../../samples/algorithms/SuperdenseCoding.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_superdense_coding() {
    let output = compile_and_run(superdense_coding_src());
    expect!["((false, true), (false, true))"].assert_eq(&output);
}

#[test]
fn debug_superdense_coding() {
    let output = compile_and_run_debug(superdense_coding_src());
    expect!["((false, true), (false, true))"].assert_eq(&output);
}

fn superposition_src() -> SourceMap {
    SourceMap::new(
        [(
            "Superposition.qs".into(),
            include_str!("../../../samples/algorithms/Superposition.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_superposition() {
    let output = compile_and_run(superposition_src());
    expect!["Zero"].assert_eq(&output);
}

#[test]
fn debug_superposition() {
    let output = compile_and_run_debug(superposition_src());
    expect!["Zero"].assert_eq(&output);
}

fn teleportation_src() -> SourceMap {
    SourceMap::new(
        [(
            "Teleportation.qs".into(),
            include_str!("../../../samples/algorithms/Teleportation.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_teleportation() {
    let output = compile_and_run(teleportation_src());
    expect![[r#"
        Teleporting state |0〉
        STATE:
        |0⟩: 1.0000+0.0000𝑖
        Received state |0〉
        STATE:
        |0⟩: 1.0000+0.0000𝑖
        Teleporting state |1〉
        STATE:
        |1⟩: 1.0000+0.0000𝑖
        Received state |1〉
        STATE:
        |1⟩: 1.0000+0.0000𝑖
        Teleporting state |+〉
        STATE:
        |0⟩: 0.7071+0.0000𝑖
        |1⟩: 0.7071+0.0000𝑖
        Received state |+〉
        STATE:
        |0⟩: 0.7071+0.0000𝑖
        |1⟩: 0.7071+0.0000𝑖
        Teleporting state |-〉
        STATE:
        |0⟩: 0.7071+0.0000𝑖
        |1⟩: −0.7071+0.0000𝑖
        Received state |-〉
        STATE:
        |0⟩: 0.7071+0.0000𝑖
        |1⟩: −0.7071+0.0000𝑖
        [Zero, One, Zero, One]"#]]
    .assert_eq(&output);
}

#[test]
fn debug_teleportation() {
    let output = compile_and_run_debug(teleportation_src());
    expect![[r#"
        Teleporting state |0〉
        STATE:
        |0⟩: 1.0000+0.0000𝑖
        Received state |0〉
        STATE:
        |0⟩: 1.0000+0.0000𝑖
        Teleporting state |1〉
        STATE:
        |1⟩: 1.0000+0.0000𝑖
        Received state |1〉
        STATE:
        |1⟩: 1.0000+0.0000𝑖
        Teleporting state |+〉
        STATE:
        |0⟩: 0.7071+0.0000𝑖
        |1⟩: 0.7071+0.0000𝑖
        Received state |+〉
        STATE:
        |0⟩: 0.7071+0.0000𝑖
        |1⟩: 0.7071+0.0000𝑖
        Teleporting state |-〉
        STATE:
        |0⟩: 0.7071+0.0000𝑖
        |1⟩: −0.7071+0.0000𝑖
        Received state |-〉
        STATE:
        |0⟩: 0.7071+0.0000𝑖
        |1⟩: −0.7071+0.0000𝑖
        [Zero, One, Zero, One]"#]]
    .assert_eq(&output);
}

fn three_qubit_repetition_code_src() -> SourceMap {
    SourceMap::new(
        [(
            "ThreeQubitRepetitionCode.qs".into(),
            include_str!("../../../samples/algorithms/ThreeQubitRepetitionCode.qs").into(),
        )],
        None,
    )
}

#[test]
fn run_three_qubit_repetition_code() {
    let output = compile_and_run(three_qubit_repetition_code_src());
    expect!["(true, 0)"].assert_eq(&output);
}

#[test]
fn debug_three_qubit_repetition_code() {
    let output = compile_and_run_debug(three_qubit_repetition_code_src());
    expect!["(true, 0)"].assert_eq(&output);
}
