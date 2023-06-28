namespace Quantum.Kata.Reference {

    // ------------------------------------------------------
    // Exercise 5.
    @EntryPoint()
    operation T5_RandomNumberInRange () : Unit {
        for (min, max) in [(1, 3), (27, 312), (0, 3), (0, 1023)] {
            Message($"Testing for min = {min} and max = {max}...");
            let solution = Delay(RandomNumberInRange, (min,max), _);
            let testingHarness = Delay(CheckUniformDistribution, (solution, min, max, 1000), _);
            RetryTestOperation(testingHarness);
            Message($"Test passed for min = {min} and max = {max}");
	    }
    }

    operation CheckRandomCalls () : Unit {
        Fact(GetOracleCallsCount(DrawRandomInt) == 0, "You are not allowed to call DrawRandomInt() in this task");
        Fact(GetOracleCallsCount(DrawRandomDouble) == 0, "You are not allowed to call DrawRandomDouble() in this task");
        ResetOracleCallsCount();
    }

}
