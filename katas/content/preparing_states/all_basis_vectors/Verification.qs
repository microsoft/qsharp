namespace Kata.Verification {
    import KatasUtils.*;

    operation AllBasisVectorsSuperposition_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        for q in qs {
            H(q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for i in 1..5 {
            Message($"Testing {i} qubit(s)...");
            if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                Kata.AllBasisVectorsSuperposition,
                AllBasisVectorsSuperposition_Reference,
                i
            ) {
                return false;
            }
        }

        return true;


    }
}
