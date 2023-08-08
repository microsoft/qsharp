namespace Kata.Verification {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;

    operation Or_Oracle_Reference(x: Qubit[], y: Qubit): Unit is Adj + Ctl {
        X(y);
        ApplyControlledOnInt(0, X, x, y);
    }

    operation Meeting_Oracle_Reference(x: Qubit[], jasmine: Qubit[], z: Qubit): Unit is Adj + Ctl {
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
                        Meeting_Oracle_Reference(_, jasmine, _));
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
