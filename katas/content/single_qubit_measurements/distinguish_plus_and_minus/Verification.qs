namespace Kata.Verification {
    import KatasUtils.*;

    // Distinguish |+❭ and |-❭ using Measure operation
    operation StatePrep_IsQubitMinus(q : Qubit, state : Int) : Unit is Adj {
        if state == 1 {
            // convert |0⟩ to |-⟩
            X(q);
            H(q);
        } else {
            // convert |0⟩ to |+⟩
            H(q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = DistinguishTwoStates_SingleQubit(
            StatePrep_IsQubitMinus,
            Kata.IsQubitMinus,
            ["|+⟩", "|-⟩"],
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
