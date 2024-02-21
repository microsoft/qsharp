namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;

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
                Message($"The solution was incorrect for the test case alpha = {alpha}, beta = {beta}.");
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
