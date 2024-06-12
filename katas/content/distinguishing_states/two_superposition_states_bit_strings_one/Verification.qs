namespace Kata.Verification {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;

    operation CheckSuperpositionBitstringsOneMeasurement (
        nQubits : Int,
        ints1 : Int[],
        ints2 : Int[]
    ): Bool {
        let bits1 = Mapped(IntAsBoolArray(_, nQubits), ints1);
        let bits2 = Mapped(IntAsBoolArray(_, nQubits), ints2);

        let stateNames = [IntArrayAsStateName(nQubits, bits1), IntArrayAsStateName(nQubits, bits2)];

        let isCorrect =  DistinguishStates_MultiQubit(
            nQubits,
            2,
            StatePrep_SuperpositionMeasurement(_, bits1, bits2, _, _),
            Kata.SuperpositionOneMeasurement(_, bits1, bits2),
            true,
            stateNames
        );

        if isCorrect {
            Message("Correct!");
        } else{
            Message($"Incorrect for: [{stateNames[0]}, {stateNames[1]}]")
        }

        return isCorrect;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        mutable isCorrect = true;

        // note that bit strings in the comments (big endian) are the reverse of the bit strings passed to the solutions (little endian)
        set isCorrect = isCorrect and CheckSuperpositionBitstringsOneMeasurement(
            2,
            [2], // [10]
            [1] // [01]
        );

        set isCorrect = isCorrect and CheckSuperpositionBitstringsOneMeasurement(
            2,
            [2, 3], // [10,11]
            [1, 0] // [01,00]
        );

        set isCorrect = isCorrect and CheckSuperpositionBitstringsOneMeasurement(
            2,
            [2], // [10]
            [1, 0] // [01,00]
        );

        set isCorrect = isCorrect and CheckSuperpositionBitstringsOneMeasurement(
            4,
            [15, 7], // [1111,0111]
            [0, 8] // [0000,1000]
        );

        set isCorrect = isCorrect and CheckSuperpositionBitstringsOneMeasurement(
            4,
            [15, 7],       // [1111,0111]
            [0, 8, 10, 12] // [0000,1000,1010,1100]
        );

        set isCorrect = isCorrect and CheckSuperpositionBitstringsOneMeasurement(
            5,
            [30, 14, 10, 6], // [11110,01110,01010,00110]
            [1, 17, 21, 25] // [00001,10001,10101,11001]
        );

        set isCorrect = isCorrect and CheckSuperpositionBitstringsOneMeasurement(
            2,
            [0, 2], // [00,10]
            [3] // [11]
        );

        set isCorrect = isCorrect and CheckSuperpositionBitstringsOneMeasurement(
            3,
            [5, 7], // [101,111]
            [2] // [010]
        );

        return isCorrect and CheckSuperpositionBitstringsOneMeasurement(
            4,
            [13, 11, 7, 3], // [1101,1011,0111,0011]
            [2, 4] // [0010,0100]
        );
    }
}
