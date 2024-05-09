namespace Kata {
    operation TwoBitstringSuperposition (qs : Qubit[], bits1 : Bool[], bits2 : Bool[]) : Unit is Adj + Ctl {
        use q = Qubit();
        H(q);

        for i in 0 .. Length(qs) - 1 {
            if bits1[i] {
                ApplyControlledOnInt(0, X, [q], qs[i]);
            }
            if bits2[i] {
                ApplyControlledOnInt(1, X, [q], qs[i]);
            }
        }

        // uncompute the auxiliary qubit to release it
        ApplyControlledOnBitString(bits2, X, qs, q);
    }    
}
