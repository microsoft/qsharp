namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Math;

    @EntryPoint()
    operation CheckSolution() : Bool {
        use qs = Qubit[2];

        // Prepare the state that will be passed to the solution.
        H(qs[0]);
        ApplyControlledOnInt(0, Ry(1.0, _), [qs[0]], qs[1]);
        ApplyControlledOnInt(1, Ry(2.0, _), [qs[0]], qs[1]);

        // Call the solution and get the answer.
        let (x1, x2) = Kata.LearnBasisStateAmplitudes(qs);

        // Calculate the expected values based on the rotation angle.
        // We convert |00⟩ + |10⟩ to |0⟩ Ry(1.0)|0⟩ + |1⟩ Ry(2.0)|0⟩, so
        // * the amplitude of |01⟩ is 2nd amp of Ry(1.0)|0⟩
        // * the amplitude of |10⟩ is 1st amp of Ry(2.0)|0⟩
        let (x1_exp, x2_exp) = (
            1.0/Sqrt(2.0) * Sin(0.5 * 1.0),
            1.0/Sqrt(2.0) * Cos(0.5 * 2.0));

        let isCorrect =
            (AbsD(x1 - x1_exp) <= 0.001) and
            (AbsD(x2 - x2_exp) <= 0.001);

        ResetAll(qs);

        // Output different feedback to the user depending on whether the exercise was correct.
        if isCorrect {
            Message("All tests passed.");
        } else {
            Message("One of the amplitudes was too far from the expected value.");
        }

        isCorrect
    }
}
