// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::test_expression;
use super::test_expression_with_lib;
use expect_test::expect;
use qsc::interpret::Value;

// Tests for Microsoft.Quantum.StatePreparation namespace

const STATE_PREPARATION_TEST_LIB: &str = include_str!("resources/state_preparation.qs");

#[test]
fn check_plus_state_preparation() {
    let out = test_expression_with_lib(
        "Test.TestPlusState()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );

    expect![[r#"
        STATE:
        |0⟩: 0.7071+0.0000𝑖
        |1⟩: 0.7071+0.0000𝑖
    "#]]
    .assert_eq(&out);
}

#[test]
fn check_minus_state_preparation() {
    let out = test_expression_with_lib(
        "Test.TestMinusState()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );

    expect![[r#"
        STATE:
        |0⟩: 0.0000−0.7071𝑖
        |1⟩: 0.0000+0.7071𝑖
    "#]]
    .assert_eq(&out);
}

#[test]
fn check_bell_state_preparation() {
    let out = test_expression_with_lib(
        "Test.TestBellState()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );

    expect![[r#"
        STATE:
        |00⟩: 0.7071+0.0000𝑖
        |11⟩: 0.7071+0.0000𝑖
    "#]]
    .assert_eq(&out);
}

#[test]
fn check_cat_state_preparation() {
    let out = test_expression_with_lib(
        "Test.TestCat3State()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );

    expect![[r#"
        STATE:
        |000⟩: 0.7071+0.0000𝑖
        |111⟩: 0.7071+0.0000𝑖
    "#]]
    .assert_eq(&out);
}

#[test]
fn check_complex_preparation() {
    let out = test_expression_with_lib(
        "Test.TestPrepareComplex()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );

    expect![[r#"
        STATE:
        |00⟩: 0.1913−0.4619𝑖
        |01⟩: 0.4619−0.1913𝑖
        |10⟩: 0.4619+0.1913𝑖
        |11⟩: 0.1913+0.4619𝑖
    "#]]
    .assert_eq(&out);
}

#[test]
fn check_preparation_completion() {
    let out = test_expression_with_lib(
        "Test.TestPreparationCompletion()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );

    expect![[r#"
        STATE:
        |0⟩: 0.7738+0.0000𝑖
        |1⟩: 0.6335+0.0000𝑖
        STATE:
        |00⟩: 0.1830+0.0000𝑖
        |01⟩: 0.4070+0.0000𝑖
        |10⟩: 0.6049+0.0000𝑖
        |11⟩: 0.6595+0.0000𝑖
        STATE:
        |000⟩: 0.0987+0.0000𝑖
        |001⟩: 0.3590+0.0000𝑖
        |010⟩: 0.4657+0.0000𝑖
        |011⟩: 0.4674+0.0000𝑖
        |100⟩: 0.4199+0.0000𝑖
        |101⟩: 0.1184+0.0000𝑖
        |110⟩: 0.4619+0.0000𝑖
        |111⟩: 0.1496+0.0000𝑖
        STATE:
        |0000⟩: 0.2715+0.0000𝑖
        |0001⟩: 0.0584+0.0000𝑖
        |0010⟩: 0.1164+0.0000𝑖
        |0011⟩: 0.3611+0.0000𝑖
        |0100⟩: 0.3074+0.0000𝑖
        |0101⟩: 0.1934+0.0000𝑖
        |0110⟩: 0.2742+0.0000𝑖
        |0111⟩: 0.3325+0.0000𝑖
        |1000⟩: 0.1302+0.0000𝑖
        |1001⟩: 0.2225+0.0000𝑖
        |1010⟩: 0.3149+0.0000𝑖
        |1011⟩: 0.2107+0.0000𝑖
        |1100⟩: 0.2124+0.0000𝑖
        |1101⟩: 0.2455+0.0000𝑖
        |1110⟩: 0.3067+0.0000𝑖
        |1111⟩: 0.2277+0.0000𝑖
        STATE:
        |0⟩: 0.0000+0.7738𝑖
        |1⟩: 0.0000−0.6335𝑖
        STATE:
        |00⟩: 0.1294−0.1294𝑖
        |01⟩: −0.2878+0.2878𝑖
        |10⟩: 0.4277−0.4277𝑖
        |11⟩: 0.4663−0.4663𝑖
        STATE:
        |000⟩: 0.0378−0.0911𝑖
        |001⟩: −0.1374+0.3317𝑖
        |010⟩: 0.1782−0.4302𝑖
        |011⟩: −0.1789+0.4318𝑖
        |100⟩: 0.1607−0.3879𝑖
        |101⟩: 0.0453−0.1094𝑖
        |110⟩: −0.1768+0.4267𝑖
        |111⟩: 0.0573−0.1382𝑖
        STATE:
        |0000⟩: −0.1039+0.2508𝑖
        |0001⟩: 0.0223−0.0539𝑖
        |0010⟩: 0.0445−0.1075𝑖
        |0011⟩: 0.1382−0.3336𝑖
        |0100⟩: −0.1176+0.2840𝑖
        |0101⟩: 0.0740−0.1787𝑖
        |0110⟩: −0.1049+0.2533𝑖
        |0111⟩: 0.1273−0.3072𝑖
        |1000⟩: 0.0498−0.1203𝑖
        |1001⟩: 0.0852−0.2056𝑖
        |1010⟩: 0.1205−0.2909𝑖
        |1011⟩: −0.0806+0.1947𝑖
        |1100⟩: 0.0813−0.1963𝑖
        |1101⟩: 0.0940−0.2268𝑖
        |1110⟩: −0.1174+0.2833𝑖
        |1111⟩: −0.0871+0.2104𝑖
        STATE:
        |000⟩: 0.6847−0.2836𝑖
        |001⟩: 0.2238−0.0927𝑖
        |010⟩: 0.2902−0.1202𝑖
        |011⟩: −0.2913+0.1207𝑖
        |100⟩: 0.2617−0.1084𝑖
        |101⟩: 0.0738−0.0306𝑖
        |110⟩: 0.2879−0.1192𝑖
        |111⟩: 0.0932−0.0386𝑖
        STATE:
        |000⟩: 0.7247−0.3002𝑖
        |001⟩: 0.2368−0.0981𝑖
        |010⟩: 0.3072−0.1272𝑖
        |011⟩: −0.3083+0.1277𝑖
        |100⟩: 0.2770−0.1147𝑖
        |101⟩: 0.0781−0.0324𝑖
    "#]]
    .assert_eq(&out);
}

#[test]
fn check_preparation_endianness() {
    let out = test_expression_with_lib(
        "Test.TestEndianness()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );

    expect![[r#"
        STATE:
        |0000⟩: 1.0000+0.0000𝑖
        STATE:
        |0001⟩: 1.0000+0.0000𝑖
        STATE:
        |0010⟩: 1.0000+0.0000𝑖
        STATE:
        |0011⟩: 1.0000+0.0000𝑖
        STATE:
        |0100⟩: 1.0000+0.0000𝑖
        STATE:
        |0101⟩: 1.0000+0.0000𝑖
        STATE:
        |0110⟩: 1.0000+0.0000𝑖
        STATE:
        |0111⟩: 1.0000+0.0000𝑖
        STATE:
        |1000⟩: 1.0000+0.0000𝑖
        STATE:
        |1001⟩: 1.0000+0.0000𝑖
        STATE:
        |1010⟩: 1.0000+0.0000𝑖
        STATE:
        |1011⟩: 1.0000+0.0000𝑖
        STATE:
        |1100⟩: 1.0000+0.0000𝑖
        STATE:
        |1101⟩: 1.0000+0.0000𝑖
        STATE:
        |1110⟩: 1.0000+0.0000𝑖
        STATE:
        |1111⟩: 1.0000+0.0000𝑖
    "#]]
    .assert_eq(&out);
}

#[test]
fn check_preparation_doc_sample() {
    let out = test_expression(
        "{
        open Microsoft.Quantum.Math;
        let amplitudes = [Sqrt(0.125), 0.0, Sqrt(0.875), 0.0];
        use qubits = Qubit[2];
        Microsoft.Quantum.Unstable.StatePreparation.PreparePureStateD(amplitudes, qubits);
        Microsoft.Quantum.Diagnostics.DumpMachine();
        ResetAll(qubits); }",
        &Value::Tuple(vec![].into()),
    );

    expect![[r#"
        STATE:
        |00⟩: 0.3536+0.0000𝑖
        |10⟩: 0.9354+0.0000𝑖
    "#]]
    .assert_eq(&out);
}
