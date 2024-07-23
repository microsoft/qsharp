// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

/// # Sample
/// Multi File Testing Project (Refer to README for Project Overview)
///
/// # Description
/// This code builds upon the concepts explained in the README file,
/// demonstrating how to organize Q# code into multiple files for testing.

namespace OperationEquivalence {
    open Microsoft.Quantum.Diagnostics;
    open Test_SWAP;
    /// # Summary
    /// Verifies the equivalence of quantum operations using `Fact` function
    /// and the `CheckOperationsAreEqual` operation.
    operation TestEquivalence() : Unit {
        let actual = qs => Test_SWAP.ApplySWAP1(qs);
        let expected = qs => Test_SWAP.ApplySWAP2(qs);
        Fact(CheckOperationsAreEqual(2, actual, expected) == true, "Actual and expected operation should be same");
    }
}
