namespace Kata {
    open Microsoft.Quantum.Math;

    operation ThreeStates_TwoQubits_Phases (qs : Qubit[]) : Unit {
        // First create (|00⟩ + |01⟩ + |10⟩) / sqrt(3) state
        ThreeStates_TwoQubits_Reference(qs);

        R1(4.0 * PI() / 3.0, qs[0]);
        R1(2.0 * PI() / 3.0, qs[1]);
    }

    operation ThreeStates_TwoQubits (qs : Qubit[]) : Unit is Adj {

        // Follow Niel's answer at https://quantumcomputing.stackexchange.com/a/2313/

        // Rotate first qubit to (sqrt(2) |0⟩ + |1⟩) / sqrt(3)
        let theta = ArcSin(1.0 / Sqrt(3.0));
        Ry(2.0 * theta, qs[0]);

        // Split the state sqrt(2) |0⟩ ⊗ |0⟩ into |00⟩ + |01⟩
        ApplyControlledOnInt(0, H, [qs[0]], qs[1]);
    }
}
