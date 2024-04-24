namespace Kata {
    open Microsoft.Quantum.Math;

    operation ThreeStates_TwoQubits (qs : Qubit[]) : Unit is Adj + Ctl {
        // Rotate first qubit to (sqrt(2) |0⟩ + |1⟩) / sqrt(3) (task 1.4 from BasicGates kata)
        let theta = ArcSin(1.0 / Sqrt(3.0));
        Ry(2.0 * theta, qs[0]);

        // Split the state sqrt(2) |0⟩ ⊗ |0⟩ into |00⟩ + |01⟩
        ApplyControlledOnInt(0, H,[qs[0]], qs[1]);
    }
}
