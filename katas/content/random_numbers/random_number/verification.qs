namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution(): Bool {

        for (min, max) in [(1, 3), (27, 312), (0, 3), (0, 1023)] {
            Message($"Testing for min = {min} and max = {max}...");

            if not RetryTestOperation(() =>
                CheckUniformDistribution(() =>
                    Kata.RandomNumberInRange(min, max), min, max, 1000)) {
                return false;
            }

            Message($"Test passed for min = {min} and max = {max}");
        }
        Message("All tests passed.");
        return true;
    }

}
