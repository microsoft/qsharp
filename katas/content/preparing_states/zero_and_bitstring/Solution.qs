namespace Kata {
    operation ZeroAndBitstringSuperposition (qs : Qubit[], bits : Bool[]) : Unit is Adj + Ctl {
        H(qs[0]);

        for i in 1 .. Length(qs) - 1 {
            if bits[i] {
                CNOT(qs[0], qs[i]);
            }
        }
    }
}
