namespace Kata {
    operation AllBasisVectorsWithComplexPhases_TwoQubits (qs : Qubit[]) : Unit is Adj + Ctl {
        // Qubit 0 is converted into |+⟩ and then z-rotated into |-⟩.
        H(qs[0]);
        Z(qs[0]);

        // Qubit 1 is converted into |+⟩ and then z-rotated into |i⟩.
        H(qs[1]);
        S(qs[1]);
    }
}
