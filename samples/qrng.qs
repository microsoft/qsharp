namespace Microsoft.Quantum.Samples.Qrng {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Diagnostics;

    operation SampleQuantumRandomNumberGenerator() : Result {
        use q = Qubit();   // Allocate a qubit.
        H(q);              // Put the qubit to superposition. It now has a 50% chance of being 0 or 1.
        let result = M(q); // Measure the qubit value, but don't look at the result yet.
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

    function ResultArrayAsInt(input : Result[]) : Int {
        let nBits = Length(input);
        Fact(nBits < 64, "`Length(bits)` must be less than 64, but was " +
            AsString(nBits) + ".");
        mutable number = 0;
        mutable power = 1;
        for idxBit in 0 .. nBits - 1 {
            if (input[idxBit] == One) {
                set number = number ||| power;
            }
            set power = power <<< 1;
        }
        return number;
    }

    @EntryPoint()
    operation Main() : Int {
        let max = 50;
        Message("Sampling a random number between 0 and " +
            AsString(max) + ": ");
        return SampleRandomNumberInRange(max);
    }
}
