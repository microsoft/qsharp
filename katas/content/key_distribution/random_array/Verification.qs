namespace Kata.Verification {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;

    @EntryPoint()
    operation CheckSolution() : Bool {
        // The test only checks that the operation returns an array of correct length
        // and that it's not always the same. It doesn't analyze the distribution of true and false elements.
        let N = 10;
        let randomArrays = ForEach(Kata.RandomArray, [N, N, N]);

        for array in randomArrays {
            if Length(array) != N {
                Message($"Returned array should have length {N}, and it had length {Length(array)}.");
                return false;
            }
        }

        let randomInts = Mapped(BoolArrayAsInt, randomArrays);
        if randomInts[0] == randomInts[1] or randomInts[1] == randomInts[2] or randomInts[0] == randomInts[2] {
            Message($"Random generation should not return a fixed array.");
            return false;
        }

        Message("Correct!");
        true
    }
}
