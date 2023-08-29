namespace Kata {
    operation FlipQubit(q : Qubit): Unit is Adj + Ctl {
        // Perform a "bit flip" on the qubit by applying the X gate.
        X(q);
    }
}