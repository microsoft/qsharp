namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation PrepareMinus(q : Qubit) : Unit is Adj + Ctl {
        X(q);
        H(q);
    }

    operation CheckSolution() : Bool {
        let isCorrect = VerifySingleQubitOperation(Kata.PrepareMinus, PrepareMinus);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[1];
        let op = register => Kata.PrepareMinus(register[0]);
        let reference = register => PrepareMinus(register[0]);
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
