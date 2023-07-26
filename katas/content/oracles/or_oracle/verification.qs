namespace Kata.Verification {

    // Task 3.1.
    operation Or_Oracle_Reference(x: Qubit[], y: Qubit): Unit is Adj + Ctl {
        X(y);
        ApplyControlledOnInt(0, X, x, y);
    }

    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = CheckTwoOraclesAreEqual(1..7, Kata.Or_Oracle, Or_Oracle_Reference);
        if (isCorrect) {
            Message("All tests passed.");
        } else {
            Message("Test failed: Operation is not the same as the reference operation.");
        }
        isCorrect
    }

}
