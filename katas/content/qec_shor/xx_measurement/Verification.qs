namespace Kata.Verification {
    import KatasUtils.*;

    operation StatePrep_XXMeasurement(qs : Qubit[], state : Int, alpha : Double) : Unit is Adj {
        // prep cos(alpha) * |0..0⟩ + sin(alpha) * |1..1⟩
        Ry(2.0 * alpha, qs[0]);
        CNOT(qs[0], qs[1]);

        if state == 1 {
            X(qs[0]);
        }

        ApplyToEachA(H, qs);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = DistinguishStates_MultiQubit(
            2,
            2,
            StatePrep_XXMeasurement,
            Kata.XXMeasurement,
            true,
            ["α|++⟩ + β|--⟩", "α|+-⟩ + β|-+⟩"]
        );

        if (isCorrect) {
            Message("Correct!");
        } else {
            Message("Incorrect.");
        }

        isCorrect
    }
}
