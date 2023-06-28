namespace Kata {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation ApplyYReference(q : Qubit) : Unit is Adj + Ctl {
        Y(q);
    }

    operation VerifyExercise() : Bool {
        let isCorrect = VerifySingleQubitOperation(ApplyY, ApplyYReference);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[1];
        let op = register => ApplyY(register[0]);
        let reference = register => ApplyYReference(register[0]);
        if isCorrect {
            ShowEffectOnQuantumState(target, op);
        } else {
            ShowQuantumStateComparison(target, op, reference);
        }

        isCorrect
    }
}