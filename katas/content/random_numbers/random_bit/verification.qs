namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution() : Bool {
        Message("Testing one random bit generation...");
        let randomnessVerifier = () => CheckUniformDistribution(Kata.RandomBit, 0, 1, 1000);
        let isCorrect = IsSufficientlyRandom(randomnessVerifier);
        if isCorrect {
            Message("All tests passed.");	
	    }
        isCorrect
    }

}
