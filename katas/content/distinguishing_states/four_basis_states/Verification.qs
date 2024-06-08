namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Katas;

    operation StatePrep_BasisStateMeasurement(qs : Qubit[], state : Int, dummyVar : Double) : Unit is Adj {

        if state / 2 == 1 {
            // |10⟩ or |11⟩
            X(qs[0]);
        }

        if state % 2 == 1 {
            // |01⟩ or |11⟩
            X(qs[1]);
        }
    }

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
