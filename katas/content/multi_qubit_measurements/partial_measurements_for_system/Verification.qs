namespace Kata.Verification {
    import KatasUtils.*;

    // Distinguish orthogonal states using partial measurements
    operation StatePrep_IsPlusPlusMinus(qs : Qubit[], state : Int, dummyVar : Double) : Unit is Adj {
        if state == 0 {
            // Prepare the state |++-⟩
            X(qs[2]);
        } else {
            // Prepare the state |---⟩
            ApplyToEachA(X, qs);
        }
        ApplyToEachA(H, qs);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = DistinguishStates_MultiQubit(
            3,
            2,
            StatePrep_IsPlusPlusMinus,
            Kata.IsPlusPlusMinus,
            true,
            ["|++-⟩", "|---⟩"]
        );
        if (isCorrect) {
            Message("Correct!");
        } else {
            Message("Incorrect.");
        }

        isCorrect
    }
}
