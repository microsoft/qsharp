namespace Kata {
    operation FourBitstringSuperposition (qs : Qubit[], bits : Bool[][]) : Unit {
        use anc = Qubit[2];
        // Put two ancillas into equal superposition of 2-qubit basis states
        ApplyToEachA(H, anc);

        // Set up the right pattern on the main qubits with control on ancillas
        for i in 0 .. 3 {
            for j in 0 .. Length(qs) - 1 {
                if bits[i][j] {
                    ApplyControlledOnInt(i, X, anc, qs[j]);
                }
            }
        }

        // Uncompute the ancillas, using patterns on main qubits as control
        for i in 0 .. 3 {
            if i % 2 == 1 {
                ApplyControlledOnBitString(bits[i], X, qs, anc[0]);
            }
            if i / 2 == 1 {
                ApplyControlledOnBitString(bits[i], X, qs, anc[1]);
            }
        }
    }
}
