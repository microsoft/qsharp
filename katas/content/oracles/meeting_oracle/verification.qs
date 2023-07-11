namespace Kata.Verification {
    open Microsoft.Quantum.Convert;

    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution(): Bool {
        for N in 1..4 {
            use jasmine = Qubit[N];
            for k in 0..(2^N-1) {
                let binaryJasmine = IntAsBoolArray(k, N);
                mutable isCorrect = false;
                within {
                    ApplyPauliFromBitString(PauliX, true, binaryJasmine, jasmine);
                } apply {
                    set isCorrect = CheckTwoOraclesAreEqual(
                        1..N,
                        Kata.Meeting_Oracle(_, jasmine, _),
                        Meeting_Oracle(_, jasmine, _));
                }
                if not isCorrect {
                    Message($"Failed on test case for N = {N}, k = {k}.");
                    return false;
                }
            }
        }
        Message("All tests passed.");
        true
    }

}
