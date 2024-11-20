namespace Kata.Verification {
    import KatasUtils.*;

    operation KthBit_Oracle_Reference(x : Qubit[], k : Int) : Unit is Adj + Ctl {
        Z(x[k]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 1..3 {
            for k in 0..N - 1 {
                let sol = Kata.KthBit_Oracle(_, k);
                let ref = KthBit_Oracle_Reference(_, k);
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
