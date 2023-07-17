// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Diagnostics {
    open QIR.Intrinsic;

    function DumpMachine() : Unit {
        body intrinsic;
    }

    operation CheckZero(qubit : Qubit) : Bool {
        body intrinsic;
    }

    operation CheckAllZero(qubits : Qubit[]) : Bool {
        for q in qubits {
            if not CheckZero(q) {
                return false;
            }
        }

        return true;
    }

    /// Checks whether a classical condition is true, and throws an exception if it is not.
    function Fact(actual : Bool, message : String) : Unit {
        if (not actual) {
            fail message;
        }
    }

}
