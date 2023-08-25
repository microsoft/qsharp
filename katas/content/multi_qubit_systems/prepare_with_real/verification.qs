namespace Kata.Verification {
    operation PrepareWithReal_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        X(qs[1]);
        H(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution(): Bool {
        let isCorrect = AssertEqualOnZeroState(Kata.PrepareWithReal, PrepareWithReal_Reference);
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
        }
        return isCorrect;
    }
}