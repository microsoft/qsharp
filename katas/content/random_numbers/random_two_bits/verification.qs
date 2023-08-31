namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution(): Bool {
        Message("Testing two random bits generation...");
        let randomnessVerifier = () => CheckUniformDistribution(Kata.RandomTwoBits, 0, 3, 1000);
        let isCorrect = IsSufficientlyRandom(randomnessVerifier);
        if isCorrect {
            Message("All tests passed.");
        }
        isCorrect
    }

}
