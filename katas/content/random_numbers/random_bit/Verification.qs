namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution() : Bool {
        let randomnessVerifier = () => CheckUniformDistribution(Kata.RandomBit, 0, 1, 1000);
        let isCorrect = IsSufficientlyRandom(randomnessVerifier);
        if isCorrect {
            Message("Correct!");	
	    }
        isCorrect
    }

}
