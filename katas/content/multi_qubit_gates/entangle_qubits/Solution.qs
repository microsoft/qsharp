namespace Kata {
    operation EntangleQubits (qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
    }
}