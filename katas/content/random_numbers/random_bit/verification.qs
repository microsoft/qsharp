namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution(): Bool {
        Message("Testing one random bit generation...");
        let result = RetryTestOperation(() => CheckUniformDistribution(Kata.RandomBit, 0, 1, 1000));
        if result {
            Message("All tests passed.");	
	}
        return result;
    }

}
