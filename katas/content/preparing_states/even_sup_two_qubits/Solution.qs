namespace Kata {
    operation AllBasisVectors_TwoQubits (qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        H(qs[1]);
    }
}
