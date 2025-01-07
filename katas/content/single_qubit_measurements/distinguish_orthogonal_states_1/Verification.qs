namespace Kata.Verification {
    import Std.Math.*;
    import KatasUtils.*;

    // Distinguish specific orthogonal states
    // |ψ₊⟩ =   0.6 * |0⟩ + 0.8 * |1⟩,
    // |ψ₋⟩ =  -0.8 * |0⟩ + 0.6 * |1⟩.
    operation StatePrep_IsQubitPsiPlus(q : Qubit, state : Int) : Unit is Adj {
        if state == 0 {
            // convert |0⟩ to |ψ₋⟩
            X(q);
            Ry(2.0 * ArcTan2(0.8, 0.6), q);
        } else {
            // convert |0⟩ to |ψ₊⟩
            Ry(2.0 * ArcTan2(0.8, 0.6), q);
        }
    }

    operation CheckSolution() : Bool {
        let isCorrect = DistinguishTwoStates_SingleQubit(
            StatePrep_IsQubitPsiPlus,
            Kata.IsQubitPsiPlus,
            ["|ψ₋⟩", "|ψ₊⟩"],
            false
        );
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
        }
        isCorrect
    }

}
