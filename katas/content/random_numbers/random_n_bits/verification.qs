namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution(): Bool {
        // Test random number generation for 1, 2, 3, 10 bits
        for N in [1, 2, 3, 10] {
            Message($"Testing N = {N}...");
            let max = (1 <<< N) - 1;
            let randomnessVerifier = () => CheckUniformDistribution(() =>
                Kata.RandomNBits(N), 0, max, 1000);
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
