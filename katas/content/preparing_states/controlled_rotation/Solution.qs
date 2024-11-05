namespace Kata {
    operation ControlledRotation (qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        Controlled H ([qs[0]], qs[1]);
    }
}
