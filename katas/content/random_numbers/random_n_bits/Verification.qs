namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution() : Bool {
        // Test random number generation for 1, 2, 3, 10 bits
        let testCases = [(1, 1000), (2, 1000), (3, 1000), (10, 10000)];
        for (N, runs) in testCases {
            Message($"Testing N = {N}...");
            let max = (1 <<< N) - 1;
            let randomnessVerifier = () => CheckUniformDistribution(() =>
                Kata.RandomNBits(N), 0, max, runs);
            let isCorrect = IsSufficientlyRandom(randomnessVerifier);
            if not isCorrect {
                return false;
            }
            Message($"Test passed for N = {N}");
        }
        Message("All tests passed.");
        true
    }

}
