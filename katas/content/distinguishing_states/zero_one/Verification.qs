namespace Kata.Verification {
    import Std.Diagnostics.*;
    import KatasUtils.*;

    operation StatePrep_IsQubitOne(q : Qubit, state : Int) : Unit is Adj {
        if state == 1 {
            // convert |0⟩ to |1⟩
            X(q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = DistinguishTwoStates_SingleQubit(
            StatePrep_IsQubitOne,
            Kata.IsQubitOne,
            ["|0⟩", "|1⟩"],
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
