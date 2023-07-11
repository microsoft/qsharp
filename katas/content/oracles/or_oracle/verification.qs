namespace Kata.Verification {


    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = CheckTwoOraclesAreEqual(1..7, Kata.Or_Oracle, Or_Oracle);
        if (isCorrect) {
            Message("All tests passed.");
        } else {
            Message("Test failed: Operation is not the same as the reference operation.");
        }
        isCorrect
    }

}
