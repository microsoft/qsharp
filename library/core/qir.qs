// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace QIR.Runtime {
    operation __quantum__rt__qubit_allocate() : Qubit {
        body intrinsic;
    }

    operation __quantum__rt__qubit_release(q : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__rt__qubit_allocate_array(size: Int) : Qubit[] {
        mutable qs = [];
        for _ in 0..size-1 {
            set qs += [__quantum__rt__qubit_allocate()];
        }
        qs
    }

    operation __quantum__rt__qubit_release_array(qs : Qubit[]) : Unit {
        for q in qs {
            __quantum__rt__qubit_release(q);
        }
    }
}
