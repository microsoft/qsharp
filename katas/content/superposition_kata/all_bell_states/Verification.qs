namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation AllBellStates_Reference (qs : Qubit[], index : Int) : Unit is Adj + Ctl {
        H(qs[0]);
        
        if index == 1 {
            Z(qs[0]);
        }
        if index == 2 {
            X(qs[1]);
        }
        if index == 3 {
            Z(qs[0]);
            X(qs[1]);
        }
        
        CNOT(qs[0], qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for index in 0 .. 3 {
            if not CheckOperationsEquivalenceOnZeroStateAndIndexWithFeedback(
                Kata.AllBellStates,
                AllBellStates_Reference,
                2,
                index) {
                Message("Incorrect");
                Message($"The test case for index = {index} did not pass.");
                return false;
            }
        }

        Message("Correct");
        return true;
    }
}
