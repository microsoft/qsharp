namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation PrepareRotatedState(q : Qubit) : Unit is Adj + Ctl {
        X(q);
        Z(q);
        Y(q);
    }

    operation CheckSolution() : Bool {
        let isCorrect = VerifySingleQubitOperation(Kata.PrepareRotatedState, PrepareRotatedState);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[1];
        let op = register => Kata.PrepareRotatedState(register[0]);
        let reference = register => PrepareRotatedState(register[0]);
        if isCorrect {
            ShowEffectOnQuantumState(target, op);
        } else {
            ShowQuantumStateComparison(target, op, reference);
        }

        isCorrect
    }
}
