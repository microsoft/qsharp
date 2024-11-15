namespace Kata.Verification {
    import KatasUtils.*;

    operation StatePrep_Bitstring(
        qs : Qubit[],
        bits : Bool[]
    ) : Unit is Adj {
        for i in 0..Length(qs) - 1 {
            if bits[i] {
                X(qs[i]);
            }
        }
    }

    operation StatePrep_TwoBitstringsMeasurement(
        qs : Qubit[],
        bits1 : Bool[],
        bits2 : Bool[],
        state : Int,
        dummyVar : Double
    ) : Unit is Adj {
        let bits = state == 0 ? bits1 | bits2;
        StatePrep_Bitstring(qs, bits);
    }

    operation CheckTwoBitstringsMeasurement(b1 : Bool[], b2 : Bool[]) : Bool {
        let stateNames = [BoolArrayAsKetState(b1), BoolArrayAsKetState(b2)];
        let isCorrect = DistinguishStates_MultiQubit(
            Length(b1),
            2,
            StatePrep_TwoBitstringsMeasurement(_, b1, b2, _, _),
            Kata.TwoBitstringsMeasurement(_, b1, b2),
            false,
            stateNames
        );

        if not isCorrect {
            Message($"Incorrect for [{stateNames[0]}, {stateNames[1]}].");
        }

        return isCorrect;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for (b1, b2) in [
            ([false, true], [true, false]),
            ([true, true, false], [false, true, true]),
            ([false, true, true, false], [false, true, true, true]),
            ([true, false, false, false], [true, false, true, true])
        ] {
            if not CheckTwoBitstringsMeasurement(b1, b2) {
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
