namespace Kata.Verification {
    import KatasUtils.*;

    operation AllBasisVectors_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        for q in qs {
            H(q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 1..5 {
            let solution = Kata.AllBasisVectors;
            let reference = AllBasisVectors_Reference;
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
