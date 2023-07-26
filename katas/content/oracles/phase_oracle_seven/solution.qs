namespace Kata {

    open Microsoft.Quantum.Arrays;

    // Task 1.2.
    operation IsSeven_PhaseOracle(x: Qubit[]): Unit is Adj + Ctl {
        Controlled Z(Most(x), Tail(x));
    }

}
