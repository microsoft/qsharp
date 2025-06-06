// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};

// Each file in the samples/algorithms folder is compiled and run as two tests and should
// have matching expect strings in this file. If new samples are added, this file will
// fail to compile until the new expect strings are added.
pub const BERNSTEINVAZIRANI_EXPECT: Expect = expect![[r#"
    Successfully decoded bit string as int: 127
    Successfully decoded bit string as int: 238
    Successfully decoded bit string as int: 512
    [127, 238, 512]"#]];
pub const BERNSTEINVAZIRANI_EXPECT_DEBUG: Expect = expect![[r#"
    Successfully decoded bit string as int: 127
    Successfully decoded bit string as int: 238
    Successfully decoded bit string as int: 512
    [127, 238, 512]"#]];
pub const BERNSTEINVAZIRANINISQ_EXPECT: Expect = expect!["[One, Zero, One, Zero, One]"];
pub const BERNSTEINVAZIRANINISQ_EXPECT_DEBUG: Expect = expect!["[One, Zero, One, Zero, One]"];
pub const BITFLIPCODE_EXPECT: Expect = expect![[r#"
    STATE:
    |001⟩: 0.4472+0.0000𝑖
    |110⟩: 0.8944+0.0000𝑖
    STATE:
    |000⟩: 0.4472+0.0000𝑖
    |111⟩: 0.8944+0.0000𝑖
    One"#]];
pub const BITFLIPCODE_EXPECT_DEBUG: Expect = expect![[r#"
    STATE:
    |001⟩: 0.4472+0.0000𝑖
    |110⟩: 0.8944+0.0000𝑖
    STATE:
    |000⟩: 0.4472+0.0000𝑖
    |111⟩: 0.8944+0.0000𝑖
    One"#]];
pub const DEUTSCHJOZSA_EXPECT: Expect = expect![[r#"
    SimpleConstantBoolF is constant
    SimpleBalancedBoolF is balanced
    ConstantBoolF is constant
    BalancedBoolF is balanced
    [(SimpleConstantBoolF, true), (SimpleBalancedBoolF, false), (ConstantBoolF, true), (BalancedBoolF, false)]"#]];
pub const DEUTSCHJOZSA_EXPECT_DEBUG: Expect = expect![[r#"
    SimpleConstantBoolF is constant
    SimpleBalancedBoolF is balanced
    ConstantBoolF is constant
    BalancedBoolF is balanced
    [(SimpleConstantBoolF, true), (SimpleBalancedBoolF, false), (ConstantBoolF, true), (BalancedBoolF, false)]"#]];
pub const DEUTSCHJOZSANISQ_EXPECT: Expect =
    expect!["([One, Zero, Zero, Zero, Zero], [Zero, Zero, Zero, Zero, Zero])"];
pub const DEUTSCHJOZSANISQ_EXPECT_DEBUG: Expect =
    expect!["([One, Zero, Zero, Zero, Zero], [Zero, Zero, Zero, Zero, Zero])"];
pub const DOTPRODUCTVIAPHASEESTIMATION_EXPECT: Expect = expect![[r#"
    Computing inner product of vectors (cos(Θ₁/2), sin(Θ₁/2))⋅(cos(Θ₂/2), sin(Θ₂/2)) ≈ -cos(x𝝅/2ⁿ)
    Θ₁=0.4487989505128276, Θ₂=0.6283185307179586.
    x = 16, n = 4.
    Computed value = 1.0, true value = 0.995974293995239
    (16, 4)"#]];
pub const DOTPRODUCTVIAPHASEESTIMATION_EXPECT_DEBUG: Expect = expect![[r#"
    Computing inner product of vectors (cos(Θ₁/2), sin(Θ₁/2))⋅(cos(Θ₂/2), sin(Θ₂/2)) ≈ -cos(x𝝅/2ⁿ)
    Θ₁=0.4487989505128276, Θ₂=0.6283185307179586.
    x = 16, n = 4.
    Computed value = 1.0, true value = 0.995974293995239
    (16, 4)"#]];
pub const GROVER_EXPECT: Expect = expect![[r#"
    Number of iterations: 4
    Reflecting about marked state...
    Reflecting about marked state...
    Reflecting about marked state...
    Reflecting about marked state...
    [Zero, One, Zero, One, Zero]"#]];
pub const GROVER_EXPECT_DEBUG: Expect = expect![[r#"
    Number of iterations: 4
    Reflecting about marked state...
    Reflecting about marked state...
    Reflecting about marked state...
    Reflecting about marked state...
    [Zero, One, Zero, One, Zero]"#]];
pub const HIDDENSHIFT_EXPECT: Expect = expect![[r#"
    Found 170 successfully!
    Found 512 successfully!
    Found 999 successfully!
    [170, 512, 999]"#]];
pub const HIDDENSHIFT_EXPECT_DEBUG: Expect = expect![[r#"
    Found 170 successfully!
    Found 512 successfully!
    Found 999 successfully!
    [170, 512, 999]"#]];
pub const HIDDENSHIFTNISQ_EXPECT: Expect = expect!["[One, Zero, Zero, Zero, Zero, One]"];
pub const HIDDENSHIFTNISQ_EXPECT_DEBUG: Expect = expect!["[One, Zero, Zero, Zero, Zero, One]"];
pub const PHASEESTIMATION_EXPECT: Expect = expect![[r#"(1.0, 1.0799224746714913)"#]];
pub const PHASEESTIMATION_EXPECT_DEBUG: Expect = expect![[r#"(1.0, 1.0799224746714913)"#]];
pub const PHASEFLIPCODE_EXPECT: Expect = expect![[r#"
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
pub const PHASEFLIPCODE_EXPECT_DEBUG: Expect = expect![[r#"
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
pub const QRNG_EXPECT: Expect = expect![[r#"
    Sampling a random number between 0 and 100:
    46"#]];
pub const QRNG_EXPECT_DEBUG: Expect = expect![[r#"
    Sampling a random number between 0 and 100:
    46"#]];
pub const QRNGNISQ_EXPECT: Expect = expect!["[Zero, Zero, One, One, One]"];
pub const QRNGNISQ_EXPECT_DEBUG: Expect = expect!["[Zero, Zero, One, One, One]"];
pub const SHOR_EXPECT: Expect = expect![[r#"
    *** Factorizing 187, attempt 1.
    Estimating period of 182.
    Estimating frequency with bitsPrecision=17.
    Estimated frequency=126158
    Found period=80
    Found factor=17
    Found factorization 187 = 17 * 11
    (17, 11)"#]];
pub const SHOR_EXPECT_DEBUG: Expect = expect![[r#"
    *** Factorizing 187, attempt 1.
    Estimating period of 182.
    Estimating frequency with bitsPrecision=17.
    Estimated frequency=126158
    Found period=80
    Found factor=17
    Found factorization 187 = 17 * 11
    (17, 11)"#]];
pub const SIMPLEPHASEESTIMATION_EXPECT: Expect = expect![[r#"[Zero, Zero, Zero, One, Zero, One]"#]];
pub const SIMPLEPHASEESTIMATION_EXPECT_DEBUG: Expect =
    expect![[r#"[Zero, Zero, Zero, One, Zero, One]"#]];
pub const SIMPLEVQE_EXPECT: Expect = expect![[r#"
   Beginning descent from value 0.43300000000000005.
   Value improved to 0.35300000000000004.
   Value improved to 0.3454.
   Value improved to 0.3422.
   Value improved to 0.3216.
   Descent done. Attempts: 52, Step: 0.0009765625, Arguments: [1.5, 1.0625], Value: 0.3216.
   0.3216"#]];
pub const SIMPLEVQE_EXPECT_DEBUG: Expect = expect![[r#"
   Beginning descent from value 0.43300000000000005.
   Value improved to 0.35300000000000004.
   Value improved to 0.3454.
   Value improved to 0.3422.
   Value improved to 0.3216.
   Descent done. Attempts: 52, Step: 0.0009765625, Arguments: [1.5, 1.0625], Value: 0.3216.
   0.3216"#]];
pub const SUPERDENSECODING_EXPECT: Expect = expect!["((false, true), (false, true))"];
pub const SUPERDENSECODING_EXPECT_DEBUG: Expect = expect!["((false, true), (false, true))"];
pub const TELEPORTATION_EXPECT: Expect = expect![[r#"
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
pub const TELEPORTATION_EXPECT_DEBUG: Expect = expect![[r#"
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
pub const THREEQUBITREPETITIONCODE_EXPECT: Expect = expect!["(true, 0)"];
pub const THREEQUBITREPETITIONCODE_EXPECT_DEBUG: Expect = expect!["(true, 0)"];
