// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use expect_test::{expect, Expect};
use qsc::{
    compile,
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
    // when we load the project, need to set these
    let (std_id, store) = compile::package_store_with_stdlib(TargetCapabilityFlags::all());

    let mut interpreter = match (if debug {
        Interpreter::new_with_debug
    } else {
        Interpreter::new
    })(
        sources,
        PackageType::Exe,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
        store,
        &[(std_id, None)],
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

include!(concat!(env!("OUT_DIR"), "/test_cases.rs"));

/// Each file in the samples/algorithms folder is compiled and run as two tests and should
/// have matching expect strings in this file. If new samples are added, this file will
/// fail to compile until the new expect strings are added.
const BELLSTATE_EXPECT: Expect = expect![[r#"
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
    [(Zero, Zero), (One, One), (One, Zero), (One, Zero)]"#]];
const BELLSTATE_EXPECT_DEBUG: Expect = expect![[r#"
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
    [(Zero, Zero), (One, One), (One, Zero), (One, Zero)]"#]];
const BERNSTEINVAZIRANI_EXPECT: Expect = expect![[r#"
    Successfully decoded bit string as int: 127
    Successfully decoded bit string as int: 238
    Successfully decoded bit string as int: 512
    [127, 238, 512]"#]];
const BERNSTEINVAZIRANI_EXPECT_DEBUG: Expect = expect![[r#"
    Successfully decoded bit string as int: 127
    Successfully decoded bit string as int: 238
    Successfully decoded bit string as int: 512
    [127, 238, 512]"#]];
const BERNSTEINVAZIRANINISQ_EXPECT: Expect = expect!["[One, Zero, One, Zero, One]"];
const BERNSTEINVAZIRANINISQ_EXPECT_DEBUG: Expect = expect!["[One, Zero, One, Zero, One]"];
const BITFLIPCODE_EXPECT: Expect = expect![[r#"
    STATE:
    |001⟩: 0.4472+0.0000𝑖
    |110⟩: 0.8944+0.0000𝑖
    STATE:
    |000⟩: 0.4472+0.0000𝑖
    |111⟩: 0.8944+0.0000𝑖
    One"#]];
const BITFLIPCODE_EXPECT_DEBUG: Expect = expect![[r#"
    STATE:
    |001⟩: 0.4472+0.0000𝑖
    |110⟩: 0.8944+0.0000𝑖
    STATE:
    |000⟩: 0.4472+0.0000𝑖
    |111⟩: 0.8944+0.0000𝑖
    One"#]];
const CATSTATE_EXPECT: Expect = expect![[r#"
    STATE:
    |00000⟩: 0.7071+0.0000𝑖
    |11111⟩: 0.7071+0.0000𝑖
    [Zero, Zero, Zero, Zero, Zero]"#]];
const CATSTATE_EXPECT_DEBUG: Expect = expect![[r#"
    STATE:
    |00000⟩: 0.7071+0.0000𝑖
    |11111⟩: 0.7071+0.0000𝑖
    [Zero, Zero, Zero, Zero, Zero]"#]];
const DEUTSCHJOZSA_EXPECT: Expect = expect![[r#"
    SimpleConstantBoolF is constant
    SimpleBalancedBoolF is balanced
    ConstantBoolF is constant
    BalancedBoolF is balanced
    [(SimpleConstantBoolF, true), (SimpleBalancedBoolF, false), (ConstantBoolF, true), (BalancedBoolF, false)]"#]];
const DEUTSCHJOZSA_EXPECT_DEBUG: Expect = expect![[r#"
    SimpleConstantBoolF is constant
    SimpleBalancedBoolF is balanced
    ConstantBoolF is constant
    BalancedBoolF is balanced
    [(SimpleConstantBoolF, true), (SimpleBalancedBoolF, false), (ConstantBoolF, true), (BalancedBoolF, false)]"#]];
const DEUTSCHJOZSANISQ_EXPECT: Expect =
    expect!["([One, Zero, Zero, Zero, Zero], [Zero, Zero, Zero, Zero, Zero])"];
const DEUTSCHJOZSANISQ_EXPECT_DEBUG: Expect =
    expect!["([One, Zero, Zero, Zero, Zero], [Zero, Zero, Zero, Zero, Zero])"];
const DOTPRODUCTVIAPHASEESTIMATION_EXPECT: Expect = expect![[r#"
    Computing inner product of vectors (cos(Θ₁/2), sin(Θ₁/2))⋅(cos(Θ₂/2), sin(Θ₂/2)) ≈ -cos(x𝝅/2ⁿ)
    Θ₁=0.4487989505128276, Θ₂=0.6283185307179586.
    x = 16, n = 4.
    Computed value = 1.0, true value = 0.995974293995239
    (16, 4)"#]];
const DOTPRODUCTVIAPHASEESTIMATION_EXPECT_DEBUG: Expect = expect![[r#"
    Computing inner product of vectors (cos(Θ₁/2), sin(Θ₁/2))⋅(cos(Θ₂/2), sin(Θ₂/2)) ≈ -cos(x𝝅/2ⁿ)
    Θ₁=0.4487989505128276, Θ₂=0.6283185307179586.
    x = 16, n = 4.
    Computed value = 1.0, true value = 0.995974293995239
    (16, 4)"#]];
const ENTANGLEMENT_EXPECT: Expect = expect![[r#"
    STATE:
    |00⟩: 0.7071+0.0000𝑖
    |11⟩: 0.7071+0.0000𝑖
    (Zero, Zero)"#]];
const ENTANGLEMENT_EXPECT_DEBUG: Expect = expect![[r#"
    STATE:
    |00⟩: 0.7071+0.0000𝑖
    |11⟩: 0.7071+0.0000𝑖
    (Zero, Zero)"#]];
const GHZ_EXPECT: Expect = expect![[r#"
    STATE:
    |000⟩: 0.7071+0.0000𝑖
    |111⟩: 0.7071+0.0000𝑖
    [Zero, Zero, Zero]"#]];
const GHZ_EXPECT_DEBUG: Expect = expect![[r#"
    STATE:
    |000⟩: 0.7071+0.0000𝑖
    |111⟩: 0.7071+0.0000𝑖
    [Zero, Zero, Zero]"#]];
const GROVER_EXPECT: Expect = expect![[r#"
    Number of iterations: 4
    Reflecting about marked state...
    Reflecting about marked state...
    Reflecting about marked state...
    Reflecting about marked state...
    [Zero, One, Zero, One, Zero]"#]];
const GROVER_EXPECT_DEBUG: Expect = expect![[r#"
    Number of iterations: 4
    Reflecting about marked state...
    Reflecting about marked state...
    Reflecting about marked state...
    Reflecting about marked state...
    [Zero, One, Zero, One, Zero]"#]];
const HIDDENSHIFT_EXPECT: Expect = expect![[r#"
    Found 170 successfully!
    Found 512 successfully!
    Found 999 successfully!
    [170, 512, 999]"#]];
const HIDDENSHIFT_EXPECT_DEBUG: Expect = expect![[r#"
    Found 170 successfully!
    Found 512 successfully!
    Found 999 successfully!
    [170, 512, 999]"#]];
const HIDDENSHIFTNISQ_EXPECT: Expect = expect!["[One, Zero, Zero, Zero, Zero, One]"];
const HIDDENSHIFTNISQ_EXPECT_DEBUG: Expect = expect!["[One, Zero, Zero, Zero, Zero, One]"];
const JOINTMEASUREMENT_EXPECT: Expect = expect![[r#"
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
    (Zero, [One, One])"#]];
const JOINTMEASUREMENT_EXPECT_DEBUG: Expect = expect![[r#"
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
    (Zero, [One, One])"#]];
const MEASUREMENT_EXPECT: Expect = expect!["(Zero, [Zero, Zero])"];
const MEASUREMENT_EXPECT_DEBUG: Expect = expect!["(Zero, [Zero, Zero])"];
const PHASEFLIPCODE_EXPECT: Expect = expect![[r#"
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
    One"#]];
const PHASEFLIPCODE_EXPECT_DEBUG: Expect = expect![[r#"
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
    One"#]];
const QRNG_EXPECT: Expect = expect![[r#"
    Sampling a random number between 0 and 100:
    46"#]];
const QRNG_EXPECT_DEBUG: Expect = expect![[r#"
    Sampling a random number between 0 and 100:
    46"#]];
const QRNGNISQ_EXPECT: Expect = expect!["[Zero, Zero, One, One, One]"];
const QRNGNISQ_EXPECT_DEBUG: Expect = expect!["[Zero, Zero, One, One, One]"];
const QUANTUMHELLOWORLD_EXPECT: Expect = expect![[r#"
    Hello world!
    Zero"#]];
const QUANTUMHELLOWORLD_EXPECT_DEBUG: Expect = expect![[r#"
    Hello world!
    Zero"#]];
const RANDOMBIT_EXPECT: Expect = expect!["Zero"];
const RANDOMBIT_EXPECT_DEBUG: Expect = expect!["Zero"];
const SHOR_EXPECT: Expect = expect![[r#"
    *** Factorizing 143, attempt 1.
    Estimating period of 139.
    Estimating frequency with bitsPrecision=17.
    Estimated frequency=30583
    Found period=30
    Found factor=13
    Found factorization 143 = 13 * 11
    (13, 11)"#]];
const SHOR_EXPECT_DEBUG: Expect = expect![[r#"
    *** Factorizing 143, attempt 1.
    Estimating period of 139.
    Estimating frequency with bitsPrecision=17.
    Estimated frequency=30583
    Found period=30
    Found factor=13
    Found factorization 143 = 13 * 11
    (13, 11)"#]];
const SUPERDENSECODING_EXPECT: Expect = expect!["((false, true), (false, true))"];
const SUPERDENSECODING_EXPECT_DEBUG: Expect = expect!["((false, true), (false, true))"];
const SUPERPOSITION_EXPECT: Expect = expect!["Zero"];
const SUPERPOSITION_EXPECT_DEBUG: Expect = expect!["Zero"];
const TELEPORTATION_EXPECT: Expect = expect![[r#"
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
    [Zero, One, Zero, One]"#]];
const TELEPORTATION_EXPECT_DEBUG: Expect = expect![[r#"
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
    [Zero, One, Zero, One]"#]];
const THREEQUBITREPETITIONCODE_EXPECT: Expect = expect!["(true, 0)"];
const THREEQUBITREPETITIONCODE_EXPECT_DEBUG: Expect = expect!["(true, 0)"];
