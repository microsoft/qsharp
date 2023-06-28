namespace Quantum.Kata.Reference {

    // ------------------------------------------------------
    // Exercise 1.
    @EntryPoint()
    operation T1_RandomBit () : Unit {
        Message("Testing one random bit generation...");
        let solution = RandomBit;
        // Delay() converts CheckUniformDistribution to a parameterless operation
        let testingHarness = Delay(CheckUniformDistribution, (solution, 0, 1, 1000), _);
        RetryTestOperation(testingHarness);
        Message("Test passed");
    }

}
