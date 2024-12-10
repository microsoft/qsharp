namespace Kata.Verification {
    import KatasUtils.*;

    operation StatePrep_AllZerosOrWState(
        qs : Qubit[],
        state : Int,
        dummyVar : Double
    ) : Unit is Adj {
        if state == 1 {
            // Prepare W state
            WState_Arbitrary_Reference(qs);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for i in 2..6 {
            let isCorrect = DistinguishStates_MultiQubit(
                i,
                2,
                StatePrep_AllZerosOrWState,
                Kata.AllZerosOrWState,
                false,
                ["|0...0⟩", "|W⟩"]
            );

            if not isCorrect {
                Message("Incorrect.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
