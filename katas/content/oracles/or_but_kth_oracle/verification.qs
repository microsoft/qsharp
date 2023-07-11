namespace Kata.Verification {

    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution(): Bool {
        for N in 1..5 {
            for k in 0..(N-1) {
                let isCorrect = CheckOperationsEqualReferenced(
                    N,
                    Kata.OrOfBitsExceptKth_Oracle(_, k),
                    OrOfBitsExceptKth_Oracle(_, k));
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
