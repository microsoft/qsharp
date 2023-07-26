namespace Kata.Verification {

    operation IsSeven_MarkingOracle_Reference(x: Qubit[], y: Qubit): Unit is Adj + Ctl {
        Controlled X(x, y);
    }

    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution () : Bool {
        let isCorrect = CheckTwoOraclesAreEqual(
            3..3,
            Kata.IsSeven_MarkingOracle,
            IsSeven_MarkingOracle_Reference);
        if isCorrect {
            Message("All tests passed.");
        } else {
            Message("Test failed: Operation is not the same as the reference operation.");
        }
        isCorrect
    }

}
