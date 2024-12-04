namespace Kata.Verification {
    import KatasUtils.*;

    operation StatePrep_IsQubitPlus(q : Qubit, state : Int) : Unit is Adj {
        if state == 1 {
            // convert |0⟩ to |+⟩
            H(q);
        } else {
            // convert |0⟩ to |-⟩
            X(q);
            H(q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = DistinguishTwoStates_SingleQubit(
            StatePrep_IsQubitPlus,
            Kata.IsQubitPlus,
            ["|-⟩", "|+⟩"],
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
