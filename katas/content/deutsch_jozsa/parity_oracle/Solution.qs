namespace Kata {
    operation PhaseOracle_Parity(x : Qubit[]) : Unit is Adj + Ctl {
        for xi in x {
            Z(xi);
        }
    }
}
