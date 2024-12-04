namespace Kata.Verification {
    import Std.StatePreparation.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let eigenvectors = [
            (Z, I, "Z, |0⟩"),
            (Z, X, "Z, |1⟩"),
            (S, I, "S, |0⟩"),
            (S, X, "S, |1⟩"),
            (X, H, "X, |+⟩"),
            (X, q => PreparePureStateD([1., -1.], [q]), "X, |-⟩")
        ];
        for (U, P, msg) in eigenvectors {
            if not Kata.IsEigenvector(U, P) {
                Message($"Incorrect for (U, P) = ({msg}): expected true");
                return false;
            }
        }

        let notEigenvectors = [
            (Z, H, "Z, |+⟩"),
            (X, X, "X, |1⟩"),
            (X, Z, "X, |0⟩"),
            (Y, H, "Y, |+⟩"),
            (Y, X, "Y, |1⟩")
        ];
        for (U, P, msg) in notEigenvectors {
            if Kata.IsEigenvector(U, P) {
                Message($"Incorrect for (U, |ψ⟩) = ({msg}): expected false");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
