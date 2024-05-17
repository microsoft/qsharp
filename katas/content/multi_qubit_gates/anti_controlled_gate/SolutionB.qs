namespace Kata {
    operation AntiControlledGate (qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyControlledOnInt(0, X, [qs[0]], qs[1]);
    }
}