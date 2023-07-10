namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
        // Apply the Pauli Y operation.
        Y(q);
    }

    operation CheckSolution() : Bool {
        let isCorrect = VerifySingleQubitOperation(Kata.ApplyY, ApplyY);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[1];
        let op = register => Kata.ApplyY(register[0]);
        let reference = register => ApplyY(register[0]);
        if isCorrect {
            ShowEffectOnQuantumState(target, op);
        } else {
            ShowQuantumStateComparison(target, op, reference);
        }

        isCorrect
    }
}