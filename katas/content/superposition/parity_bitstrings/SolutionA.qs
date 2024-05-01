namespace Kata {
    operation AllStatesWithParitySuperposition (qs : Qubit[], parity : Int) : Unit is Adj + Ctl {
        if Length(qs) == 1 {
            if parity == 1 {
                X(qs[0]);
            }
        } else {
            H(qs[0]);
            ApplyControlledOnInt(0, AllStatesWithParitySuperposition, qs[0 .. 0], (qs[1 ...], parity));
            ApplyControlledOnInt(1, AllStatesWithParitySuperposition, qs[0 .. 0], (qs[1 ...], 1 - parity));
        }
    }
}
