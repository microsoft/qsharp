namespace Quantum.Kata.Reference {

    operation PrepareState1(qs : Qubit[]) : Unit is Adj+Ctl {
        X(qs[0]);
        X(qs[1]);
    }

}
