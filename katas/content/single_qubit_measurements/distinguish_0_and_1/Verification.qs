namespace Kata.Verification {
    import Std.Diagnostics.*;
    import KatasUtils.*;

    operation StatePrep_IsQubitZero(q : Qubit, state : Int) : Unit is Adj {
        if state == 0 {
            // convert |0⟩ to |1⟩
            X(q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = DistinguishTwoStates_SingleQubit(
            StatePrep_IsQubitZero,
            Kata.IsQubitZero,
            ["|1⟩", "|0⟩"],
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
