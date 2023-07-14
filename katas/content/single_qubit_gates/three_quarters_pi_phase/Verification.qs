namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation ThreeQuartersPiPhase(q : Qubit) : Unit is Adj + Ctl {
        S(q);
        T(q);
    }

    operation CheckSolution() : Bool {
        let isCorrect = VerifySingleQubitOperation(Kata.ThreeQuartersPiPhase, ThreeQuartersPiPhase);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[1];
        let op = register => Kata.ThreeQuartersPiPhase(register[0]);
        let reference = register => ThreeQuartersPiPhase(register[0]);
        if isCorrect {
            ShowEffectOnQuantumState(target, op);
        } else {
            ShowQuantumStateComparison(target, op, reference);
        }

        return false;
    }
}
