namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution(): Bool {
        let testCases = [(1, 3), (27, 312), (0, 3), (0, 1023)];
        for (min, max) in testCases {
            Message($"Testing for min = {min} and max = {max}...");
            let randomnessVerifier = () => CheckUniformDistribution(() =>
                Kata.RandomNumberInRange(min, max), min, max, 1000);
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
