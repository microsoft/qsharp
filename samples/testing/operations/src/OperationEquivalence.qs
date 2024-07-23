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
    open CustomOperation;
    /// # Summary
    /// Verifies the equivalence of quantum operations using `Fact` function
    /// and the `CheckOperationsAreEqual` operation. You can either run this here,
    /// by clicking `Run` in VsCode or call `TestEquivalence` operation in python.
    @EntryPoint()
    operation TestEquivalence() : Unit {
        let actual = qs => CustomOperation.ApplySWAP(qs[0], qs[1]);
        let expected = qs => SWAP(qs[0], qs[1]);
        Fact(CheckOperationsAreEqual(2, actual, expected), "Actual and expected operation should be same");
    }
}
