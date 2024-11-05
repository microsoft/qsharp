namespace Kata {
    operation RelativePhaseMinusOne (qs : Qubit[]) : Unit is Adj + Ctl {
        Controlled Z([qs[0]], qs[1]);
    }
}