namespace Kata.Verification {
    import KatasUtils.*;

    operation Or_Oracle_Reference(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        X(y);
        ApplyControlledOnInt(0, X, x, y);
    }

    operation OrOfBitsExceptKth_Oracle_Reference(x : Qubit[], k : Int) : Unit is Adj + Ctl {
        use minus = Qubit();
        within {
            X(minus);
            H(minus);
        } apply {
            Or_Oracle_Reference(x[...k-1] + x[k + 1...], minus);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 2..4 {
            for k in 0..N - 1 {
                let sol = Kata.OrOfBitsExceptKth_Oracle(_, k);
                let ref = OrOfBitsExceptKth_Oracle_Reference(_, k);
                let isCorrect = CheckOperationsAreEqualStrict(N, sol, ref);

                if not isCorrect {
                    Message("Incorrect.");
                    Message("Hint: examine how your solution transforms the given state and compare it with the expected " +
                        $"transformation for the {N}-bit oracle for k = {k}");
                    ShowQuantumStateComparison(N, PrepDemoState, sol, ref);
                    return false;
                }
            }
        }
        Message("Correct!");
        true
    }
}
