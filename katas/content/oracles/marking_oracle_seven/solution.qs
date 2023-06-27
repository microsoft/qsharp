namespace Quantum.Kata.Reference {

    // Task 1.3.
    operation IsSeven_MarkingOracle_Reference (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        Controlled X(x, y);
    }

}
