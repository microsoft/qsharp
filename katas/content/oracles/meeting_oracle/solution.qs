namespace Quantum.Kata.Reference {

    // Task 4.3.
    operation Meeting_Oracle_Reference (x : Qubit[], jasmine : Qubit[], z : Qubit) : Unit is Adj + Ctl {
        use q = Qubit[Length(x)];
        within {
            for i in IndexRange(q) {
                // flip q[i] if both x and jasmine are free on the given day
                X(x[i]);
                X(jasmine[i]);
                CCNOT(x[i], jasmine[i], q[i]);
            }
        } apply {
            Or_Oracle_Reference(q, z);
        }
    }

}
