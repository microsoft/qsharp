namespace Kata.Verification {
    import KatasUtils.*;

    operation StatePrep_ZeroZeroOrOneOne(qs : Qubit[], state : Int, dummyVar : Double) : Unit is Adj {
        if state == 1 {
            // |11⟩
            X(qs[0]);
            X(qs[1]);
        }
    }

    operation CheckSolution() : Bool {
        let isCorrect = DistinguishStates_MultiQubit(2, 2, StatePrep_ZeroZeroOrOneOne, Kata.ZeroZeroOrOneOne, true, ["|00⟩", "|11⟩"]);
        if (isCorrect) {
            Message("Correct!");
        } else {
            Message("Incorrect.");
        }

        isCorrect
    }
}
