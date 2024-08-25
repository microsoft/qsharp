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
                // In case of an error, this value defines the precision with which complex numbers should be displayed
                let precision = 6;
                Message("Incorrect.");
                Message($"The solution was incorrect for the test case alpha = {DoubleAsStringWithPrecision(alpha,precision)}, beta = {DoubleAsStringWithPrecision(beta,precision)}.");
                Message("Hint: examine the state prepared by your solution and compare it with the state it " +
                    "is expected to prepare.");
                ShowQuantumStateComparison(1, qs => (), solution, reference);
                return false;
            }
        }

        Message("Correct!");
        Message("The solution was correct for all test cases.");
        true
    }
}
