namespace Kata.Verification {
    import KatasUtils.*;

    // 0 - |Φ⁺⟩ = (|00⟩ + |11⟩) / sqrt(2)
    // 1 - |Φ⁻⟩ = (|00⟩ - |11⟩) / sqrt(2)
    // 2 - |Ψ⁺⟩ = (|01⟩ + |10⟩) / sqrt(2)
    // 3 - |Ψ⁻⟩ = (|01⟩ - |10⟩) / sqrt(2)
    operation StatePrep_BellState(
        qs : Qubit[],
        state : Int,
        dummyVar : Double
    ) : Unit is Adj {
        H(qs[0]);
        CNOT(qs[0], qs[1]);

        // Now we have |00⟩ + |11⟩ - modify it based on state arg
        if state % 2 == 1 {
            // negative phase
            Z(qs[1]);
        }
        if state / 2 == 1 {
            X(qs[1]);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = DistinguishStates_MultiQubit(
            2,
            4,
            StatePrep_BellState,
            Kata.BellState,
            false,
            ["|Φ⁺⟩ = (|00⟩ + |11⟩) / sqrt(2)", "|Φ⁻⟩ = (|00⟩ - |11⟩) / sqrt(2)", "|Ψ⁺⟩ = (|01⟩ + |10⟩) / sqrt(2)", "|Ψ⁻⟩ = (|01⟩ - |10⟩) / sqrt(2)"]
        );

        if not isCorrect {
            Message("Incorrect.");
        } else {
            Message("Correct!");
        }

        return isCorrect;
    }
}
