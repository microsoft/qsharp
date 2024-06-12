namespace Kata.Verification{
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;

    operation CheckSuperpositionBitstringsMeasurement (
        nQubits : Int,
        ints1 : Int[],
        ints2 : Int[]
    ): Bool {
        let bits1 = Mapped(IntAsBoolArray(_, nQubits), ints1);
        let bits2 = Mapped(IntAsBoolArray(_, nQubits), ints2);

        let stateNames = [IntArrayAsStateName(nQubits, bits1), IntArrayAsStateName(nQubits, bits2)];

        let isCorrect = DistinguishStates_MultiQubit(nQubits, 2, StatePrep_SuperpositionMeasurement(_, bits1, bits2, _, _),
                                     Kata.SuperpositionMeasurement(_, bits1, bits2), false,
                                     stateNames);

        if isCorrect {
            Message("Correct!");
        } else{
            Message($"Incorrect for: [{stateNames[0]}, {stateNames[1]}]")
        }

        return isCorrect;
    }

    operation CheckSolution () : Bool {
        mutable isCorrect = true;

        // note that bit strings in the comments (big endian) are the reverse of the bit strings passed to the solutions (little endian)
        set isCorrect = isCorrect and CheckSuperpositionBitstringsMeasurement(
            2,
            [2],  // [10]
            [1]
        ); // [01]

        set isCorrect = isCorrect and CheckSuperpositionBitstringsMeasurement(
            2,
            [2,1], // [10,01]
            [3,0] // [11,00]
        );

        set isCorrect = isCorrect and CheckSuperpositionBitstringsMeasurement(
            2,
            [2],    // [10]
            [3,0]) // [11,00]
        ;

        set isCorrect = isCorrect and CheckSuperpositionBitstringsMeasurement(
            4,
            [15,6], // [1111,0110]
            [0,14] // [0000,1110]
        );

        set isCorrect = isCorrect and CheckSuperpositionBitstringsMeasurement(
            4,
            [15,7],      // [1111,0111]
            [0,8,10,13] // [0000,1000,1010,1101]
        );

        set isCorrect = isCorrect and CheckSuperpositionBitstringsMeasurement(
            5,
            [30,14,10,7], // [11110,01110,01010,00111]
            [1,17,21,27] // [00001,10001,10101,11011]
        );

        set isCorrect = isCorrect and CheckSuperpositionBitstringsMeasurement(
            2,
            [2,1], // [10,01]
            [3] // [11]
        );

        set isCorrect = isCorrect and CheckSuperpositionBitstringsMeasurement(
            3,
            [7,5], // [111,101]
            [2] // [010]
        );

        return isCorrect and CheckSuperpositionBitstringsMeasurement(
            4,
            [13,11,7,3], // [1101,1011,0111,0011]
            [5,2] // [0101,0010]
        );
    }
}
