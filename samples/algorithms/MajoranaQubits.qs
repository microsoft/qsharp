/// # Sample
/// Majorana Qubits
///
/// # Description
/// In hardware providing majorana qubits, common quantum operations
/// are implemented using measurements and Pauli corrections. This
/// sample shows a hypotetical hardware provider exposing some custom
/// gates to Q# and a small library built on top of it.

namespace Main {
    /// Sample program using gates for custom hardware provider.
    operation Main() : (Result, Result) {
        use qs = Qubit[2];
        GateSet.BellPair(qs[0], qs[1]);
        let res = GateSet.BellMeasurement(qs[0], qs[1]);
        ResetAll(qs);
        res
    }
}

/// A set of gates built upon the custom measurements
/// provided by the hardware provider.
///
/// Source:
///  [1] Surface code compilation via edge-disjoint paths
///      https://arxiv.org/pdf/2110.11493
namespace GateSet {
    /// Apply a CNOT gate to the given qubits.
    /// Source: [1] Figure 3.
    operation CNOT(control : Qubit, target : Qubit) : Unit {
        use ancilla = Qubit();
        let a = Mzz(control, target);
        let b = Mxx(target, ancilla);
        let c = Mx(target);
        if b == One {
            Z(control);
        }
        if [a, c] == [One, One] {
            X(ancilla);
        }
    }

    /// Prepare a Bell Pair.
    /// Source: [1] Figure 18a.
    operation BellPair(q1 : Qubit, q2 : Qubit) : Unit {
        // Bring the qubits to their ground state.
        Mz(q1);
        Mz(q2);

        // Bell Pair preparation
        if Mxx(q1, q2) == One {
            Z(q2);
        }
    }

    /// Measure a Bell Pair.
    /// Source: [1] Figure 18b.
    operation BellMeasurement(q1 : Qubit, q2 : Qubit) : (Result, Result) {
        let z = Mzz(q1, q2);
        let x = Mxx(q1, q2);
        (x, z)
    }

    /// User friendly wrapper around the Mx hardware gate.
    operation Mx(q : Qubit) : Result {
        HardwareProvider.__quantum__qis__mx__body(q)
    }

    /// User friendly wrapper around the Mz hardware gate.
    operation Mz(q : Qubit) : Result {
        HardwareProvider.__quantum__qis__mz__body(q)
    }

    /// User friendly wrapper around the Mxx hardware gate.
    operation Mxx(q1 : Qubit, q2 : Qubit) : Result {
        HardwareProvider.__quantum__qis__mxx__body(q1, q2)
    }

    /// User friendly wrapper around the Mzz hardware gate.
    operation Mzz(q1 : Qubit, q2 : Qubit) : Result {
        HardwareProvider.__quantum__qis__mzz__body(q1, q2)
    }
}

/// A set of custom measurements exposed from a hardware
/// provider using Majorana Qubits.
namespace HardwareProvider {
    @Measurement()
    @SimulatableIntrinsic()
    operation __quantum__qis__mx__body(q : Qubit) : Result {
        H(q);
        M(q)
    }

    @Measurement()
    @SimulatableIntrinsic()
    operation __quantum__qis__mz__body(q : Qubit) : Result {
        M(q)
    }

    @Measurement()
    @SimulatableIntrinsic()
    operation __quantum__qis__mxx__body(q1 : Qubit, q2 : Qubit) : Result {
        Std.Intrinsic.Measure([PauliX, PauliX], [q1, q2])
    }

    @Measurement()
    @SimulatableIntrinsic()
    operation __quantum__qis__mzz__body(q1 : Qubit, q2 : Qubit) : Result {
        Std.Intrinsic.Measure([PauliZ, PauliZ], [q1, q2])
    }
}
