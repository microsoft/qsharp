namespace Kata.Verification {
    // ------------------------------------------------------
    // Exercise 5. Distinguish specific orthogonal states
    // ------------------------------------------------------

    // |ψ₊⟩ =   0.6 * |0⟩ + 0.8 * |1⟩,
    // |ψ₋⟩ =  -0.8 * |0⟩ + 0.6 * |1⟩.
    operation StatePrep_IsQubitPsiPlus(q: Qubit, state: Int): Unit is Adj {
        if state == 0 {
            // convert |0⟩ to |ψ₋⟩
            X(q);
            Ry(2.0 * ArcTan2(0.8, 0.6), q);
        } else {
            // convert |0⟩ to |ψ₊⟩
            Ry(2.0 * ArcTan2(0.8, 0.6), q);
        }
    }

    operation CheckSolution(): Bool {
        let isCorrect = DistinguishTwoStates(
            StatePrep_IsQubitPsiPlus,
            Kata.IsQubitPsiPlus, 
            ["|ψ₋⟩", "|ψ₊⟩"],
            false);
        if isCorrect {
            Message("All tests passed.");
        }
        isCorrect
    }

}
