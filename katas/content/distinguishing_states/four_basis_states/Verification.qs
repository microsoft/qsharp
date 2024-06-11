namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Katas;

    operation CheckSolution() : Bool {
        let isCorrect = DistinguishStates_MultiQubit(2, 4, StatePrep_BasisStateMeasurement, Kata.BasisStateMeasurement, false, ["|00⟩", "|01⟩", "|10⟩", "|11⟩"]);
        if (isCorrect) {
            Message("Correct!");
        } else {
            Message("Incorrect.");
        }

        isCorrect
    }
}
