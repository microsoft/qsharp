namespace Kata {
    operation PrepareQubits(qs : Qubit[], bases : Bool[], bits : Bool[]) : Unit {
        for i in 0 .. Length(qs) - 1 {
            if bits[i] {
                X(qs[i]);
            }
            if bases[i] {
                H(qs[i]);
            }
        }
    }
}
