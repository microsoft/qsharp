namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Diagnostics;

    operation ThreeQuartersPiPhase(q : Qubit) : Unit is Adj + Ctl {
        S(q);
        T(q);
    }

    operation CheckSolution() : Bool {
        let isCorrect = VerifySingleQubitOperationModified(Kata.ThreeQuartersPiPhase, ThreeQuartersPiPhase);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[1];

        let op = register => Kata.ThreeQuartersPiPhase(register[0]);
        let reference = register => ThreeQuartersPiPhase(register[0]);
        if isCorrect {
            Message("Correct!");
            Message("The solution was correct for all test cases.");
            ShowEffectOnQuantumState(target, op);
        } else {
            Message("Incorrect.");
            Message("The solution was incorrect for at least one test case.");
            ShowQuantumStateComparison(target, op, reference);
        }

        return isCorrect;
    }

    operation VerifySingleQubitOperationModified(
        op : (Qubit => Unit is Adj + Ctl),
        reference : (Qubit => Unit is Adj + Ctl))
    : Bool {
        use (control, target) = (Qubit(), Qubit());
        within {
            X(target);
            H(control);
        }
        apply {
            Controlled op([control], target);
            Adjoint Controlled reference([control], target);
        }
        let isCorrect = CheckAllZero([control, target]);
        ResetAll([control, target]);

        isCorrect
    }
}
