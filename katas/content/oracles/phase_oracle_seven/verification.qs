namespace Kata.Verification {

    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution(): Bool {
        let N = 3;
        let isCorrect = CheckOperationsEqualReferenced(N, Kata.IsSeven_PhaseOracle, IsSeven_PhaseOracle);
        if isCorrect {
            Message("All tests passed.");
        } else {
            Message("Test failed: Operation is not the same as the reference operation.");
        }
        isCorrect
    }

}
