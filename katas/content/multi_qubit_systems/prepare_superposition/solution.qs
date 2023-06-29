namespace Quantum.Kata.Reference {

    operation PrepareState2(qs : Qubit[]) : Unit is Adj+Ctl {
        X(qs[1]);
        H(qs[1]);
    }

}
