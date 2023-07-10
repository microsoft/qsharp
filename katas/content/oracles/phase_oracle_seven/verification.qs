namespace Kata.Verification {

    // ------------------------------------------------------
    @EntryPoint()
    operation T12_IsSeven_PhaseOracle () : Unit {
        let N = 3;
        within {
            AllowAtMostNQubits(2*N, "You are not allowed to allocate extra qubits"); // This could be no-op
        } apply {
            AssertOperationsEqualReferenced(N, IsSeven_PhaseOracle, IsSeven_PhaseOracle_Reference);
        }
    }

}
