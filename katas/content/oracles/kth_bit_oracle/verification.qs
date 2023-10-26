namespace Kata.Verification {
    operation KthBit_Oracle_Reference(x : Qubit[], k : Int) : Unit is Adj + Ctl {
        Z(x[k]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 1..5 {
            for k in 0..(N-1) {
                let isCorrect = CheckOperationsEqualReferenced(
                    N,
                    Kata.KthBit_Oracle(_, k),
                    KthBit_Oracle_Reference(_, k));
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
