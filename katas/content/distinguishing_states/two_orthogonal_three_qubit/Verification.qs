namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Math.*;

    operation StatePrep_ThreeQubitMeasurement(
        qs : Qubit[],
        state : Int,
        dummyVar : Double
    ) : Unit is Adj {
        WState_Arbitrary_Reference(qs);

        if state == 0 {
            // prep 1/sqrt(3) ( |100⟩ + ω |010⟩ + ω² |001⟩ )
            R1(2.0 * PI() / 3.0, qs[1]);
            R1(4.0 * PI() / 3.0, qs[2]);
        } else {
            //  prep 1/sqrt(3) ( |100⟩ + ω² |010⟩ + ω |001⟩ )
            R1(4.0 * PI() / 3.0, qs[1]);
            R1(2.0 * PI() / 3.0, qs[2]);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = DistinguishStates_MultiQubit(
            3,
            2,
            StatePrep_ThreeQubitMeasurement,
            Kata.ThreeQubitMeasurement,
            false,
            ["1/sqrt(3) (|100⟩ + ω |010⟩ + ω² |001⟩)", "1/sqrt(3) (|100⟩ + ω² |010⟩ + ω |001⟩)"]
        );

        if not isCorrect {
            Message("Incorrect.");
        } else {
            Message("Correct!");
        }

        return isCorrect
    }
}
