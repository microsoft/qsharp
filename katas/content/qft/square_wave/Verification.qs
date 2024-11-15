namespace Kata.Verification {
    import KatasUtils.*;

    operation SquareWave_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        for q in qs {
            H(q);
        }
        Z(qs[Length(qs) - 2]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 2..5 {
            let solution = Kata.SquareWave;
            let reference = SquareWave_Reference;
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
