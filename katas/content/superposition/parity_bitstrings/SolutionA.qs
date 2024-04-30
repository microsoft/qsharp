namespace Kata {
    operation AllStatesWithParitySuperposition (qs : Qubit[], parity : Int) : Unit is Adj + Ctl {
        // base of recursion: if N = 1, set the qubit to parity
        let N = Length(qs);
        if N == 1 {
            if parity == 1 {
                X(qs[0]);
            }
        } else {
            // split the first qubit into 0 and 1 (with equal amplitudes!)
            H(qs[0]);
            // prep 0 ⊗ state with the same parity and 1 ⊗ state with the opposite parity
            ApplyControlledOnInt(0, AllStatesWithParitySuperposition, qs[0 .. 0], (qs[1 ...], parity));
            ApplyControlledOnInt(1, AllStatesWithParitySuperposition, qs[0 .. 0], (qs[1 ...], 1 - parity));
        }
    }
}
