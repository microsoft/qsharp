// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Diagnostics {
    open QIR.Intrinsic;

    function DumpMachine() : Unit {
        __quantum__qis__dumpmachine__body();
    }

    function AssertZero(qubit : Qubit) : Unit {
        __quantum__qis__assertzero__body(qubit);
    }

    function CheckZero(qubit : Qubit) : Bool {
        return __quantum__qis__checkzero__body(qubit);
    }
}
