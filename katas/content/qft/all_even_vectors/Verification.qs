namespace Kata.Verification {
    import KatasUtils.*;

    operation AllEvenVectors_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        for q in qs[...Length(qs) - 2] {
            H(q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 1..5 {
            let solution = Kata.AllEvenVectors;
            let reference = AllEvenVectors_Reference;
            if not CheckOperationsEquivalenceOnZeroState(solution, reference, n) {
                Message($"Incorrect for {n} qubit(s)");
                ShowQuantumStateComparison(n, qs => (), solution, reference);
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
