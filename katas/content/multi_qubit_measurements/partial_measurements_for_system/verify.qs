namespace Kata.Reference {

    // ------------------------------------------------------
    // Exercise 5: Distinguish orthogonal states using partial measurements
    // ------------------------------------------------------
    operation StatePrep_IsPlusPlusMinus (qs : Qubit[], state : Int, dummyVar : Double) : Unit is Adj{
        if state == 0 {
            // prepare the state |++-⟩
            H(qs[0]);
            H(qs[1]);
            X(qs[2]);
            H(qs[2]);
        } else {
            // prepare the state |---⟩
            X(qs[0]);
            H(qs[0]);
            X(qs[1]);
            H(qs[1]);
            X(qs[2]);
            H(qs[2]);
        }
    }

    @Test("Microsoft.Quantum.Katas.CounterSimulator")
    operation T2_IsPlusPlusMinus () : Unit {
        DistinguishStates_MultiQubit(3, 2, StatePrep_IsPlusPlusMinus, IsPlusPlusMinus, false, ["|++-⟩", "|---⟩"]);
    }

}
