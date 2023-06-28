namespace Kata.Solution {
    open Microsoft.Quantum.Intrinsic;

    operation ApplyX(q : Qubit) : Unit is Adj + Ctl {
        X(q);
    }
}