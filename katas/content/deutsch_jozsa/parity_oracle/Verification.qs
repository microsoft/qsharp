namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;

    operation PhaseOracle_Parity_Reference(x : Qubit[]) : Unit is Adj + Ctl {
        for xi in x {
            Z(xi);
        }
    }

    operation CheckSolution() : Bool {
        let solution = Kata.PhaseOracle_Parity;
        let reference = PhaseOracle_Parity_Reference;
        for N in 1 .. 4 {
            if not CheckOperationsEquivalenceStrict(solution, reference, N) {
                Message("Incorrect.");
                Message($"Hint: examine the effect your solution has on the {N}-qubit and compare it with the effect it " +
                    "is expected to have. Note that the simulator might drop the global phase -1, so if you're getting " +
                    "verdict \"Incorrect\" but the actual state matches the expected one, check that you're handling the global phase correctly.");
                use initial = Qubit[N];
                PrepRandomState(initial);
                ShowQuantumStateComparison(initial, solution, reference);
                ResetAll(initial);
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
