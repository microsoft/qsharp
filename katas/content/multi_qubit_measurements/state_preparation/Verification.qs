namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;

    // State preparation using partial measurements
    operation StatePrep_Rotations(qs : Qubit[]) : Unit is Adj {
        // Rotate first qubit to (sqrt(2) |0⟩ + |1⟩) / sqrt(3)
        let theta = ArcSin(1.0 / Sqrt(3.0));
        Ry(2.0 * theta, qs[0]);

        // Split the state sqrt(2) |0⟩ ⊗ |0⟩ into |00⟩ + |01⟩
        ApplyControlledOnInt(0, H, [qs[0]], qs[1]);
    }


    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = CheckOperationsEquivalenceOnZeroState(Kata.PostSelection, StatePrep_Rotations, 2);
        if (isCorrect) {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            // TODO #1207: refactor kata libraries ShowQuantumStateComparison to not require adjoint
            use qs = Qubit[2];
            Message("Expected quantum state after applying the operation:");
            StatePrep_Rotations(qs);
            DumpMachine();
            ResetAll(qs);

            Kata.PostSelection(qs);
            Message("Actual quantum state after applying the operation:");
            DumpMachine();
            ResetAll(qs);
        }
        isCorrect
    }
}
