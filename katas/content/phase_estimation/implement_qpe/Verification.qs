namespace Kata.Verification {
    import Std.StatePreparation.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let tests = [
            (Z, I, 1, 0, "Z, |0⟩"),
            (Z, X, 1, 1, "Z, |1⟩"),
            (X, H, 1, 0, "X, |+⟩"),
            (X, q => PreparePureStateD([1., -1.], [q]), 1, 1, "X, |-⟩"),
            (S, I, 2, 0, "S, |0⟩"),
            (S, X, 2, 1, "S, |1⟩"),
            (Z, X, 2, 2, "Z, |1⟩"), // Higher precision than necessary
            (T, I, 3, 0, "T, |0⟩"),
            (T, X, 3, 1, "T, |1⟩"),
            (S, X, 3, 2, "S, |1⟩"), // Higher precision than necessary
            (Z, X, 3, 4, "Z, |1⟩"), // Higher precision than necessary
        ];
        for (U, P, n, expected, msg) in tests {
            for _ in 1..10 {
                // Repeat several times to catch probabilistic failures
                let actual = Kata.PhaseEstimation(U, P, n);
                if actual != expected {
                    Message($"Incorrect eigenphase for (U, |ψ⟩, n) = ({msg}, {n}): expected {expected}, got {actual}");
                    return false;
                }
            }
        }

        Message("Correct!");
        return true;
    }
}
