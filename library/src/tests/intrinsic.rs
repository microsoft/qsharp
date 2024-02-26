// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;
use indoc::indoc;
use qsc::{interpret::Value, target::Profile, SparseSim};

use super::test_expression_with_lib_and_profile_and_sim;

// These tests verify multi-controlled decomposition logic for gate operations. Each test
// manually allocates 2N qubits, performs the decomposed operation from the library on the first N,
// verifies the resulting state via dump, and then uncomputes the operation via simulator-native
// multi-controlled operations to verify via Choi-Jamiolkowski isomorphism that the decomposition
// was correct.

#[test]
fn test_mch_1_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(2);
            let aux = QIR.Runtime.AllocateQubitArray(2);
            for i in 0..1 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled H(qs[0..0], qs[1]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000⟩: 0.5000+0.0000𝑖
        |0101⟩: 0.5000+0.0000𝑖
        |1010⟩: 0.3536+0.0000𝑖
        |1011⟩: 0.3536+0.0000𝑖
        |1110⟩: 0.3536+0.0000𝑖
        |1111⟩: −0.3536+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mch(&[0], 1);
    for i in 0..2 {
        sim.sim.mcx(&[i + 2], i);
        sim.sim.h(i + 2);
        assert!(sim.sim.qubit_is_zero(i + 2), "qubit {} is not zero", i + 2);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_mch_2_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(3);
            let aux = QIR.Runtime.AllocateQubitArray(3);
            for i in 0..2 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled H(qs[0..1], qs[2]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |000000⟩: 0.3536+0.0000𝑖
        |001001⟩: 0.3536+0.0000𝑖
        |010010⟩: 0.3536+0.0000𝑖
        |011011⟩: 0.3536+0.0000𝑖
        |100100⟩: 0.3536+0.0000𝑖
        |101101⟩: 0.3536+0.0000𝑖
        |110110⟩: 0.2500+0.0000𝑖
        |110111⟩: 0.2500+0.0000𝑖
        |111110⟩: 0.2500+0.0000𝑖
        |111111⟩: −0.2500+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mch(&[0, 1], 2);
    for i in 0..3 {
        sim.sim.mcx(&[i + 3], i);
        sim.sim.h(i + 3);
        assert!(sim.sim.qubit_is_zero(i + 3), "qubit {} is not zero", i + 3);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mch_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled H(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2500+0.0000𝑖
        |00010001⟩: 0.2500+0.0000𝑖
        |00100010⟩: 0.2500+0.0000𝑖
        |00110011⟩: 0.2500+0.0000𝑖
        |01000100⟩: 0.2500+0.0000𝑖
        |01010101⟩: 0.2500+0.0000𝑖
        |01100110⟩: 0.2500+0.0000𝑖
        |01110111⟩: 0.2500+0.0000𝑖
        |10001000⟩: 0.2500+0.0000𝑖
        |10011001⟩: 0.2500+0.0000𝑖
        |10101010⟩: 0.2500+0.0000𝑖
        |10111011⟩: 0.2500+0.0000𝑖
        |11001100⟩: 0.2500+0.0000𝑖
        |11011101⟩: 0.2500+0.0000𝑖
        |11101110⟩: 0.1768+0.0000𝑖
        |11101111⟩: 0.1768+0.0000𝑖
        |11111110⟩: 0.1768+0.0000𝑖
        |11111111⟩: −0.1768+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mch(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mch_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled H(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2500+0.0000𝑖
        |00010001⟩: 0.2500+0.0000𝑖
        |00100010⟩: 0.2500+0.0000𝑖
        |00110011⟩: 0.2500+0.0000𝑖
        |01000100⟩: 0.2500+0.0000𝑖
        |01010101⟩: 0.2500+0.0000𝑖
        |01100110⟩: 0.2500+0.0000𝑖
        |01110111⟩: 0.2500+0.0000𝑖
        |10001000⟩: 0.2500+0.0000𝑖
        |10011001⟩: 0.2500+0.0000𝑖
        |10101010⟩: 0.2500+0.0000𝑖
        |10111011⟩: 0.2500+0.0000𝑖
        |11001100⟩: 0.2500+0.0000𝑖
        |11011101⟩: 0.2500+0.0000𝑖
        |11101110⟩: 0.1768+0.0000𝑖
        |11101111⟩: 0.1768+0.0000𝑖
        |11111110⟩: 0.1768+0.0000𝑖
        |11111111⟩: −0.1768+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mch(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mch_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled H(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1768+0.0000𝑖
        |0000100001⟩: 0.1768+0.0000𝑖
        |0001000010⟩: 0.1768+0.0000𝑖
        |0001100011⟩: 0.1768+0.0000𝑖
        |0010000100⟩: 0.1768+0.0000𝑖
        |0010100101⟩: 0.1768+0.0000𝑖
        |0011000110⟩: 0.1768+0.0000𝑖
        |0011100111⟩: 0.1768+0.0000𝑖
        |0100001000⟩: 0.1768+0.0000𝑖
        |0100101001⟩: 0.1768+0.0000𝑖
        |0101001010⟩: 0.1768+0.0000𝑖
        |0101101011⟩: 0.1768+0.0000𝑖
        |0110001100⟩: 0.1768+0.0000𝑖
        |0110101101⟩: 0.1768+0.0000𝑖
        |0111001110⟩: 0.1768+0.0000𝑖
        |0111101111⟩: 0.1768+0.0000𝑖
        |1000010000⟩: 0.1768+0.0000𝑖
        |1000110001⟩: 0.1768+0.0000𝑖
        |1001010010⟩: 0.1768+0.0000𝑖
        |1001110011⟩: 0.1768+0.0000𝑖
        |1010010100⟩: 0.1768+0.0000𝑖
        |1010110101⟩: 0.1768+0.0000𝑖
        |1011010110⟩: 0.1768+0.0000𝑖
        |1011110111⟩: 0.1768+0.0000𝑖
        |1100011000⟩: 0.1768+0.0000𝑖
        |1100111001⟩: 0.1768+0.0000𝑖
        |1101011010⟩: 0.1768+0.0000𝑖
        |1101111011⟩: 0.1768+0.0000𝑖
        |1110011100⟩: 0.1768+0.0000𝑖
        |1110111101⟩: 0.1768+0.0000𝑖
        |1111011110⟩: 0.1250+0.0000𝑖
        |1111011111⟩: 0.1250+0.0000𝑖
        |1111111110⟩: 0.1250+0.0000𝑖
        |1111111111⟩: −0.1250+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mch(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mch_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled H(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1768+0.0000𝑖
        |0000100001⟩: 0.1768+0.0000𝑖
        |0001000010⟩: 0.1768+0.0000𝑖
        |0001100011⟩: 0.1768+0.0000𝑖
        |0010000100⟩: 0.1768+0.0000𝑖
        |0010100101⟩: 0.1768+0.0000𝑖
        |0011000110⟩: 0.1768+0.0000𝑖
        |0011100111⟩: 0.1768+0.0000𝑖
        |0100001000⟩: 0.1768+0.0000𝑖
        |0100101001⟩: 0.1768+0.0000𝑖
        |0101001010⟩: 0.1768+0.0000𝑖
        |0101101011⟩: 0.1768+0.0000𝑖
        |0110001100⟩: 0.1768+0.0000𝑖
        |0110101101⟩: 0.1768+0.0000𝑖
        |0111001110⟩: 0.1768+0.0000𝑖
        |0111101111⟩: 0.1768+0.0000𝑖
        |1000010000⟩: 0.1768+0.0000𝑖
        |1000110001⟩: 0.1768+0.0000𝑖
        |1001010010⟩: 0.1768+0.0000𝑖
        |1001110011⟩: 0.1768+0.0000𝑖
        |1010010100⟩: 0.1768+0.0000𝑖
        |1010110101⟩: 0.1768+0.0000𝑖
        |1011010110⟩: 0.1768+0.0000𝑖
        |1011110111⟩: 0.1768+0.0000𝑖
        |1100011000⟩: 0.1768+0.0000𝑖
        |1100111001⟩: 0.1768+0.0000𝑖
        |1101011010⟩: 0.1768+0.0000𝑖
        |1101111011⟩: 0.1768+0.0000𝑖
        |1110011100⟩: 0.1768+0.0000𝑖
        |1110111101⟩: 0.1768+0.0000𝑖
        |1111011110⟩: 0.1250+0.0000𝑖
        |1111011111⟩: 0.1250+0.0000𝑖
        |1111111110⟩: 0.1250+0.0000𝑖
        |1111111111⟩: −0.1250+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mch(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_mcrz_1_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(2);
            let aux = QIR.Runtime.AllocateQubitArray(2);
            for i in 0..1 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Rz(qs[0..0], (Microsoft.Quantum.Math.PI() / 7.0, qs[1]));
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000⟩: 0.5000+0.0000𝑖
        |0101⟩: 0.5000+0.0000𝑖
        |1010⟩: 0.4875−0.1113𝑖
        |1111⟩: 0.4875+0.1113𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcrz(&[0], std::f64::consts::PI / -7.0, 1);
    for i in 0..2 {
        sim.sim.mcx(&[i + 2], i);
        sim.sim.h(i + 2);
        assert!(sim.sim.qubit_is_zero(i + 2), "qubit {} is not zero", i + 2);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mcrz_2_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(3);
            let aux = QIR.Runtime.AllocateQubitArray(3);
            for i in 0..2 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Rz(qs[0..1], (Microsoft.Quantum.Math.PI() / 7.0, qs[2]));
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |000000⟩: 0.3536+0.0000𝑖
        |001001⟩: 0.3536+0.0000𝑖
        |010010⟩: 0.3536+0.0000𝑖
        |011011⟩: 0.3536+0.0000𝑖
        |100100⟩: 0.3536+0.0000𝑖
        |101101⟩: 0.3536+0.0000𝑖
        |110110⟩: 0.3447−0.0787𝑖
        |111111⟩: 0.3447+0.0787𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcrz(&[0, 1], std::f64::consts::PI / -7.0, 2);
    for i in 0..3 {
        sim.sim.mcx(&[i + 3], i);
        sim.sim.h(i + 3);
        assert!(sim.sim.qubit_is_zero(i + 3), "qubit {} is not zero", i + 3);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mcrz_2_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(3);
            let aux = QIR.Runtime.AllocateQubitArray(3);
            for i in 0..2 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Rz(qs[0..1], (Microsoft.Quantum.Math.PI() / 7.0, qs[2]));
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |000000⟩: 0.3536+0.0000𝑖
        |001001⟩: 0.3536+0.0000𝑖
        |010010⟩: 0.3536+0.0000𝑖
        |011011⟩: 0.3536+0.0000𝑖
        |100100⟩: 0.3536+0.0000𝑖
        |101101⟩: 0.3536+0.0000𝑖
        |110110⟩: 0.3447−0.0787𝑖
        |111111⟩: 0.3447+0.0787𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcrz(&[0, 1], std::f64::consts::PI / -7.0, 2);
    for i in 0..3 {
        sim.sim.mcx(&[i + 3], i);
        sim.sim.h(i + 3);
        assert!(sim.sim.qubit_is_zero(i + 3), "qubit {} is not zero", i + 3);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mcrz_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Rz(qs[0..2], (Microsoft.Quantum.Math.PI() / 7.0, qs[3]));
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2500+0.0000𝑖
        |00010001⟩: 0.2500+0.0000𝑖
        |00100010⟩: 0.2500+0.0000𝑖
        |00110011⟩: 0.2500+0.0000𝑖
        |01000100⟩: 0.2500+0.0000𝑖
        |01010101⟩: 0.2500+0.0000𝑖
        |01100110⟩: 0.2500+0.0000𝑖
        |01110111⟩: 0.2500+0.0000𝑖
        |10001000⟩: 0.2500+0.0000𝑖
        |10011001⟩: 0.2500+0.0000𝑖
        |10101010⟩: 0.2500+0.0000𝑖
        |10111011⟩: 0.2500+0.0000𝑖
        |11001100⟩: 0.2500+0.0000𝑖
        |11011101⟩: 0.2500+0.0000𝑖
        |11101110⟩: 0.2437−0.0556𝑖
        |11111111⟩: 0.2437+0.0556𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcrz(&[0, 1, 2], std::f64::consts::PI / -7.0, 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mcrz_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Rz(qs[0..2], (Microsoft.Quantum.Math.PI() / 7.0, qs[3]));
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2500+0.0000𝑖
        |00010001⟩: 0.2500+0.0000𝑖
        |00100010⟩: 0.2500+0.0000𝑖
        |00110011⟩: 0.2500+0.0000𝑖
        |01000100⟩: 0.2500+0.0000𝑖
        |01010101⟩: 0.2500+0.0000𝑖
        |01100110⟩: 0.2500+0.0000𝑖
        |01110111⟩: 0.2500+0.0000𝑖
        |10001000⟩: 0.2500+0.0000𝑖
        |10011001⟩: 0.2500+0.0000𝑖
        |10101010⟩: 0.2500+0.0000𝑖
        |10111011⟩: 0.2500+0.0000𝑖
        |11001100⟩: 0.2500+0.0000𝑖
        |11011101⟩: 0.2500+0.0000𝑖
        |11101110⟩: 0.2437−0.0556𝑖
        |11111111⟩: 0.2437+0.0556𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcrz(&[0, 1, 2], std::f64::consts::PI / -7.0, 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_mcrx_1_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(2);
            let aux = QIR.Runtime.AllocateQubitArray(2);
            for i in 0..1 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Rx(qs[0..0], (Microsoft.Quantum.Math.PI() / 7.0, qs[1]));
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000⟩: 0.5000+0.0000𝑖
        |0101⟩: 0.5000+0.0000𝑖
        |1010⟩: 0.4875+0.0000𝑖
        |1011⟩: 0.0000−0.1113𝑖
        |1110⟩: 0.0000−0.1113𝑖
        |1111⟩: 0.4875+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcrx(&[0], std::f64::consts::PI / -7.0, 1);
    for i in 0..2 {
        sim.sim.mcx(&[i + 2], i);
        sim.sim.h(i + 2);
        assert!(sim.sim.qubit_is_zero(i + 2), "qubit {} is not zero", i + 2);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_mcry_1_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(2);
            let aux = QIR.Runtime.AllocateQubitArray(2);
            for i in 0..1 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Ry(qs[0..0], (Microsoft.Quantum.Math.PI() / 7.0, qs[1]));
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000⟩: 0.5000+0.0000𝑖
        |0101⟩: 0.5000+0.0000𝑖
        |1010⟩: 0.4875+0.0000𝑖
        |1011⟩: −0.1113+0.0000𝑖
        |1110⟩: 0.1113+0.0000𝑖
        |1111⟩: 0.4875+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcry(&[0], std::f64::consts::PI / -7.0, 1);
    for i in 0..2 {
        sim.sim.mcx(&[i + 2], i);
        sim.sim.h(i + 2);
        assert!(sim.sim.qubit_is_zero(i + 2), "qubit {} is not zero", i + 2);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_mcs_1_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(2);
            let aux = QIR.Runtime.AllocateQubitArray(2);
            for i in 0..1 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled S(qs[0..0], qs[1]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000⟩: 0.5000+0.0000𝑖
        |0101⟩: 0.5000+0.0000𝑖
        |1010⟩: 0.5000+0.0000𝑖
        |1111⟩: 0.0000+0.5000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcsadj(&[0], 1);
    for i in 0..2 {
        sim.sim.mcx(&[i + 2], i);
        sim.sim.h(i + 2);
        assert!(sim.sim.qubit_is_zero(i + 2), "qubit {} is not zero", i + 2);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_mcs_2_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(3);
            let aux = QIR.Runtime.AllocateQubitArray(3);
            for i in 0..2 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled S(qs[0..1], qs[2]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |000000⟩: 0.3468−0.0690𝑖
        |001001⟩: 0.3468−0.0690𝑖
        |010010⟩: 0.3468−0.0690𝑖
        |011011⟩: 0.3468−0.0690𝑖
        |100100⟩: 0.3468−0.0690𝑖
        |101101⟩: 0.3468−0.0690𝑖
        |110110⟩: 0.3468−0.0690𝑖
        |111111⟩: 0.0690+0.3468𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcsadj(&[0, 1], 2);
    for i in 0..3 {
        sim.sim.mcx(&[i + 3], i);
        sim.sim.h(i + 3);
        assert!(sim.sim.qubit_is_zero(i + 3), "qubit {} is not zero", i + 3);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mcs_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled S(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2452−0.0488𝑖
        |00010001⟩: 0.2452−0.0488𝑖
        |00100010⟩: 0.2452−0.0488𝑖
        |00110011⟩: 0.2452−0.0488𝑖
        |01000100⟩: 0.2452−0.0488𝑖
        |01010101⟩: 0.2452−0.0488𝑖
        |01100110⟩: 0.2452−0.0488𝑖
        |01110111⟩: 0.2452−0.0488𝑖
        |10001000⟩: 0.2452−0.0488𝑖
        |10011001⟩: 0.2452−0.0488𝑖
        |10101010⟩: 0.2452−0.0488𝑖
        |10111011⟩: 0.2452−0.0488𝑖
        |11001100⟩: 0.2452−0.0488𝑖
        |11011101⟩: 0.2452−0.0488𝑖
        |11101110⟩: 0.2452−0.0488𝑖
        |11111111⟩: 0.0488+0.2452𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcsadj(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mcs_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled S(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2452−0.0488𝑖
        |00010001⟩: 0.2452−0.0488𝑖
        |00100010⟩: 0.2452−0.0488𝑖
        |00110011⟩: 0.2452−0.0488𝑖
        |01000100⟩: 0.2452−0.0488𝑖
        |01010101⟩: 0.2452−0.0488𝑖
        |01100110⟩: 0.2452−0.0488𝑖
        |01110111⟩: 0.2452−0.0488𝑖
        |10001000⟩: 0.2452−0.0488𝑖
        |10011001⟩: 0.2452−0.0488𝑖
        |10101010⟩: 0.2452−0.0488𝑖
        |10111011⟩: 0.2452−0.0488𝑖
        |11001100⟩: 0.2452−0.0488𝑖
        |11011101⟩: 0.2452−0.0488𝑖
        |11101110⟩: 0.2452−0.0488𝑖
        |11111111⟩: 0.0488+0.2452𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcsadj(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mcs_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled S(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1734−0.0345𝑖
        |0000100001⟩: 0.1734−0.0345𝑖
        |0001000010⟩: 0.1734−0.0345𝑖
        |0001100011⟩: 0.1734−0.0345𝑖
        |0010000100⟩: 0.1734−0.0345𝑖
        |0010100101⟩: 0.1734−0.0345𝑖
        |0011000110⟩: 0.1734−0.0345𝑖
        |0011100111⟩: 0.1734−0.0345𝑖
        |0100001000⟩: 0.1734−0.0345𝑖
        |0100101001⟩: 0.1734−0.0345𝑖
        |0101001010⟩: 0.1734−0.0345𝑖
        |0101101011⟩: 0.1734−0.0345𝑖
        |0110001100⟩: 0.1734−0.0345𝑖
        |0110101101⟩: 0.1734−0.0345𝑖
        |0111001110⟩: 0.1734−0.0345𝑖
        |0111101111⟩: 0.1734−0.0345𝑖
        |1000010000⟩: 0.1734−0.0345𝑖
        |1000110001⟩: 0.1734−0.0345𝑖
        |1001010010⟩: 0.1734−0.0345𝑖
        |1001110011⟩: 0.1734−0.0345𝑖
        |1010010100⟩: 0.1734−0.0345𝑖
        |1010110101⟩: 0.1734−0.0345𝑖
        |1011010110⟩: 0.1734−0.0345𝑖
        |1011110111⟩: 0.1734−0.0345𝑖
        |1100011000⟩: 0.1734−0.0345𝑖
        |1100111001⟩: 0.1734−0.0345𝑖
        |1101011010⟩: 0.1734−0.0345𝑖
        |1101111011⟩: 0.1734−0.0345𝑖
        |1110011100⟩: 0.1734−0.0345𝑖
        |1110111101⟩: 0.1734−0.0345𝑖
        |1111011110⟩: 0.1734−0.0345𝑖
        |1111111111⟩: 0.0345+0.1734𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcsadj(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mcs_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled S(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1734−0.0345𝑖
        |0000100001⟩: 0.1734−0.0345𝑖
        |0001000010⟩: 0.1734−0.0345𝑖
        |0001100011⟩: 0.1734−0.0345𝑖
        |0010000100⟩: 0.1734−0.0345𝑖
        |0010100101⟩: 0.1734−0.0345𝑖
        |0011000110⟩: 0.1734−0.0345𝑖
        |0011100111⟩: 0.1734−0.0345𝑖
        |0100001000⟩: 0.1734−0.0345𝑖
        |0100101001⟩: 0.1734−0.0345𝑖
        |0101001010⟩: 0.1734−0.0345𝑖
        |0101101011⟩: 0.1734−0.0345𝑖
        |0110001100⟩: 0.1734−0.0345𝑖
        |0110101101⟩: 0.1734−0.0345𝑖
        |0111001110⟩: 0.1734−0.0345𝑖
        |0111101111⟩: 0.1734−0.0345𝑖
        |1000010000⟩: 0.1734−0.0345𝑖
        |1000110001⟩: 0.1734−0.0345𝑖
        |1001010010⟩: 0.1734−0.0345𝑖
        |1001110011⟩: 0.1734−0.0345𝑖
        |1010010100⟩: 0.1734−0.0345𝑖
        |1010110101⟩: 0.1734−0.0345𝑖
        |1011010110⟩: 0.1734−0.0345𝑖
        |1011110111⟩: 0.1734−0.0345𝑖
        |1100011000⟩: 0.1734−0.0345𝑖
        |1100111001⟩: 0.1734−0.0345𝑖
        |1101011010⟩: 0.1734−0.0345𝑖
        |1101111011⟩: 0.1734−0.0345𝑖
        |1110011100⟩: 0.1734−0.0345𝑖
        |1110111101⟩: 0.1734−0.0345𝑖
        |1111011110⟩: 0.1734−0.0345𝑖
        |1111111111⟩: 0.0345+0.1734𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcsadj(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_mcsadj_1_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(2);
            let aux = QIR.Runtime.AllocateQubitArray(2);
            for i in 0..1 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Adjoint S(qs[0..0], qs[1]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000⟩: 0.5000+0.0000𝑖
        |0101⟩: 0.5000+0.0000𝑖
        |1010⟩: 0.5000+0.0000𝑖
        |1111⟩: 0.0000−0.5000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcs(&[0], 1);
    for i in 0..2 {
        sim.sim.mcx(&[i + 2], i);
        sim.sim.h(i + 2);
        assert!(sim.sim.qubit_is_zero(i + 2), "qubit {} is not zero", i + 2);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_mcsadj_2_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(3);
            let aux = QIR.Runtime.AllocateQubitArray(3);
            for i in 0..2 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Adjoint S(qs[0..1], qs[2]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |000000⟩: 0.3468+0.0690𝑖
        |001001⟩: 0.3468+0.0690𝑖
        |010010⟩: 0.3468+0.0690𝑖
        |011011⟩: 0.3468+0.0690𝑖
        |100100⟩: 0.3468+0.0690𝑖
        |101101⟩: 0.3468+0.0690𝑖
        |110110⟩: 0.3468+0.0690𝑖
        |111111⟩: 0.0690−0.3468𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcs(&[0, 1], 2);
    for i in 0..3 {
        sim.sim.mcx(&[i + 3], i);
        sim.sim.h(i + 3);
        assert!(sim.sim.qubit_is_zero(i + 3), "qubit {} is not zero", i + 3);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mcsadj_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Adjoint S(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2452+0.0488𝑖
        |00010001⟩: 0.2452+0.0488𝑖
        |00100010⟩: 0.2452+0.0488𝑖
        |00110011⟩: 0.2452+0.0488𝑖
        |01000100⟩: 0.2452+0.0488𝑖
        |01010101⟩: 0.2452+0.0488𝑖
        |01100110⟩: 0.2452+0.0488𝑖
        |01110111⟩: 0.2452+0.0488𝑖
        |10001000⟩: 0.2452+0.0488𝑖
        |10011001⟩: 0.2452+0.0488𝑖
        |10101010⟩: 0.2452+0.0488𝑖
        |10111011⟩: 0.2452+0.0488𝑖
        |11001100⟩: 0.2452+0.0488𝑖
        |11011101⟩: 0.2452+0.0488𝑖
        |11101110⟩: 0.2452+0.0488𝑖
        |11111111⟩: 0.0488−0.2452𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcs(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mcsadj_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Adjoint S(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2452+0.0488𝑖
        |00010001⟩: 0.2452+0.0488𝑖
        |00100010⟩: 0.2452+0.0488𝑖
        |00110011⟩: 0.2452+0.0488𝑖
        |01000100⟩: 0.2452+0.0488𝑖
        |01010101⟩: 0.2452+0.0488𝑖
        |01100110⟩: 0.2452+0.0488𝑖
        |01110111⟩: 0.2452+0.0488𝑖
        |10001000⟩: 0.2452+0.0488𝑖
        |10011001⟩: 0.2452+0.0488𝑖
        |10101010⟩: 0.2452+0.0488𝑖
        |10111011⟩: 0.2452+0.0488𝑖
        |11001100⟩: 0.2452+0.0488𝑖
        |11011101⟩: 0.2452+0.0488𝑖
        |11101110⟩: 0.2452+0.0488𝑖
        |11111111⟩: 0.0488−0.2452𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcs(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mcsadj_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Adjoint S(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1734+0.0345𝑖
        |0000100001⟩: 0.1734+0.0345𝑖
        |0001000010⟩: 0.1734+0.0345𝑖
        |0001100011⟩: 0.1734+0.0345𝑖
        |0010000100⟩: 0.1734+0.0345𝑖
        |0010100101⟩: 0.1734+0.0345𝑖
        |0011000110⟩: 0.1734+0.0345𝑖
        |0011100111⟩: 0.1734+0.0345𝑖
        |0100001000⟩: 0.1734+0.0345𝑖
        |0100101001⟩: 0.1734+0.0345𝑖
        |0101001010⟩: 0.1734+0.0345𝑖
        |0101101011⟩: 0.1734+0.0345𝑖
        |0110001100⟩: 0.1734+0.0345𝑖
        |0110101101⟩: 0.1734+0.0345𝑖
        |0111001110⟩: 0.1734+0.0345𝑖
        |0111101111⟩: 0.1734+0.0345𝑖
        |1000010000⟩: 0.1734+0.0345𝑖
        |1000110001⟩: 0.1734+0.0345𝑖
        |1001010010⟩: 0.1734+0.0345𝑖
        |1001110011⟩: 0.1734+0.0345𝑖
        |1010010100⟩: 0.1734+0.0345𝑖
        |1010110101⟩: 0.1734+0.0345𝑖
        |1011010110⟩: 0.1734+0.0345𝑖
        |1011110111⟩: 0.1734+0.0345𝑖
        |1100011000⟩: 0.1734+0.0345𝑖
        |1100111001⟩: 0.1734+0.0345𝑖
        |1101011010⟩: 0.1734+0.0345𝑖
        |1101111011⟩: 0.1734+0.0345𝑖
        |1110011100⟩: 0.1734+0.0345𝑖
        |1110111101⟩: 0.1734+0.0345𝑖
        |1111011110⟩: 0.1734+0.0345𝑖
        |1111111111⟩: 0.0345−0.1734𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcs(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mcsadj_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Adjoint S(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1734+0.0345𝑖
        |0000100001⟩: 0.1734+0.0345𝑖
        |0001000010⟩: 0.1734+0.0345𝑖
        |0001100011⟩: 0.1734+0.0345𝑖
        |0010000100⟩: 0.1734+0.0345𝑖
        |0010100101⟩: 0.1734+0.0345𝑖
        |0011000110⟩: 0.1734+0.0345𝑖
        |0011100111⟩: 0.1734+0.0345𝑖
        |0100001000⟩: 0.1734+0.0345𝑖
        |0100101001⟩: 0.1734+0.0345𝑖
        |0101001010⟩: 0.1734+0.0345𝑖
        |0101101011⟩: 0.1734+0.0345𝑖
        |0110001100⟩: 0.1734+0.0345𝑖
        |0110101101⟩: 0.1734+0.0345𝑖
        |0111001110⟩: 0.1734+0.0345𝑖
        |0111101111⟩: 0.1734+0.0345𝑖
        |1000010000⟩: 0.1734+0.0345𝑖
        |1000110001⟩: 0.1734+0.0345𝑖
        |1001010010⟩: 0.1734+0.0345𝑖
        |1001110011⟩: 0.1734+0.0345𝑖
        |1010010100⟩: 0.1734+0.0345𝑖
        |1010110101⟩: 0.1734+0.0345𝑖
        |1011010110⟩: 0.1734+0.0345𝑖
        |1011110111⟩: 0.1734+0.0345𝑖
        |1100011000⟩: 0.1734+0.0345𝑖
        |1100111001⟩: 0.1734+0.0345𝑖
        |1101011010⟩: 0.1734+0.0345𝑖
        |1101111011⟩: 0.1734+0.0345𝑖
        |1110011100⟩: 0.1734+0.0345𝑖
        |1110111101⟩: 0.1734+0.0345𝑖
        |1111011110⟩: 0.1734+0.0345𝑖
        |1111111111⟩: 0.0345−0.1734𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcs(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_mct_1_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(2);
            let aux = QIR.Runtime.AllocateQubitArray(2);
            for i in 0..1 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled T(qs[0..0], qs[1]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000⟩: 0.4904−0.0975𝑖
        |0101⟩: 0.4904−0.0975𝑖
        |1010⟩: 0.4904−0.0975𝑖
        |1111⟩: 0.4157+0.2778𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mctadj(&[0], 1);
    for i in 0..2 {
        sim.sim.mcx(&[i + 2], i);
        sim.sim.h(i + 2);
        assert!(sim.sim.qubit_is_zero(i + 2), "qubit {} is not zero", i + 2);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_mct_2_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(3);
            let aux = QIR.Runtime.AllocateQubitArray(3);
            for i in 0..2 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled T(qs[0..1], qs[2]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |000000⟩: 0.3468−0.0690𝑖
        |001001⟩: 0.3468−0.0690𝑖
        |010010⟩: 0.3468−0.0690𝑖
        |011011⟩: 0.3468−0.0690𝑖
        |100100⟩: 0.3468−0.0690𝑖
        |101101⟩: 0.3468−0.0690𝑖
        |110110⟩: 0.3468−0.0690𝑖
        |111111⟩: 0.2940+0.1964𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mctadj(&[0, 1], 2);
    for i in 0..3 {
        sim.sim.mcx(&[i + 3], i);
        sim.sim.h(i + 3);
        assert!(sim.sim.qubit_is_zero(i + 3), "qubit {} is not zero", i + 3);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mct_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled T(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2452−0.0488𝑖
        |00010001⟩: 0.2452−0.0488𝑖
        |00100010⟩: 0.2452−0.0488𝑖
        |00110011⟩: 0.2452−0.0488𝑖
        |01000100⟩: 0.2452−0.0488𝑖
        |01010101⟩: 0.2452−0.0488𝑖
        |01100110⟩: 0.2452−0.0488𝑖
        |01110111⟩: 0.2452−0.0488𝑖
        |10001000⟩: 0.2452−0.0488𝑖
        |10011001⟩: 0.2452−0.0488𝑖
        |10101010⟩: 0.2452−0.0488𝑖
        |10111011⟩: 0.2452−0.0488𝑖
        |11001100⟩: 0.2452−0.0488𝑖
        |11011101⟩: 0.2452−0.0488𝑖
        |11101110⟩: 0.2452−0.0488𝑖
        |11111111⟩: 0.2079+0.1389𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mctadj(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mct_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled T(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2452−0.0488𝑖
        |00010001⟩: 0.2452−0.0488𝑖
        |00100010⟩: 0.2452−0.0488𝑖
        |00110011⟩: 0.2452−0.0488𝑖
        |01000100⟩: 0.2452−0.0488𝑖
        |01010101⟩: 0.2452−0.0488𝑖
        |01100110⟩: 0.2452−0.0488𝑖
        |01110111⟩: 0.2452−0.0488𝑖
        |10001000⟩: 0.2452−0.0488𝑖
        |10011001⟩: 0.2452−0.0488𝑖
        |10101010⟩: 0.2452−0.0488𝑖
        |10111011⟩: 0.2452−0.0488𝑖
        |11001100⟩: 0.2452−0.0488𝑖
        |11011101⟩: 0.2452−0.0488𝑖
        |11101110⟩: 0.2452−0.0488𝑖
        |11111111⟩: 0.2079+0.1389𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mctadj(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mct_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled T(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1734−0.0345𝑖
        |0000100001⟩: 0.1734−0.0345𝑖
        |0001000010⟩: 0.1734−0.0345𝑖
        |0001100011⟩: 0.1734−0.0345𝑖
        |0010000100⟩: 0.1734−0.0345𝑖
        |0010100101⟩: 0.1734−0.0345𝑖
        |0011000110⟩: 0.1734−0.0345𝑖
        |0011100111⟩: 0.1734−0.0345𝑖
        |0100001000⟩: 0.1734−0.0345𝑖
        |0100101001⟩: 0.1734−0.0345𝑖
        |0101001010⟩: 0.1734−0.0345𝑖
        |0101101011⟩: 0.1734−0.0345𝑖
        |0110001100⟩: 0.1734−0.0345𝑖
        |0110101101⟩: 0.1734−0.0345𝑖
        |0111001110⟩: 0.1734−0.0345𝑖
        |0111101111⟩: 0.1734−0.0345𝑖
        |1000010000⟩: 0.1734−0.0345𝑖
        |1000110001⟩: 0.1734−0.0345𝑖
        |1001010010⟩: 0.1734−0.0345𝑖
        |1001110011⟩: 0.1734−0.0345𝑖
        |1010010100⟩: 0.1734−0.0345𝑖
        |1010110101⟩: 0.1734−0.0345𝑖
        |1011010110⟩: 0.1734−0.0345𝑖
        |1011110111⟩: 0.1734−0.0345𝑖
        |1100011000⟩: 0.1734−0.0345𝑖
        |1100111001⟩: 0.1734−0.0345𝑖
        |1101011010⟩: 0.1734−0.0345𝑖
        |1101111011⟩: 0.1734−0.0345𝑖
        |1110011100⟩: 0.1734−0.0345𝑖
        |1110111101⟩: 0.1734−0.0345𝑖
        |1111011110⟩: 0.1734−0.0345𝑖
        |1111111111⟩: 0.1470+0.0982𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mctadj(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mct_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled T(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1734−0.0345𝑖
        |0000100001⟩: 0.1734−0.0345𝑖
        |0001000010⟩: 0.1734−0.0345𝑖
        |0001100011⟩: 0.1734−0.0345𝑖
        |0010000100⟩: 0.1734−0.0345𝑖
        |0010100101⟩: 0.1734−0.0345𝑖
        |0011000110⟩: 0.1734−0.0345𝑖
        |0011100111⟩: 0.1734−0.0345𝑖
        |0100001000⟩: 0.1734−0.0345𝑖
        |0100101001⟩: 0.1734−0.0345𝑖
        |0101001010⟩: 0.1734−0.0345𝑖
        |0101101011⟩: 0.1734−0.0345𝑖
        |0110001100⟩: 0.1734−0.0345𝑖
        |0110101101⟩: 0.1734−0.0345𝑖
        |0111001110⟩: 0.1734−0.0345𝑖
        |0111101111⟩: 0.1734−0.0345𝑖
        |1000010000⟩: 0.1734−0.0345𝑖
        |1000110001⟩: 0.1734−0.0345𝑖
        |1001010010⟩: 0.1734−0.0345𝑖
        |1001110011⟩: 0.1734−0.0345𝑖
        |1010010100⟩: 0.1734−0.0345𝑖
        |1010110101⟩: 0.1734−0.0345𝑖
        |1011010110⟩: 0.1734−0.0345𝑖
        |1011110111⟩: 0.1734−0.0345𝑖
        |1100011000⟩: 0.1734−0.0345𝑖
        |1100111001⟩: 0.1734−0.0345𝑖
        |1101011010⟩: 0.1734−0.0345𝑖
        |1101111011⟩: 0.1734−0.0345𝑖
        |1110011100⟩: 0.1734−0.0345𝑖
        |1110111101⟩: 0.1734−0.0345𝑖
        |1111011110⟩: 0.1734−0.0345𝑖
        |1111111111⟩: 0.1470+0.0982𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mctadj(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_mctadj_1_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(2);
            let aux = QIR.Runtime.AllocateQubitArray(2);
            for i in 0..1 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Adjoint T(qs[0..0], qs[1]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000⟩: 0.4904+0.0975𝑖
        |0101⟩: 0.4904+0.0975𝑖
        |1010⟩: 0.4904+0.0975𝑖
        |1111⟩: 0.4157−0.2778𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mct(&[0], 1);
    for i in 0..2 {
        sim.sim.mcx(&[i + 2], i);
        sim.sim.h(i + 2);
        assert!(sim.sim.qubit_is_zero(i + 2), "qubit {} is not zero", i + 2);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_mctadj_2_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(3);
            let aux = QIR.Runtime.AllocateQubitArray(3);
            for i in 0..2 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Adjoint T(qs[0..1], qs[2]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |000000⟩: 0.3468+0.0690𝑖
        |001001⟩: 0.3468+0.0690𝑖
        |010010⟩: 0.3468+0.0690𝑖
        |011011⟩: 0.3468+0.0690𝑖
        |100100⟩: 0.3468+0.0690𝑖
        |101101⟩: 0.3468+0.0690𝑖
        |110110⟩: 0.3468+0.0690𝑖
        |111111⟩: 0.2940−0.1964𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mct(&[0, 1], 2);
    for i in 0..3 {
        sim.sim.mcx(&[i + 3], i);
        sim.sim.h(i + 3);
        assert!(sim.sim.qubit_is_zero(i + 3), "qubit {} is not zero", i + 3);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mctadj_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Adjoint T(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2452+0.0488𝑖
        |00010001⟩: 0.2452+0.0488𝑖
        |00100010⟩: 0.2452+0.0488𝑖
        |00110011⟩: 0.2452+0.0488𝑖
        |01000100⟩: 0.2452+0.0488𝑖
        |01010101⟩: 0.2452+0.0488𝑖
        |01100110⟩: 0.2452+0.0488𝑖
        |01110111⟩: 0.2452+0.0488𝑖
        |10001000⟩: 0.2452+0.0488𝑖
        |10011001⟩: 0.2452+0.0488𝑖
        |10101010⟩: 0.2452+0.0488𝑖
        |10111011⟩: 0.2452+0.0488𝑖
        |11001100⟩: 0.2452+0.0488𝑖
        |11011101⟩: 0.2452+0.0488𝑖
        |11101110⟩: 0.2452+0.0488𝑖
        |11111111⟩: 0.2079−0.1389𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mct(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mctadj_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Adjoint T(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2452+0.0488𝑖
        |00010001⟩: 0.2452+0.0488𝑖
        |00100010⟩: 0.2452+0.0488𝑖
        |00110011⟩: 0.2452+0.0488𝑖
        |01000100⟩: 0.2452+0.0488𝑖
        |01010101⟩: 0.2452+0.0488𝑖
        |01100110⟩: 0.2452+0.0488𝑖
        |01110111⟩: 0.2452+0.0488𝑖
        |10001000⟩: 0.2452+0.0488𝑖
        |10011001⟩: 0.2452+0.0488𝑖
        |10101010⟩: 0.2452+0.0488𝑖
        |10111011⟩: 0.2452+0.0488𝑖
        |11001100⟩: 0.2452+0.0488𝑖
        |11011101⟩: 0.2452+0.0488𝑖
        |11101110⟩: 0.2452+0.0488𝑖
        |11111111⟩: 0.2079−0.1389𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mct(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mctadj_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Adjoint T(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1734+0.0345𝑖
        |0000100001⟩: 0.1734+0.0345𝑖
        |0001000010⟩: 0.1734+0.0345𝑖
        |0001100011⟩: 0.1734+0.0345𝑖
        |0010000100⟩: 0.1734+0.0345𝑖
        |0010100101⟩: 0.1734+0.0345𝑖
        |0011000110⟩: 0.1734+0.0345𝑖
        |0011100111⟩: 0.1734+0.0345𝑖
        |0100001000⟩: 0.1734+0.0345𝑖
        |0100101001⟩: 0.1734+0.0345𝑖
        |0101001010⟩: 0.1734+0.0345𝑖
        |0101101011⟩: 0.1734+0.0345𝑖
        |0110001100⟩: 0.1734+0.0345𝑖
        |0110101101⟩: 0.1734+0.0345𝑖
        |0111001110⟩: 0.1734+0.0345𝑖
        |0111101111⟩: 0.1734+0.0345𝑖
        |1000010000⟩: 0.1734+0.0345𝑖
        |1000110001⟩: 0.1734+0.0345𝑖
        |1001010010⟩: 0.1734+0.0345𝑖
        |1001110011⟩: 0.1734+0.0345𝑖
        |1010010100⟩: 0.1734+0.0345𝑖
        |1010110101⟩: 0.1734+0.0345𝑖
        |1011010110⟩: 0.1734+0.0345𝑖
        |1011110111⟩: 0.1734+0.0345𝑖
        |1100011000⟩: 0.1734+0.0345𝑖
        |1100111001⟩: 0.1734+0.0345𝑖
        |1101011010⟩: 0.1734+0.0345𝑖
        |1101111011⟩: 0.1734+0.0345𝑖
        |1110011100⟩: 0.1734+0.0345𝑖
        |1110111101⟩: 0.1734+0.0345𝑖
        |1111011110⟩: 0.1734+0.0345𝑖
        |1111111111⟩: 0.1470−0.0982𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mct(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mctadj_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Adjoint T(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1734+0.0345𝑖
        |0000100001⟩: 0.1734+0.0345𝑖
        |0001000010⟩: 0.1734+0.0345𝑖
        |0001100011⟩: 0.1734+0.0345𝑖
        |0010000100⟩: 0.1734+0.0345𝑖
        |0010100101⟩: 0.1734+0.0345𝑖
        |0011000110⟩: 0.1734+0.0345𝑖
        |0011100111⟩: 0.1734+0.0345𝑖
        |0100001000⟩: 0.1734+0.0345𝑖
        |0100101001⟩: 0.1734+0.0345𝑖
        |0101001010⟩: 0.1734+0.0345𝑖
        |0101101011⟩: 0.1734+0.0345𝑖
        |0110001100⟩: 0.1734+0.0345𝑖
        |0110101101⟩: 0.1734+0.0345𝑖
        |0111001110⟩: 0.1734+0.0345𝑖
        |0111101111⟩: 0.1734+0.0345𝑖
        |1000010000⟩: 0.1734+0.0345𝑖
        |1000110001⟩: 0.1734+0.0345𝑖
        |1001010010⟩: 0.1734+0.0345𝑖
        |1001110011⟩: 0.1734+0.0345𝑖
        |1010010100⟩: 0.1734+0.0345𝑖
        |1010110101⟩: 0.1734+0.0345𝑖
        |1011010110⟩: 0.1734+0.0345𝑖
        |1011110111⟩: 0.1734+0.0345𝑖
        |1100011000⟩: 0.1734+0.0345𝑖
        |1100111001⟩: 0.1734+0.0345𝑖
        |1101011010⟩: 0.1734+0.0345𝑖
        |1101111011⟩: 0.1734+0.0345𝑖
        |1110011100⟩: 0.1734+0.0345𝑖
        |1110111101⟩: 0.1734+0.0345𝑖
        |1111011110⟩: 0.1734+0.0345𝑖
        |1111111111⟩: 0.1470−0.0982𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mct(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mcx_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled X(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2500+0.0000𝑖
        |00010001⟩: 0.2500+0.0000𝑖
        |00100010⟩: 0.2500+0.0000𝑖
        |00110011⟩: 0.2500+0.0000𝑖
        |01000100⟩: 0.2500+0.0000𝑖
        |01010101⟩: 0.2500+0.0000𝑖
        |01100110⟩: 0.2500+0.0000𝑖
        |01110111⟩: 0.2500+0.0000𝑖
        |10001000⟩: 0.2500+0.0000𝑖
        |10011001⟩: 0.2500+0.0000𝑖
        |10101010⟩: 0.2500+0.0000𝑖
        |10111011⟩: 0.2500+0.0000𝑖
        |11001100⟩: 0.2500+0.0000𝑖
        |11011101⟩: 0.2500+0.0000𝑖
        |11101111⟩: 0.2500+0.0000𝑖
        |11111110⟩: 0.2500+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcx(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mcx_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled X(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2500+0.0000𝑖
        |00010001⟩: 0.2500+0.0000𝑖
        |00100010⟩: 0.2500+0.0000𝑖
        |00110011⟩: 0.2500+0.0000𝑖
        |01000100⟩: 0.2500+0.0000𝑖
        |01010101⟩: 0.2500+0.0000𝑖
        |01100110⟩: 0.2500+0.0000𝑖
        |01110111⟩: 0.2500+0.0000𝑖
        |10001000⟩: 0.2500+0.0000𝑖
        |10011001⟩: 0.2500+0.0000𝑖
        |10101010⟩: 0.2500+0.0000𝑖
        |10111011⟩: 0.2500+0.0000𝑖
        |11001100⟩: 0.2500+0.0000𝑖
        |11011101⟩: 0.2500+0.0000𝑖
        |11101111⟩: 0.2500+0.0000𝑖
        |11111110⟩: 0.2500+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcx(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mcx_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled X(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1768+0.0000𝑖
        |0000100001⟩: 0.1768+0.0000𝑖
        |0001000010⟩: 0.1768+0.0000𝑖
        |0001100011⟩: 0.1768+0.0000𝑖
        |0010000100⟩: 0.1768+0.0000𝑖
        |0010100101⟩: 0.1768+0.0000𝑖
        |0011000110⟩: 0.1768+0.0000𝑖
        |0011100111⟩: 0.1768+0.0000𝑖
        |0100001000⟩: 0.1768+0.0000𝑖
        |0100101001⟩: 0.1768+0.0000𝑖
        |0101001010⟩: 0.1768+0.0000𝑖
        |0101101011⟩: 0.1768+0.0000𝑖
        |0110001100⟩: 0.1768+0.0000𝑖
        |0110101101⟩: 0.1768+0.0000𝑖
        |0111001110⟩: 0.1768+0.0000𝑖
        |0111101111⟩: 0.1768+0.0000𝑖
        |1000010000⟩: 0.1768+0.0000𝑖
        |1000110001⟩: 0.1768+0.0000𝑖
        |1001010010⟩: 0.1768+0.0000𝑖
        |1001110011⟩: 0.1768+0.0000𝑖
        |1010010100⟩: 0.1768+0.0000𝑖
        |1010110101⟩: 0.1768+0.0000𝑖
        |1011010110⟩: 0.1768+0.0000𝑖
        |1011110111⟩: 0.1768+0.0000𝑖
        |1100011000⟩: 0.1768+0.0000𝑖
        |1100111001⟩: 0.1768+0.0000𝑖
        |1101011010⟩: 0.1768+0.0000𝑖
        |1101111011⟩: 0.1768+0.0000𝑖
        |1110011100⟩: 0.1768+0.0000𝑖
        |1110111101⟩: 0.1768+0.0000𝑖
        |1111011111⟩: 0.1768+0.0000𝑖
        |1111111110⟩: 0.1768+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcx(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mcx_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled X(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1768+0.0000𝑖
        |0000100001⟩: 0.1768+0.0000𝑖
        |0001000010⟩: 0.1768+0.0000𝑖
        |0001100011⟩: 0.1768+0.0000𝑖
        |0010000100⟩: 0.1768+0.0000𝑖
        |0010100101⟩: 0.1768+0.0000𝑖
        |0011000110⟩: 0.1768+0.0000𝑖
        |0011100111⟩: 0.1768+0.0000𝑖
        |0100001000⟩: 0.1768+0.0000𝑖
        |0100101001⟩: 0.1768+0.0000𝑖
        |0101001010⟩: 0.1768+0.0000𝑖
        |0101101011⟩: 0.1768+0.0000𝑖
        |0110001100⟩: 0.1768+0.0000𝑖
        |0110101101⟩: 0.1768+0.0000𝑖
        |0111001110⟩: 0.1768+0.0000𝑖
        |0111101111⟩: 0.1768+0.0000𝑖
        |1000010000⟩: 0.1768+0.0000𝑖
        |1000110001⟩: 0.1768+0.0000𝑖
        |1001010010⟩: 0.1768+0.0000𝑖
        |1001110011⟩: 0.1768+0.0000𝑖
        |1010010100⟩: 0.1768+0.0000𝑖
        |1010110101⟩: 0.1768+0.0000𝑖
        |1011010110⟩: 0.1768+0.0000𝑖
        |1011110111⟩: 0.1768+0.0000𝑖
        |1100011000⟩: 0.1768+0.0000𝑖
        |1100111001⟩: 0.1768+0.0000𝑖
        |1101011010⟩: 0.1768+0.0000𝑖
        |1101111011⟩: 0.1768+0.0000𝑖
        |1110011100⟩: 0.1768+0.0000𝑖
        |1110111101⟩: 0.1768+0.0000𝑖
        |1111011111⟩: 0.1768+0.0000𝑖
        |1111111110⟩: 0.1768+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcx(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mcy_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Y(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2500+0.0000𝑖
        |00010001⟩: 0.2500+0.0000𝑖
        |00100010⟩: 0.2500+0.0000𝑖
        |00110011⟩: 0.2500+0.0000𝑖
        |01000100⟩: 0.2500+0.0000𝑖
        |01010101⟩: 0.2500+0.0000𝑖
        |01100110⟩: 0.2500+0.0000𝑖
        |01110111⟩: 0.2500+0.0000𝑖
        |10001000⟩: 0.2500+0.0000𝑖
        |10011001⟩: 0.2500+0.0000𝑖
        |10101010⟩: 0.2500+0.0000𝑖
        |10111011⟩: 0.2500+0.0000𝑖
        |11001100⟩: 0.2500+0.0000𝑖
        |11011101⟩: 0.2500+0.0000𝑖
        |11101111⟩: 0.0000−0.2500𝑖
        |11111110⟩: 0.0000+0.2500𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcy(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mcy_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Y(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2500+0.0000𝑖
        |00010001⟩: 0.2500+0.0000𝑖
        |00100010⟩: 0.2500+0.0000𝑖
        |00110011⟩: 0.2500+0.0000𝑖
        |01000100⟩: 0.2500+0.0000𝑖
        |01010101⟩: 0.2500+0.0000𝑖
        |01100110⟩: 0.2500+0.0000𝑖
        |01110111⟩: 0.2500+0.0000𝑖
        |10001000⟩: 0.2500+0.0000𝑖
        |10011001⟩: 0.2500+0.0000𝑖
        |10101010⟩: 0.2500+0.0000𝑖
        |10111011⟩: 0.2500+0.0000𝑖
        |11001100⟩: 0.2500+0.0000𝑖
        |11011101⟩: 0.2500+0.0000𝑖
        |11101111⟩: 0.0000−0.2500𝑖
        |11111110⟩: 0.0000+0.2500𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcy(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mcy_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Y(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1768+0.0000𝑖
        |0000100001⟩: 0.1768+0.0000𝑖
        |0001000010⟩: 0.1768+0.0000𝑖
        |0001100011⟩: 0.1768+0.0000𝑖
        |0010000100⟩: 0.1768+0.0000𝑖
        |0010100101⟩: 0.1768+0.0000𝑖
        |0011000110⟩: 0.1768+0.0000𝑖
        |0011100111⟩: 0.1768+0.0000𝑖
        |0100001000⟩: 0.1768+0.0000𝑖
        |0100101001⟩: 0.1768+0.0000𝑖
        |0101001010⟩: 0.1768+0.0000𝑖
        |0101101011⟩: 0.1768+0.0000𝑖
        |0110001100⟩: 0.1768+0.0000𝑖
        |0110101101⟩: 0.1768+0.0000𝑖
        |0111001110⟩: 0.1768+0.0000𝑖
        |0111101111⟩: 0.1768+0.0000𝑖
        |1000010000⟩: 0.1768+0.0000𝑖
        |1000110001⟩: 0.1768+0.0000𝑖
        |1001010010⟩: 0.1768+0.0000𝑖
        |1001110011⟩: 0.1768+0.0000𝑖
        |1010010100⟩: 0.1768+0.0000𝑖
        |1010110101⟩: 0.1768+0.0000𝑖
        |1011010110⟩: 0.1768+0.0000𝑖
        |1011110111⟩: 0.1768+0.0000𝑖
        |1100011000⟩: 0.1768+0.0000𝑖
        |1100111001⟩: 0.1768+0.0000𝑖
        |1101011010⟩: 0.1768+0.0000𝑖
        |1101111011⟩: 0.1768+0.0000𝑖
        |1110011100⟩: 0.1768+0.0000𝑖
        |1110111101⟩: 0.1768+0.0000𝑖
        |1111011111⟩: 0.0000−0.1768𝑖
        |1111111110⟩: 0.0000+0.1768𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcy(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mcy_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Y(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1768+0.0000𝑖
        |0000100001⟩: 0.1768+0.0000𝑖
        |0001000010⟩: 0.1768+0.0000𝑖
        |0001100011⟩: 0.1768+0.0000𝑖
        |0010000100⟩: 0.1768+0.0000𝑖
        |0010100101⟩: 0.1768+0.0000𝑖
        |0011000110⟩: 0.1768+0.0000𝑖
        |0011100111⟩: 0.1768+0.0000𝑖
        |0100001000⟩: 0.1768+0.0000𝑖
        |0100101001⟩: 0.1768+0.0000𝑖
        |0101001010⟩: 0.1768+0.0000𝑖
        |0101101011⟩: 0.1768+0.0000𝑖
        |0110001100⟩: 0.1768+0.0000𝑖
        |0110101101⟩: 0.1768+0.0000𝑖
        |0111001110⟩: 0.1768+0.0000𝑖
        |0111101111⟩: 0.1768+0.0000𝑖
        |1000010000⟩: 0.1768+0.0000𝑖
        |1000110001⟩: 0.1768+0.0000𝑖
        |1001010010⟩: 0.1768+0.0000𝑖
        |1001110011⟩: 0.1768+0.0000𝑖
        |1010010100⟩: 0.1768+0.0000𝑖
        |1010110101⟩: 0.1768+0.0000𝑖
        |1011010110⟩: 0.1768+0.0000𝑖
        |1011110111⟩: 0.1768+0.0000𝑖
        |1100011000⟩: 0.1768+0.0000𝑖
        |1100111001⟩: 0.1768+0.0000𝑖
        |1101011010⟩: 0.1768+0.0000𝑖
        |1101111011⟩: 0.1768+0.0000𝑖
        |1110011100⟩: 0.1768+0.0000𝑖
        |1110111101⟩: 0.1768+0.0000𝑖
        |1111011111⟩: 0.0000−0.1768𝑖
        |1111111110⟩: 0.0000+0.1768𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcy(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mcz_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Z(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2500+0.0000𝑖
        |00010001⟩: 0.2500+0.0000𝑖
        |00100010⟩: 0.2500+0.0000𝑖
        |00110011⟩: 0.2500+0.0000𝑖
        |01000100⟩: 0.2500+0.0000𝑖
        |01010101⟩: 0.2500+0.0000𝑖
        |01100110⟩: 0.2500+0.0000𝑖
        |01110111⟩: 0.2500+0.0000𝑖
        |10001000⟩: 0.2500+0.0000𝑖
        |10011001⟩: 0.2500+0.0000𝑖
        |10101010⟩: 0.2500+0.0000𝑖
        |10111011⟩: 0.2500+0.0000𝑖
        |11001100⟩: 0.2500+0.0000𝑖
        |11011101⟩: 0.2500+0.0000𝑖
        |11101110⟩: 0.2500+0.0000𝑖
        |11111111⟩: −0.2500+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcz(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mcz_3_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(4);
            let aux = QIR.Runtime.AllocateQubitArray(4);
            for i in 0..3 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Z(qs[0..2], qs[3]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |00000000⟩: 0.2500+0.0000𝑖
        |00010001⟩: 0.2500+0.0000𝑖
        |00100010⟩: 0.2500+0.0000𝑖
        |00110011⟩: 0.2500+0.0000𝑖
        |01000100⟩: 0.2500+0.0000𝑖
        |01010101⟩: 0.2500+0.0000𝑖
        |01100110⟩: 0.2500+0.0000𝑖
        |01110111⟩: 0.2500+0.0000𝑖
        |10001000⟩: 0.2500+0.0000𝑖
        |10011001⟩: 0.2500+0.0000𝑖
        |10101010⟩: 0.2500+0.0000𝑖
        |10111011⟩: 0.2500+0.0000𝑖
        |11001100⟩: 0.2500+0.0000𝑖
        |11011101⟩: 0.2500+0.0000𝑖
        |11101110⟩: 0.2500+0.0000𝑖
        |11111111⟩: −0.2500+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcz(&[0, 1, 2], 3);
    for i in 0..4 {
        sim.sim.mcx(&[i + 4], i);
        sim.sim.h(i + 4);
        assert!(sim.sim.qubit_is_zero(i + 4), "qubit {} is not zero", i + 4);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_unrestricted_mcz_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Z(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        "",
        Profile::Unrestricted,
        &mut sim,
        &Value::unit(),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1768+0.0000𝑖
        |0000100001⟩: 0.1768+0.0000𝑖
        |0001000010⟩: 0.1768+0.0000𝑖
        |0001100011⟩: 0.1768+0.0000𝑖
        |0010000100⟩: 0.1768+0.0000𝑖
        |0010100101⟩: 0.1768+0.0000𝑖
        |0011000110⟩: 0.1768+0.0000𝑖
        |0011100111⟩: 0.1768+0.0000𝑖
        |0100001000⟩: 0.1768+0.0000𝑖
        |0100101001⟩: 0.1768+0.0000𝑖
        |0101001010⟩: 0.1768+0.0000𝑖
        |0101101011⟩: 0.1768+0.0000𝑖
        |0110001100⟩: 0.1768+0.0000𝑖
        |0110101101⟩: 0.1768+0.0000𝑖
        |0111001110⟩: 0.1768+0.0000𝑖
        |0111101111⟩: 0.1768+0.0000𝑖
        |1000010000⟩: 0.1768+0.0000𝑖
        |1000110001⟩: 0.1768+0.0000𝑖
        |1001010010⟩: 0.1768+0.0000𝑖
        |1001110011⟩: 0.1768+0.0000𝑖
        |1010010100⟩: 0.1768+0.0000𝑖
        |1010110101⟩: 0.1768+0.0000𝑖
        |1011010110⟩: 0.1768+0.0000𝑖
        |1011110111⟩: 0.1768+0.0000𝑖
        |1100011000⟩: 0.1768+0.0000𝑖
        |1100111001⟩: 0.1768+0.0000𝑖
        |1101011010⟩: 0.1768+0.0000𝑖
        |1101111011⟩: 0.1768+0.0000𝑖
        |1110011100⟩: 0.1768+0.0000𝑖
        |1110111101⟩: 0.1768+0.0000𝑖
        |1111011110⟩: 0.1768+0.0000𝑖
        |1111111111⟩: −0.1768+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcz(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}

#[test]
fn test_base_mcz_4_control() {
    let mut sim = SparseSim::default();
    let dump = test_expression_with_lib_and_profile_and_sim(
        indoc! {"{
            let qs = QIR.Runtime.AllocateQubitArray(5);
            let aux = QIR.Runtime.AllocateQubitArray(5);
            for i in 0..4 {
                H(aux[i]);
                CNOT(aux[i], qs[i]);
            }
            Controlled Z(qs[0..3], qs[4]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            let result : Result[] = [];
            result
        }"},
        "",
        Profile::Base,
        &mut sim,
        &Value::Array(Vec::new().into()),
    );
    expect![[r#"
        STATE:
        |0000000000⟩: 0.1768+0.0000𝑖
        |0000100001⟩: 0.1768+0.0000𝑖
        |0001000010⟩: 0.1768+0.0000𝑖
        |0001100011⟩: 0.1768+0.0000𝑖
        |0010000100⟩: 0.1768+0.0000𝑖
        |0010100101⟩: 0.1768+0.0000𝑖
        |0011000110⟩: 0.1768+0.0000𝑖
        |0011100111⟩: 0.1768+0.0000𝑖
        |0100001000⟩: 0.1768+0.0000𝑖
        |0100101001⟩: 0.1768+0.0000𝑖
        |0101001010⟩: 0.1768+0.0000𝑖
        |0101101011⟩: 0.1768+0.0000𝑖
        |0110001100⟩: 0.1768+0.0000𝑖
        |0110101101⟩: 0.1768+0.0000𝑖
        |0111001110⟩: 0.1768+0.0000𝑖
        |0111101111⟩: 0.1768+0.0000𝑖
        |1000010000⟩: 0.1768+0.0000𝑖
        |1000110001⟩: 0.1768+0.0000𝑖
        |1001010010⟩: 0.1768+0.0000𝑖
        |1001110011⟩: 0.1768+0.0000𝑖
        |1010010100⟩: 0.1768+0.0000𝑖
        |1010110101⟩: 0.1768+0.0000𝑖
        |1011010110⟩: 0.1768+0.0000𝑖
        |1011110111⟩: 0.1768+0.0000𝑖
        |1100011000⟩: 0.1768+0.0000𝑖
        |1100111001⟩: 0.1768+0.0000𝑖
        |1101011010⟩: 0.1768+0.0000𝑖
        |1101111011⟩: 0.1768+0.0000𝑖
        |1110011100⟩: 0.1768+0.0000𝑖
        |1110111101⟩: 0.1768+0.0000𝑖
        |1111011110⟩: 0.1768+0.0000𝑖
        |1111111111⟩: −0.1768+0.0000𝑖
    "#]]
    .assert_eq(&dump);

    sim.sim.mcz(&[0, 1, 2, 3], 4);
    for i in 0..5 {
        sim.sim.mcx(&[i + 5], i);
        sim.sim.h(i + 5);
        assert!(sim.sim.qubit_is_zero(i + 5), "qubit {} is not zero", i + 5);
        assert!(sim.sim.qubit_is_zero(i), "qubit {i} is not zero");
    }
}
