namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation StatePrep_Bitstring(
        qs : Qubit[],
        bits : Bool[]
    ) : Unit is Adj {
        for i in 0 .. Length(qs) - 1 {
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
            true,
            stateNames
        );

        if (isCorrect) {
            Message("Correct!");
        } else {
            Message($"Incorrect for [{stateNames[0]}, {stateNames[1]}].");
        }

        return isCorrect;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        mutable isCorrect = true;

        mutable b1 = [false, true];
        mutable b2 = [true, false];
        set isCorrect = isCorrect and CheckTwoBitstringsMeasurement(b1, b2);

        set b1 = [true, true, false];
        set b2 = [false, true, true];
        set isCorrect = isCorrect and CheckTwoBitstringsMeasurement(b1, b2);

        set b1 = [false, true, true, false];
        set b2 = [false, true, true, true];
        set isCorrect = isCorrect and CheckTwoBitstringsMeasurement(b1, b2);

        set b1 = [true, false, false, false];
        set b2 = [true, false, true, true];
        return isCorrect and CheckTwoBitstringsMeasurement(b1, b2);
    }
}
