namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation GHZ_State_Reference(qs : Qubit[]) : Unit is Adj {
        H(Head(qs));
        for q in Rest(qs) {
            CNOT(Head(qs), q);
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

        for i in 2 .. 6 {
            let isCorrect = DistinguishStates_MultiQubit(
                i, 2,
                StatePrep_GHZOrWState,
                Kata.GHZOrWState,
                false,
                ["|GHZ⟩", "|W⟩"]);

            if not isCorrect {
                Message("Incorrect!");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
