namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;

    operation UnequalSuperposition_Reference(q : Qubit, alpha : Double) : Unit is Adj + Ctl {
        Ry(2.0 * alpha, q);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let limit = 36;
        let precision = 3;
        for i in 0 .. limit {
            let alpha = 2.0 * PI() * IntAsDouble(i) / IntAsDouble(limit);
            let solution = Kata.UnequalSuperposition(_, alpha);
            let reference = UnequalSuperposition_Reference(_, alpha);
            Message($"Testing for alpha = {DoubleAsStringWithPrecision(alpha, precision)}...");
            if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                qs => solution(qs[0]), 
                qs => reference(qs[0]),
                1) {
                return false;
            }
        }

        true
    }
}
