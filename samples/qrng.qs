namespace Microsoft.Quantum.Samples.Qrng {
    open Microsoft.Quantum.Math; // BitSizeI
    open Microsoft.Quantum.Convert; // ResultArrayAsInt

    operation SampleQuantumRandomNumberGenerator() : Result {

        // Allocate a qubit.
        use q = Qubit();

        // Put the qubit into superposition of 0 and 1.
        // It now has a 50% chance of being measured as 0 or 1.
        H(q);

        // Measure the qubit value, but don't look at the result yet.
        let result = M(q);

        // Reset qubit to Zero state.          
        Reset(q);

        // Return the result of the measurement.
        result

        // Note that Qubit `q` is automatically released
        // at the end of the block.
    }

    operation SampleRandomNumberInRange(max : Int) : Int {
        mutable bits = [];
        for idxBit in 1..BitSizeI(max) {
            set bits += [SampleQuantumRandomNumberGenerator()];
        }
        let sample = ResultArrayAsInt(bits);

        // Return random number if it is within the requested range
        // Or sample it again if it is outside the range.
        sample > max ? SampleRandomNumberInRange(max) | sample
    }

    @EntryPoint()
    operation Main() : Int {
        let max = 50;
        Message($"Sampling a random number between 0 and {max}: ");

        // Draw and return random number from 0..max range.
        SampleRandomNumberInRange(max)
    }
}
