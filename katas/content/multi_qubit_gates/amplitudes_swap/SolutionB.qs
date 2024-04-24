namespace Kata {
    operation AmplitudesSwap (qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyControlledOnInt(0, X, [qs[0]], qs[1]);
    }
}