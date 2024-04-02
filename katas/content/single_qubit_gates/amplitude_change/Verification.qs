namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;

    operation AmplitudeChange (alpha : Double, q : Qubit) : Unit is Adj+Ctl {
        Ry(2.0 * alpha, q);
    }

    operation CheckSolution() : Bool {
        for i in 0 .. 36 {
            let alpha = ((2.0 * PI()) * IntAsDouble(i)) / 36.0;
            let solution = register => Kata.AmplitudeChange(alpha, register[0]);
            let reference = register => AmplitudeChange(alpha, register[0]);
            let isCorrect = CheckOperationsEquivalenceOnZeroStateStrict(solution, reference, 1);
            if not isCorrect {
                Message("Incorrect.");
                Message($"The solution was incorrect for the test case alpha = {alpha}.");
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