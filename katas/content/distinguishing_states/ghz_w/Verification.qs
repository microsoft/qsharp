namespace Kata.Verification {
    import KatasUtils.*;

    operation GHZ_State_Reference(qs : Qubit[]) : Unit is Adj {
        H(qs[0]);
        for q in qs[1...] {
            CNOT(qs[0], q);
        }
    }

    operation StatePrep_GHZOrWState(
        qs : Qubit[],
        state : Int,
        dummyVar : Double
    ) : Unit is Adj {
        if state == 0 {
            // Prepare GHZ state
            GHZ_State_Reference(qs);
        } else {
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
                StatePrep_GHZOrWState,
                Kata.GHZOrWState,
                false,
                ["|GHZ⟩", "|W⟩"]
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
