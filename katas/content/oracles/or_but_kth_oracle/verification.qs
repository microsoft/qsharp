namespace Quantum.Kata.Reference {

    // ------------------------------------------------------
    @EntryPoint()
    operation T33_OrOfBitsExceptKth_Oracle () : Unit {
        for N in 1..5 {
            for k in 0..(N-1) {
                AssertOperationsEqualReferenced(N,
                                                OrOfBitsExceptKth_Oracle(_, k),
                                                OrOfBitsExceptKth_Oracle_Reference(_, k));
            }
        }
    }

}
