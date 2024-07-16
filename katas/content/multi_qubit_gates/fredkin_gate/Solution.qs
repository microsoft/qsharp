namespace Kata {
    operation FredkinGate (qs : Qubit[]) : Unit is Adj + Ctl {
        Controlled SWAP([qs[0]], (qs[1], qs[2]));
    }
}