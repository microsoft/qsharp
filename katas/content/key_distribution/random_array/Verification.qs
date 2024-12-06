namespace Kata.Verification {
    import Std.Arrays.*;
    import Std.Convert.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        // The test only checks that the operation returns an array of correct length
        // and that it's not always the same. It doesn't analyze the distribution of true and false elements.
        let N = 20;
        let randomArrays = ForEach(Kata.RandomArray, [N, size = 10]);

        for array in randomArrays {
            if Length(array) != N {
                Message($"Returned array should have length {N}, and it had length {Length(array)}.");
                return false;
            }
        }

        let randomInts = Mapped(BoolArrayAsInt, randomArrays);
        mutable allSame = true;
        for int in randomInts {
            if int != randomInts[0] {
                set allSame = false;
            }
        }
        if allSame {
            Message($"Random generation should not return a fixed array.");
            return false;
        }

        Message("Correct!");
        true
    }
}
