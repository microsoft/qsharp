namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;

    operation PrepareArbitraryState (alpha : Double, beta : Double, theta : Double, q : Qubit) : Unit is Adj+Ctl {
        let phi = ArcTan2(beta, alpha);
        Ry(2.0 * phi, q);
        R1(theta, q);
    }

    operation CheckSolution() : Bool {
         for i in 0 .. 10 {
            let i = IntAsDouble(i);
            let alpha = Cos(i);
            let beta = Sin(i);
            let theta = Sin(i);
            let solution = register => Kata.PrepareArbitraryState(alpha, beta, theta, register[0]);
            let reference = register => PrepareArbitraryState(alpha, beta, theta, register[0]);
            let isCorrect = CheckOperationsEquivalenceOnZeroStateStrict(solution, reference, 1);
            if not isCorrect {
                Message("Incorrect.");
                Message("The solution was incorrect for at least one test case.");
                Message("Hint: examine the effect your solution has on the |0〉 state and compare it with the effect " +
                    "it is expected to have.");
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
