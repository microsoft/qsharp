namespace Kata.Verification {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;

    @EntryPoint()
    operation CheckSolution(): Bool {
        for x in [0.0, 0.25, 0.5, 0.75, 1.0] {
            Message($"Testing generating zero with {x*100.0}% probability...");

            if not RetryTestOperation(() =>
                CheckXPercentZero(() =>
                    Kata.WeightedRandomBit(x), x)) {
                return false;
            }

            Message($"Test passed for generating zero with {x*100.0}% probability");
        }
        Message("All tests passed.");
        return true;
    }

    /// # Summary
    /// Helper operation that checks that the given RNG operation generates zero with x percent probability
    /// # Input
    /// ## op
    /// Random number generation operation to be tested.
    /// ## x
    /// Probability of generating zero
    operation CheckXPercentZero (op : (Unit => Int), x : Double) : Bool {
        mutable oneCount = 0;
        let nRuns = 1000;
        for N in 1..nRuns {
            let val = op();
            if (val < 0 or val > 1) {
                Message($"Unexpected number generated. Expected 0 or 1, instead generated {val}");
                return false;
            }
            set oneCount += val;
        }

        let zeroCount = nRuns - oneCount;
        let goalZeroCount = (x == 0.0) ? 0 | (x == 1.0) ? nRuns | Floor(x * IntAsDouble(nRuns));
        // We don't have tests with probabilities near 0.0 or 1.0, so for those the matching has to be exact
        if (goalZeroCount == 0 or goalZeroCount == nRuns) {
            if zeroCount != goalZeroCount {
                Message($"Expected {x * 100.0}% 0's, instead got {zeroCount} 0's out of {nRuns}");
                return false;
            }
        } else {
            if zeroCount < goalZeroCount - 4 * nRuns / 100 {
                Message($"Unexpectedly low number of 0's generated: expected around {x * IntAsDouble(nRuns)} 0's, got {zeroCount} out of {nRuns}");
                return false;
            } elif zeroCount > goalZeroCount + 4 * nRuns / 100 {
                Message($"Unexpectedly high number of 0's generated: expected around {x * IntAsDouble(nRuns)} 0's, got {zeroCount} out of {nRuns}");
                return false;
            }
        }
        return true;
    }

}
