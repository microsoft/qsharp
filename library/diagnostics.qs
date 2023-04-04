// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Diagnostics {
    open QIR.Intrinsic;

    function DumpMachine() : Unit {
        body intrinsic;
    }

    function CheckZero(qubit : Qubit) : Bool {
        body intrinsic;
    }

    /// # Summary
    /// Checks whether a classical condition is true, and throws an exception if it is not.
    ///
    /// # Input
    /// ## actual
    /// The condition to be checked.
    /// ## message
    /// Failure message string to be used as an error message if the classical
    /// condition is false.
    ///
    /// # See Also
    /// - Microsoft.Quantum.Diagnostics.Contradiction
    ///
    /// # Example
    /// The following Q# snippet will throw an exception:
    /// ```qsharp
    /// Fact(false, "Expected true.");
    /// ```
    function Fact(actual : Bool, message : String) : Unit {
        if (not actual) { fail message; }
    }

}
