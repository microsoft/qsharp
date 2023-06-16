namespace Kata.Reference {

    // ------------------------------------------------------
    // Exercise 3. Distinguish |+❭ and |-❭ using Measure operation
    // ------------------------------------------------------
    operation StatePrep_IsQubitMinus (q : Qubit, state : Int) : Unit is Adj {
        if state == 1 {
            // convert |0⟩ to |-⟩
            X(q);
            H(q);
        } else {
            // convert |0⟩ to |+⟩
            H(q);
        }
    }

}
