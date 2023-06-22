namespace Kata.Solution {
    open Microsoft.Quantum.Intrinsic;

    operation PrepareBasisState(register : Qubit[]) : Unit is Adj + Ctl {
        // TODO: Explain.
        X(register[1]);
        H(register[1]);
    }
}