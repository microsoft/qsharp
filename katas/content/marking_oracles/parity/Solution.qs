namespace Kata {
    operation Oracle_Parity(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        for xi in x {
            CNOT(xi, y);
        }
    }
}
