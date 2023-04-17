// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace QIR.Intrinsic {

    // Controlled Gates

    operation __quantum__qis__ccx__body(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__cx__body(control : Qubit, target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__cy__body(control : Qubit, target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__cz__body(control : Qubit, target : Qubit) : Unit {
        body intrinsic;
    }

    // Rotation Gates

    operation __quantum__qis__rx__body(angle : Double, target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__rxx__body(angle : Double, target1 : Qubit, target2 : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__ry__body(angle : Double, target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__ryy__body(angle : Double, target1 : Qubit, target2 : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__rz__body(angle : Double, target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__rzz__body(angle : Double, target1 : Qubit, target2 : Qubit) : Unit {
        body intrinsic;
    }

    // Single-Qubit Gates

    operation __quantum__qis__h__body(target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__s__body(target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__s__adj(target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__t__body(target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__t__adj(target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__x__body(target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__y__body(target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__z__body(target : Qubit) : Unit {
        body intrinsic;
    }

    // Two-Qubit Gates

    operation __quantum__qis__swap__body(target1 : Qubit, target2 : Qubit) : Unit {
        body intrinsic;
    }

    // Quantum Measurement

    operation __quantum__qis__m__body(target : Qubit) : Result {
        body intrinsic;
    }

    operation __quantum__qis__reset__body(target : Qubit) : Unit {
        body intrinsic;
    }

    operation __quantum__qis__mresetz__body(target : Qubit) : Result {
        body intrinsic;
    }
}

namespace QIR.Runtime {
    function __quantum__rt__qubit_allocate() : Qubit {
        body intrinsic;
    }

    function __quantum__rt__qubit_release(q : Qubit) : Unit {
        body intrinsic;
    }

    function __quantum__rt__qubit_allocate_array(size: Int) : Qubit[] {
        mutable qs = [];
        for _ in 0..size-1 {
            set qs += [__quantum__rt__qubit_allocate()];
        }
        return qs;
    }

    function __quantum__rt__qubit_release_array(qs : Qubit[]) : Unit {
        for q in qs {
            __quantum__rt__qubit_release(q);
        }
    }
}
