namespace Kata {
    open Microsoft.Quantum.Arrays;

    operation IsSeven_PhaseOracle(x : Qubit[]) : Unit is Adj + Ctl {
        Controlled Z(Most(x), Tail(x));
    }
}
