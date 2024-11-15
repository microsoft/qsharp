namespace Kata.Verification {
    import Std.Convert.*;
    import Std.Math.*;
    import Std.Katas.*;

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
