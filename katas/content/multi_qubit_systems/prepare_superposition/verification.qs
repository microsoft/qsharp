namespace Kata.Verification {
    operation PrepareSuperposition_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[1]);
        H(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution(): Bool {
        let isCorrect = AssertEqualOnZeroState(Kata.PrepareSuperposition, PrepareSuperposition_Reference);
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
        }
        return isCorrect;
    }
}