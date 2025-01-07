namespace Kata.Verification {
    import Std.Diagnostics.*;
    import Std.Arrays.*;
    import Std.Convert.*;
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for nBits in 1..3 {
            for V in 1..3 {
                use qs = Qubit[V * nBits];
                for state in 0..(1 <<< (V * nBits)) - 1 {
                    // Prepare the register in the input state
                    let binaryState = IntAsBoolArray(state, V * nBits);
                    ApplyPauliFromBitString(PauliX, true, binaryState, qs);

                    // Call the solution
                    let result = Kata.ReadColoring(nBits, qs);

                    // verify that the register remained in the same state
                    ApplyPauliFromBitString(PauliX, true, binaryState, qs);
                    if not CheckAllZero(qs) {
                        Message("The input state should not change");
                        ResetAll(qs);
                        return false;
                    }

                    // Get the expected coloring by splitting binaryState into parts and converting them into integers
                    // (remember to use big endian)
                    let partitions = Chunks(nBits, binaryState);
                    let partitionToInt = bits -> BoolArrayAsInt(Reversed(bits));
                    let expectedColors = Mapped(partitionToInt, partitions);

                    // Verify the return value
                    if Length(result) != V {
                        Message($"Unexpected number of colors for V = {V}, nBits = {nBits} : {Length(result)}");
                        return false;
                    }
                    for (expected, actual) in Zipped(expectedColors, result) {
                        if expected != actual {
                            Message($"Unexpected colors for V = {V}, nBits = {nBits}, " +
                                $"state = {BoolArrayAsKetState(binaryState)} : expected {expectedColors}, got {result}");
                            return false;
                        }
                    }
                }
            }
        }
        Message("Correct!");
        true
    }
}
