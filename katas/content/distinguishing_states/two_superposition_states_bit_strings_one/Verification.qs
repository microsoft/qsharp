namespace Kata.Verification {
    import Std.Arrays.*;
    import Std.Convert.*;
    import KatasUtils.*;

    operation CheckSuperpositionBitstringsOneMeasurement(
        nQubits : Int,
        ints1 : Int[],
        ints2 : Int[]
    ) : Bool {
        let bits1 = Mapped(IntAsBoolArray(_, nQubits), ints1);
        let bits2 = Mapped(IntAsBoolArray(_, nQubits), ints2);

        let stateNames = [IntArrayAsStateName(nQubits, bits1), IntArrayAsStateName(nQubits, bits2)];

        let isCorrect = DistinguishStates_MultiQubit(
            nQubits,
            2,
            StatePrep_SuperpositionMeasurement(_, bits1, bits2, _, _),
            Kata.SuperpositionOneMeasurement(_, bits1, bits2),
            false,
            stateNames
        );

        if not isCorrect {
            Message($"Incorrect for: [{stateNames[0]}, {stateNames[1]}]")
        }

        return isCorrect;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        // note that bit strings in the comments (big endian) are the reverse of the bit strings passed to the solutions (little endian)
        for (n, ints1, ints2) in [
            (2, [2], [1]),                        // [10] vs [01]
            (2, [2, 3], [1, 0]),                  // [10,11] vs [01,00]
            (2, [2], [1, 0]),                     // [10] vs [01,00]
            (2, [0, 2], [3]),                     // [00,10] vs [11]
            (3, [5, 7], [2]),                     // [101,111] vs [010]
            (4, [15, 7], [0, 8]),                 // [1111,0111] vs [0000,1000]
            (4, [15, 7], [0, 8, 10, 12]),         // [1111,0111] vs [0000,1000,1010,1100]
            (4, [13, 11, 7, 3], [2, 4]),          // [1101,1011,0111,0011] vs [0010,0100]
            (5, [30, 14, 10, 6], [1, 17, 21, 25]) // [11110,01110,01010,00110] vs [00001,10001,10101,11001]
        ] {
            if not CheckSuperpositionBitstringsOneMeasurement(n, ints1, ints2) {
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
