namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;

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
                Message($"The solution was incorrect for the test case alpha = {alpha}, beta = {beta}, theta = {theta}.");
                Message("Hint: examine the state prepared by your solution and compare it with the state it " +
                    "is expected to prepare.");
                use initial = Qubit(); // |0〉
                ShowQuantumStateComparison([initial], solution, reference);
                Reset(initial);
                return false;
            }
        }

        Message("Correct!");
        Message("The solution was correct for all test cases.");
        true
    }
}
