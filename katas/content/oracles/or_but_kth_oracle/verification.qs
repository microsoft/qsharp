namespace Kata.Verification {

    operation Or_Oracle_Reference(x: Qubit[], y: Qubit): Unit is Adj + Ctl {
        X(y);
        ApplyControlledOnInt(0, X, x, y);
    }

    // Task 3.3.
    operation OrOfBitsExceptKth_Oracle_Reference(x: Qubit[], k: Int): Unit is Adj + Ctl {
        use minus = Qubit();
        within {
            X(minus);
            H(minus);
        } apply {
            Or_Oracle(x[...k-1] + x[k+1...], minus);
        }
    }

    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution(): Bool {
        for N in 1..5 {
            for k in 0..(N-1) {
                let isCorrect = CheckOperationsEqualReferenced(
                    N,
                    Kata.OrOfBitsExceptKth_Oracle(_, k),
                    OrOfBitsExceptKth_Oracle_Reference(_, k));
                if not isCorrect {
                    Message($"Failed on test case for NumberOfQubits = {N}, k = {k}.");
                    return false;
                }
            }
        }
        Message("All tests passed.");
        true
    }

}
