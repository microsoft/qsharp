namespace Kata.Verification {
    import KatasUtils.*;

    // 0 - ( |00⟩ - |01⟩ - |10⟩ - |11⟩) / 2
    // 1 - (-|00⟩ + |01⟩ - |10⟩ - |11⟩) / 2
    // 2 - (-|00⟩ - |01⟩ + |10⟩ - |11⟩) / 2
    // 3 - (-|00⟩ - |01⟩ - |10⟩ + |11⟩) / 2
    operation StatePrep_TwoQubitStateTwo(
        qs : Qubit[],
        state : Int,
        dummyVar : Double
    ) : Unit is Adj {
        StatePrep_BasisStateMeasurement(qs, state, dummyVar);

        // Now apply all gates for unitary in reference impl (in reverse + adjoint)
        within {
            ApplyToEachA(X, qs);
            Controlled Z([qs[0]], qs[1]);
            ApplyToEachA(X, qs);
        } apply {
            ApplyToEachCA(H, qs);
        }

        SWAP(qs[0], qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = DistinguishStates_MultiQubit(
            2,
            4,
            StatePrep_TwoQubitStateTwo,
            Kata.TwoQubitStateTwo,
            false,
            ["(+|00⟩ - |01⟩ - |10⟩ - |11⟩) / 2", "(-|00⟩ + |01⟩ - |10⟩ - |11⟩) / 2", "(-|00⟩ - |01⟩ + |10⟩ - |11⟩) / 2", "(-|00⟩ - |01⟩ - |10⟩ + |11⟩) / 2"]
        );

        if not isCorrect {
            Message("Incorrect.");
        } else {
            Message("Correct!");
        }

        return isCorrect
    }
}
