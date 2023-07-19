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
            Message("Correct!");
            Message("The solution was correct for all test cases.");
            ShowEffectOnQuantumState(target, op);
        } else {
            Message("Incorrect.");
            Message("The solution was incorrect for at least one test case.");
            ShowQuantumStateComparison(target, op, reference);
        }

        isCorrect
    }
}
