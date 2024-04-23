namespace Kata {
    operation RelativePhaseMinusOne (qs : Qubit[]) : Unit is Adj + Ctl {
        CZ(qs[0], qs[1]);
    }
}