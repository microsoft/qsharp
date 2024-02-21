namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution() : Bool {
        let testCases = [(1, 3, 1000), (27, 312, 5000), (0, 3, 1000), (0, 1023, 10000)];
        for (min, max, runs) in testCases {
            Message($"Testing for min = {min} and max = {max}...");
            let randomnessVerifier = () => CheckUniformDistribution(() =>
                Kata.RandomNumberInRange(min, max), min, max, runs);
            let isCorrect = IsSufficientlyRandom(randomnessVerifier);
            if not isCorrect {
                return false;
            }

            Message($"Test passed for min = {min} and max = {max}");
        }
        Message("All tests passed.");
        true
    }
}
