namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

    // ------------------------------------------------------
    // Exercise 2. Distinguish |0❭ and |1❭
    // ------------------------------------------------------
    operation StatePrep_IsQubitZero (q : Qubit, state : Int) : Unit is Adj {
        if state == 0 {
            // convert |0⟩ to |1⟩
            X(q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = DistinguishTwoStates(
            StatePrep_IsQubitZero,
            Kata.IsQubitZero,
            ["|1⟩", "|0⟩"],
            false);
        if isCorrect {
            Message("All tests passed.");
        }
        isCorrect
    }

}
