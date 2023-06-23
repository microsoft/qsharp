namespace Kata {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation GlobalPhaseIReference(q : Qubit) : Unit is Adj + Ctl {
        X(q);
        Z(q);
        Y(q);
    }

    operation VerifyExercise() : Bool {
        let isCorrect = VerifySingleQubitOperation(GlobalPhaseI, GlobalPhaseIReference);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[1];
        let op = register => GlobalPhaseI(register[0]);
        let reference = register => GlobalPhaseIReference(register[0]);
        if isCorrect {
            ShowEffectOnQuantumState(target, op);
        } else {
            ShowQuantumStateComparison(target, op, reference);
        }

        isCorrect
    }
}