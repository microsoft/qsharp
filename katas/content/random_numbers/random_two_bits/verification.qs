namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution(): Bool {
        Message("Testing two random bits generation...");

        if not RetryTestOperation(() =>
            CheckUniformDistribution(Kata.RandomTwoBits, 0, 3, 1000)) {
            return false;
        }
        Message("All tests passed.");
        return true;
    }

}
