namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;

    operation PrepareRotatedState (alpha : Double, beta : Double, q : Qubit) : Unit is Adj+Ctl {
        let phi = ArcTan2(beta, alpha);
        Rx(2.0 * phi, q);
    }

    operation CheckSolution() : Bool {
        for i in 0 .. 10 {
            let i = IntAsDouble(i);
            let alpha = Cos(i);
            let beta = Sin(i);
            let solution = register => Kata.PrepareRotatedState(alpha, beta, register[0]);
            let reference = register => PrepareRotatedState(alpha, beta, register[0]);
            let isCorrect = CheckOperationsEquivalenceOnZeroStateStrict(solution, reference, 1);
            if not isCorrect {
                Message("Incorrect.");
                Message("The solution was incorrect for at least one test case.");
                use target = Qubit[1]; // |0〉
                ShowQuantumStateComparison(target, solution, reference);
                ResetAll(target);
                return false;
            }
        }

        Message("Correct!");
        Message("The solution was correct for all test cases.");
        true
    }
}
