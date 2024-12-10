namespace Kata.Verification {
    import KatasUtils.*;

    // 0 - (|00⟩ + |01⟩ + |10⟩ + |11⟩) / 2
    // 1 - (|00⟩ - |01⟩ + |10⟩ - |11⟩) / 2
    // 2 - (|00⟩ + |01⟩ - |10⟩ - |11⟩) / 2
    // 3 - (|00⟩ - |01⟩ - |10⟩ + |11⟩) / 2
    operation StatePrep_TwoQubitState(
        qs : Qubit[],
        state : Int,
        dummyVar : Double
    ) : Unit is Adj {
        StatePrep_BasisStateMeasurement(qs, state, dummyVar);

        H(qs[0]);
        H(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = DistinguishStates_MultiQubit(
            2,
            4,
            StatePrep_TwoQubitState,
            Kata.TwoQubitState,
            false,
            ["(|00⟩ + |01⟩ + |10⟩ + |11⟩) / 2", "(|00⟩ - |01⟩ + |10⟩ - |11⟩) / 2", "(|00⟩ + |01⟩ - |10⟩ - |11⟩) / 2", "(|00⟩ - |01⟩ - |10⟩ + |11⟩) / 2"]
        );

        if not isCorrect {
            Message("Incorrect.");
        } else {
            Message("Correct!");
        }

        return isCorrect
    }
}
