namespace Kata.Reference {
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

    operation T2_IsQubitZero () : Unit {
        DistinguishTwoStates(StatePrep_IsQubitZero, IsQubitZero, ["|1⟩", "|0⟩"], false);
    }

    operation Verify() : Bool {
        return true;
        // TODO: Make sure correct result is returned.
    }

}