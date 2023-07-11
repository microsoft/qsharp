namespace Kata.Verification {

    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution () : Bool {
        let isCorrect = CheckTwoOraclesAreEqual(3..3, Kata.IsSeven_MarkingOracle, IsSeven_MarkingOracle);
        if isCorrect {
            Message("All tests passed.");
        } else {
            Message("Test failed: Operation is not the same as the reference operation.");
        }
        isCorrect
    }

}
