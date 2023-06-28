namespace Quantum.Kata.Reference {

    // ------------------------------------------------------
    // Exercise 2.
    @EntryPoint()
    operation T2_RandomTwoBits () : Unit {
        Message("Testing two random bits generation...");
        let solution = RandomTwoBits;
        // Delay() converts CheckUniformDistribution to a parameterless operation
        let testingHarness = Delay(CheckUniformDistribution, (solution, 0, 3, 1000), _);
        RetryTestOperation(testingHarness);
        Message("Test passed");
    }

}
