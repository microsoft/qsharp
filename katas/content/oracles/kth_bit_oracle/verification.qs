namespace Quantum.Kata.Reference {

    // ------------------------------------------------------
    @EntryPoint()
    operation T32_KthBit_Oracle () : Unit {
        for N in 1..5 {
            for k in 0..(N-1) {
                within {
                    AllowAtMostNQubits(2*N, "You are not allowed to allocate extra qubits");
                } apply {
                    AssertOperationsEqualReferenced(N,
                                                KthBit_Oracle(_, k),
                                                KthBit_Oracle_Reference(_, k));
                }
            }
        }
    }

}
