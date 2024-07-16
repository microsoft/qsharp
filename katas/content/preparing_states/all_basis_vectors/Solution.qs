namespace Kata {
    operation AllBasisVectorsSuperposition (qs : Qubit[]) : Unit is Adj + Ctl {
        for q in qs {
            H(q);
        }
    }
}
