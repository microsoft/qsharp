namespace Microsoft.Quantum.Samples.Qrng {
    open Microsoft.Quantum.Math; // BitSizeI
    open Microsoft.Quantum.Convert; // ResultArrayAsInt

    operation SampleQuantumRandomNumberGenerator() : Result {
        use q = Qubit();   // Allocate a qubit.
        H(q);              // Put the qubit to superposition.
                           // It now has a 50% chance of being 0 or 1.
        let result = M(q); // Measure the qubit value,
                           // but don't look at the result yet.
        Reset(q);          // Reset qubit to Zero state.
        return result;     // Return result of the measurement.
    }

    operation SampleRandomNumberInRange(max : Int) : Int {
        mutable bits = [];
        for idxBit in 1..BitSizeI(max) {
            set bits += [SampleQuantumRandomNumberGenerator()];
        }
        let sample = ResultArrayAsInt(bits);
        return sample > max
               ? SampleRandomNumberInRange(max)
               | sample;
    }

    @EntryPoint()
    operation Main() : Int {
        let max = 50;
        Message($"Sampling a random number between 0 and {max}: ");
        return SampleRandomNumberInRange(max);
    }
}
