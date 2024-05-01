namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;

    operation WState_PowerOfTwo_Reference (qs : Qubit[]) : Unit is Adj+Ctl {
        let N = Length(qs);

        if N == 1 {
            // base of recursion: |1⟩
            X(qs[0]);
        } else {
            let K = N / 2;
            use anc = Qubit();
            H(anc);

            ApplyControlledOnInt(0, WState_PowerOfTwo_Reference, [anc], qs[0 .. K - 1]);
            ApplyControlledOnInt(1, WState_PowerOfTwo_Reference, [anc], qs[K .. N - 1]);

            for i in K .. N - 1 {
                CNOT(qs[i], anc);
            }
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.WState_PowerOfTwo,
            ApplyToEachA(X, _),
            1) {
            return false;
        }

        return true;
    }
}
