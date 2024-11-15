namespace Kata.Verification {
    import KatasUtils.*;

    operation AlternatingAmplitudes_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        for q in qs {
            H(q);
        }
        Z(qs[Length(qs) - 1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 1..5 {
            let solution = Kata.AlternatingAmplitudes;
            let reference = AlternatingAmplitudes_Reference;
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
