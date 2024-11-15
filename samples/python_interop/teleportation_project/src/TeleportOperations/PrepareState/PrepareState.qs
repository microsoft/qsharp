operation PrepareBellPair(left : Qubit, right : Qubit) : Unit is Adj + Ctl {
    H(left);
    CNOT(left, right);
}
