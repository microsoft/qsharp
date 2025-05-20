// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};

// Each file in the samples/OpenQASM folder is compiled and run as two tests and should
// have matching expect strings in this file. If new samples are added, this file will
// fail to compile until the new expect strings are added.
pub const BELLPAIR_EXPECT: Expect = expect!["(One, One)"];
pub const BELLPAIR_EXPECT_DEBUG: Expect = expect!["(One, One)"];
pub const OPENQASMHELLOWORLD_EXPECT: Expect = expect!["Zero"];
pub const OPENQASMHELLOWORLD_EXPECT_DEBUG: Expect = expect!["Zero"];
pub const BERNSTEINVAZIRANI_EXPECT: Expect = expect!["[One, Zero, One, Zero, One]"];
pub const BERNSTEINVAZIRANI_EXPECT_DEBUG: Expect = expect!["[One, Zero, One, Zero, One]"];
pub const GROVER_EXPECT: Expect = expect!["[Zero, One, Zero, One, Zero]"];
pub const GROVER_EXPECT_DEBUG: Expect = expect!["[Zero, One, Zero, One, Zero]"];
pub const RANDOMNUMBER_EXPECT: Expect = expect!["9"];
pub const RANDOMNUMBER_EXPECT_DEBUG: Expect = expect!["9"];
pub const SIMPLE1DISINGORDER1_EXPECT: Expect = expect!["[Zero, One, One, Zero, Zero, One, One, One, One]"];
pub const SIMPLE1DISINGORDER1_EXPECT_DEBUG: Expect = expect!["[Zero, One, One, Zero, Zero, One, One, One, One]"];
