namespace Kata.Verification {
    import Std.StatePreparation.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let eigenvectors = [
            (Z, I, 1, "Z, |0⟩"),
            (Z, X, -1, "Z, |1⟩"),
            (S, I, 1, "S, |0⟩"),
            (X, H, 1, "X, |+⟩"),
            (X, q => PreparePureStateD([1., -1.], [q]), -1, "X, |-⟩")
        ];
        for (U, P, expected, msg) in eigenvectors {
            let actual = Kata.OneBitPhaseEstimation(U, P);
            if actual != expected {
                Message($"Incorrect eigenvalue for (U, |ψ⟩) = ({msg}): expected {expected}, got {actual}");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
