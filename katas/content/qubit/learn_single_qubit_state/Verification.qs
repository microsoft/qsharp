namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Math;

    @EntryPoint()
    operation CheckSolution(): Bool {
        use q = Qubit();

        // Prepare the state that will be passed to the solution.
        let angle = 0.5;
        Ry(angle, q);

        // Call the solution and get the answer.
        let (a, b) = Kata.LearnSingleQubitState(q);

        // Calculate the expected values based on the rotation angle.
        let (a_exp, b_exp) = (Cos(0.5 * angle), Sin(0.5 * angle));

        Reset(q);

        let isCorrect =
            (AbsD(a - a_exp) <= 0.001) and
            (AbsD(b - b_exp) <= 0.001);

        // Output different feedback to the user depending on whether the exercise was correct.
        if isCorrect {
            Message("All tests passed.");
        } else {
            Message("At least one of the amplitudes was too far from the expected value.");
        }

        isCorrect
    }
}
