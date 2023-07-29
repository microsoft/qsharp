namespace Kata {
    operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
        // Apply the Pauli Y operation.
        Y(q);
    }
}