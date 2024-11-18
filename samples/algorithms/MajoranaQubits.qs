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
        let a = HardwareProvider.Mzz(control, target);
        let b = HardwareProvider.Mxx(target, ancilla);
        let c = HardwareProvider.Mx(target);
        if b == One {
            HardwareProvider.Z(control);
        }
        if [a, c] == [One, One] {
            HardwareProvider.X(ancilla);
        }
    }

    /// Prepare a Bell Pair.
    /// Source: [1] Figure 18a.
    operation BellPair(q1 : Qubit, q2 : Qubit) : Unit {
        // Bring the qubits to their ground state.
        HardwareProvider.Mz(q1);
        HardwareProvider.Mz(q2);

        // Bell Pair preparation
        if HardwareProvider.Mxx(q1, q2) == One {
            HardwareProvider.Z(q2);
        }
    }

    /// Measure a Bell Pair.
    /// Source: [1] Figure 18b.
    operation BellMeasurement(q1 : Qubit, q2 : Qubit) : (Result, Result) {
        let z = HardwareProvider.Mzz(q1, q2);
        let x = HardwareProvider.Mxx(q1, q2);
        (x, z)
    }
}

/// A set of custom measurements exposed from a hardware
/// provider using Majorana Qubits.
namespace HardwareProvider {
    @SimulatableIntrinsic()
    operation X(q : Qubit) : Unit {
        Std.Intrinsic.X(q);
    }

    @SimulatableIntrinsic()
    operation Z(q : Qubit) : Unit {
        Std.Intrinsic.Z(q);
    }

    @Measurement()
    @SimulatableIntrinsic()
    operation Mx(q : Qubit) : Result {
        H(q);
        M(q)
    }

    @Measurement()
    @SimulatableIntrinsic()
    operation Mz(q : Qubit) : Result {
        M(q)
    }

    @Measurement()
    @SimulatableIntrinsic()
    operation Mxx(q1 : Qubit, q2 : Qubit) : Result {
        Std.Intrinsic.Measure([PauliX, PauliX], [q1, q2])
    }

    @Measurement()
    @SimulatableIntrinsic()
    operation Mzz(q1 : Qubit, q2 : Qubit) : Result {
        Std.Intrinsic.Measure([PauliZ, PauliZ], [q1, q2])
    }
}
