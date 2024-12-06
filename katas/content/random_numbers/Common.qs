namespace Kata.Verification {
    import Std.Arrays.*;
    import Std.Diagnostics.*;
    import Std.Convert.*;
    import Std.Math.*;

    /// # Summary
    /// Helper operation that checks that the given RNG operation generates a uniform distribution.
    ///
    /// # Input
    /// ## randomGenerator
    /// Random number generation operation to be tested.
    /// The parameters to this operation are provided by the caller using Delay().
    /// ## min, max
    /// Minimal and maximal numbers in the range to be generated, inclusive.
    /// ## nRuns
    /// The number of random numbers to generate for test.
    ///
    /// # Output
    /// 0x0 if the generated distribution is uniform.
    /// 0x1 if a value was generated outside the specified range.
    /// 0x2 if the average of the distribution is outside the expected range.
    /// 0x3 if the median of the distribution is outside the expected range.
    /// 0x4 if the minimum count requirements were not met.
    operation CheckUniformDistribution(
        randomGenerator : (Unit => Int),
        min : Int,
        max : Int,
        nRuns : Int
    ) : Int {
        let idealMean = 0.5 * IntAsDouble(max + min);
        let rangeDividedByTwo = 0.5 * IntAsDouble(max - min);
        // Variance = a*(a+1)/3, where a = (max-min)/2
        // For sample population : divide it by nRuns
        let varianceInSamplePopulation = (rangeDividedByTwo * (rangeDividedByTwo + 1.0)) / IntAsDouble(3 * nRuns);
        let standardDeviation = Sqrt(varianceInSamplePopulation);

        // lowRange : The lower bound of the median and average for generated dataset
        // highRange : The upper bound of the median and average for generated dataset
        // Set them with 3 units of std deviation for 99% accuracy.
        let lowRange = idealMean - 3.0 * standardDeviation;
        let highRange = idealMean + 3.0 * standardDeviation;

        let idealCopiesGenerated = IntAsDouble(nRuns) / IntAsDouble(max-min + 1);
        let minimumCopiesGenerated = (0.8 * idealCopiesGenerated > 40.0) ? 0.8 * idealCopiesGenerated | 0.0;

        mutable counts = [0, size = max + 1];
        mutable average = 0.0;
        for i in 1..nRuns {
            let val = randomGenerator();
            if (val < min or val > max) {
                Message($"Unexpected number generated. Expected values from {min} to {max}, generated {val}");
                return 0x1;
            }
            set average += IntAsDouble(val);
            set counts w/= val <- counts[val] + 1;
        }

        set average = average / IntAsDouble(nRuns);
        if (average < lowRange or average > highRange) {
            Message($"Unexpected average of generated numbers. Expected between {lowRange} and {highRange}, got {average}");
            return 0x2;
        }

        let median = FindMedian(counts, max + 1, nRuns);
        if (median < Floor(lowRange) or median > Ceiling(highRange)) {
            Message($"Unexpected median of generated numbers. Expected between {Floor(lowRange)} and {Ceiling(highRange)}, got {median}.");
            return 0x3;
        }

        for i in min..max {
            if counts[i] < Floor(minimumCopiesGenerated) {
                Message($"Unexpectedly low number of {i}'s generated. Only {counts[i]} out of {nRuns} were {i}");
                return 0x4;
            }
        }
        return 0x0;
    }

    operation FindMedian(counts : Int[], arrSize : Int, sampleSize : Int) : Int {
        mutable totalCount = 0;
        for i in 0..arrSize - 1 {
            set totalCount = totalCount + counts[i];
            if totalCount >= sampleSize / 2 {
                return i;
            }
        }
        return -1;
    }

    operation IsSufficientlyRandom(verifier : (Unit => Int)) : Bool {
        let results = RunRandomnessVerifier(verifier, 10);
        Tail(results) == 0x0
    }

    /// # Summary
    /// Helper operation that runs a randomness verifier up to a maximum number of times.
    /// A single run can fail with non-negligible probability even for a "correct" random generator.
    ///
    /// # Input
    /// ## verifier
    /// Operation which verifies the a random generator.
    /// ## maxAttempts
    /// Maximum number of times the verifier is run until a successful result occurs.
    ///
    /// # Output
    /// Array with the results of each verifier run.
    operation RunRandomnessVerifier(verifier : (Unit => Int), maxAttempts : Int) : Int[] {
        mutable attemptResults = [];
        mutable result = -1;
        repeat {
            set result = verifier();
            set attemptResults += [result];
            // If the result is 0x1, the generator returned an invalid result.
            // That's different from "not random enough" verdicts, so we break right away.
        } until result == 0 or result == 0x1 or Length(attemptResults) >= maxAttempts;

        attemptResults
    }
}
