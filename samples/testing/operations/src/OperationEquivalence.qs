// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Std.Diagnostics.*;
import CustomOperation.*;
/// # Summary
/// Verifies the equivalence of quantum operations up to a global phase using `Fact` function
/// and the `CheckOperationsAreEqual` operation. You can either run this here,
/// by clicking `Run` in VsCode or call `TestEquivalence` operation in Python.
@EntryPoint()
operation TestEquivalence() : Unit {
    let actual = qs => ApplySWAP(qs[0], qs[1]);
    let expected = qs => SWAP(qs[0], qs[1]);
    Fact(CheckOperationsAreEqual(2, actual, expected), "Actual and expected operation should be same");
}
