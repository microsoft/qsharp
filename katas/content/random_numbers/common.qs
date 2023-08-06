namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;

    // ------------------------------------------------------
    /// # Summary
    /// Helper operation to rerun test operation several times
    /// (a single run can fail with non-negligible probability even for a correct solution).
    /// # Input
    /// ## testingHarness
    /// Test operation which verifies the user's solution.
    operation RetryTestOperation (testingHarness : (Unit => Bool)) : Bool {
        let numRetries = 3;
        mutable sufficientlyRandom = false;
        mutable attemptNum = 1;
        repeat {
            set sufficientlyRandom = testingHarness();
            set attemptNum += 1;
        } until (sufficientlyRandom or attemptNum > numRetries);

        if not sufficientlyRandom {
            Message("Failed to generate sufficiently random integer");
        }

        return sufficientlyRandom;
    }

    //operation CheckRandomCalls () : Unit {
    //    Fact(GetOracleCallsCount(DrawRandomInt) == 0, "You are not allowed to call DrawRandomInt() in this task");
    //    Fact(GetOracleCallsCount(DrawRandomDouble) == 0, "You are not allowed to call DrawRandomDouble() in this task");
    //    ResetOracleCallsCount();
    //}

    // ------------------------------------------------------
    /// # Summary
    /// Helper operation that checks that the given RNG operation generates a uniform distribution.
    /// # Input
    /// ## op
    /// Random number generation operation to be tested.
    /// The parameters to this operation are provided by the caller using Delay().
    /// ## min, max
    /// Minimal and maximal numbers in the range to be generated, inclusive.
    /// ## nRuns
    /// The number of random numbers to generate for test.
    operation CheckUniformDistribution (op : (Unit => Int), min : Int, max : Int, nRuns : Int) : Bool {
        let idealMean = 0.5 * IntAsDouble(max + min) ;
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

        let idealCopiesGenerated = IntAsDouble(nRuns) / IntAsDouble(max-min+1);
        let minimumCopiesGenerated = (0.8 * idealCopiesGenerated > 40.0) ? 0.8 * idealCopiesGenerated | 0.0;

        mutable counts = [0, size = max + 1];
        mutable average = 0.0;

        //ResetOracleCallsCount();
        for i in 1..nRuns {
            let val = op();
            if (val < min or val > max) {
                Message($"Unexpected number generated. Expected values from {min} to {max}, generated {val}");
                return false;
            }
            set average += IntAsDouble(val);
            set counts w/= val <- counts[val] + 1;
        }
        //CheckRandomCalls();

        set average = average / IntAsDouble(nRuns);
        if (average < lowRange or average > highRange) {
            Message($"Unexpected average of generated numbers. Expected between {lowRange} and {highRange}, got {average}");
            return false;
        }

        let median = FindMedian (counts, max+1, nRuns);
        if (median < Floor(lowRange) or median > Ceiling(highRange)) {
            Message($"Unexpected median of generated numbers. Expected between {Floor(lowRange)} and {Ceiling(highRange)}, got {median}.");
            return false;
        }

        for i in min..max {
            if counts[i] < Floor(minimumCopiesGenerated) {
                Message($"Unexpectedly low number of {i}'s generated. Only {counts[i]} out of {nRuns} were {i}");
                return false;
            }
        }
        return true;
    }

    operation FindMedian (counts : Int[], arrSize : Int, sampleSize : Int) : Int {
        mutable totalCount = 0;
        for i in 0 .. arrSize - 1 {
            set totalCount = totalCount + counts[i];
            if totalCount >= sampleSize / 2 {
                return i;
            }
        }
        return -1;
    }


}
